use axum::{
    extract::FromRequestParts,
    http::request::Parts,
    RequestPartsExt,
};
use axum_extra::headers::{Authorization, authorization::Bearer};
use axum_extra::TypedHeader;
use sha2::{Digest, Sha256};

use crate::{AppState, error::AppError};

fn sha256_hex(input: &str) -> String {
    hex::encode(Sha256::digest(input.as_bytes()))
}

/// Extractor that validates Bearer token and returns the token row id.
pub struct TokenAuth(pub String);

#[axum::async_trait]
impl FromRequestParts<AppState> for TokenAuth {
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, Self::Rejection> {
        let TypedHeader(Authorization(bearer)) = parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await
            .map_err(|_| AppError::Unauthorized)?;

        let raw_token = bearer.token();

        let tokens = sqlx::query!(
            r#"SELECT id as "id!", token_hash as "token_hash!" FROM tokens"#
        )
        .fetch_all(&state.db)
        .await
        .map_err(AppError::Db)?;

        for t in tokens {
            if bcrypt::verify(raw_token, &t.token_hash).unwrap_or(false) {
                let _ = sqlx::query!(
                    "UPDATE tokens SET last_used_at = datetime('now') WHERE id = ?",
                    t.id
                )
                .execute(&state.db)
                .await;
                return Ok(TokenAuth(t.id));
            }
        }

        Err(AppError::Unauthorized)
    }
}

/// Extractor for admin web sessions (Bearer token issued at /auth/login).
pub struct AdminSession(pub String);

#[axum::async_trait]
impl FromRequestParts<AppState> for AdminSession {
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, Self::Rejection> {
        let TypedHeader(Authorization(bearer)) = parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await
            .map_err(|_| AppError::Unauthorized)?;

        let raw_token = bearer.token();
        let hash = sha256_hex(raw_token);

        let row = sqlx::query!(
            r#"SELECT id as "id!"
               FROM admin_sessions
               WHERE token_hash = ? AND expires_at > datetime('now')"#,
            hash
        )
        .fetch_optional(&state.db)
        .await
        .map_err(AppError::Db)?;

        match row {
            Some(r) => Ok(AdminSession(r.id)),
            None => Err(AppError::Unauthorized),
        }
    }
}
