use axum::{
    extract::State,
    http::StatusCode,
    Json,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{AppState, auth::AdminSession, error::{AppError, Result}};

fn hash_token(token: &str) -> String {
    let digest = Sha256::digest(token.as_bytes());
    hex::encode(digest)
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct LoginResponse {
    pub username: String,
    pub token: String,
}

pub async fn login(
    State(state): State<AppState>,
    Json(body): Json<LoginRequest>,
) -> Result<impl IntoResponse> {
    let user = sqlx::query!(
        r#"SELECT username as "username!", password_hash as "password_hash!" FROM admin_user WHERE username = ?"#,
        body.username
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::Unauthorized)?;

    let valid = bcrypt::verify(&body.password, &user.password_hash)
        .map_err(|_| AppError::Unauthorized)?;

    if !valid {
        return Err(AppError::Unauthorized);
    }

    let raw_token = format!("{}", uuid::Uuid::new_v4());
    let hash = hash_token(&raw_token);

    sqlx::query!(
        "INSERT INTO admin_sessions (token_hash, expires_at) VALUES (?, datetime('now', '+7 days'))",
        hash
    )
    .execute(&state.db)
    .await?;

    Ok(Json(LoginResponse {
        username: user.username,
        token: raw_token,
    }))
}

pub async fn logout(
    State(state): State<AppState>,
    session: AdminSession,
) -> Result<impl IntoResponse> {
    sqlx::query!("DELETE FROM admin_sessions WHERE id = ?", session.0)
        .execute(&state.db)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn me(_: AdminSession) -> impl IntoResponse {
    StatusCode::NO_CONTENT
}
