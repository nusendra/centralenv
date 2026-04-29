use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::{AppState, auth::AdminSession, error::{AppError, Result}};

#[derive(Deserialize)]
pub struct CreateToken {
    pub name: String,
    /// Optional list of project ids to scope this token. Empty = all projects.
    pub project_ids: Option<Vec<String>>,
}

#[derive(Serialize)]
pub struct TokenCreated {
    pub id: String,
    pub name: String,
    pub token: String, // only returned once
    pub project_ids: Vec<String>,
}

#[derive(Serialize)]
pub struct TokenItem {
    pub id: String,
    pub name: String,
    pub project_ids: Vec<String>,
    pub last_used_at: Option<String>,
    pub created_at: String,
}

pub async fn list(
    _: AdminSession,
    State(state): State<AppState>,
) -> Result<Json<Vec<TokenItem>>> {
    let rows = sqlx::query!(
        r#"SELECT id as "id!", name as "name!", project_ids as "project_ids!", last_used_at, created_at as "created_at!" FROM tokens ORDER BY created_at DESC"#
    )
    .fetch_all(&state.db)
    .await?;

    let items = rows.into_iter().map(|r| TokenItem {
        id: r.id,
        name: r.name,
        project_ids: serde_json::from_str(&r.project_ids).unwrap_or_default(),
        last_used_at: r.last_used_at,
        created_at: r.created_at,
    }).collect();

    Ok(Json(items))
}

pub async fn create(
    _: AdminSession,
    State(state): State<AppState>,
    Json(body): Json<CreateToken>,
) -> Result<(StatusCode, Json<TokenCreated>)> {
    let raw_token: String = rand::thread_rng()
        .sample_iter(&rand::distributions::Alphanumeric)
        .take(48)
        .map(char::from)
        .collect();

    let hash = bcrypt::hash(&raw_token, bcrypt::DEFAULT_COST)
        .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?;

    let id = uuid::Uuid::new_v4().to_string();
    let project_ids = body.project_ids.unwrap_or_default();
    let project_ids_json = serde_json::to_string(&project_ids).unwrap();

    sqlx::query!(
        "INSERT INTO tokens (id, name, token_hash, project_ids) VALUES (?, ?, ?, ?)",
        id, body.name, hash, project_ids_json
    )
    .execute(&state.db)
    .await?;

    Ok((StatusCode::CREATED, Json(TokenCreated {
        id,
        name: body.name,
        token: raw_token,
        project_ids,
    })))
}

pub async fn delete(
    _: AdminSession,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode> {
    let affected = sqlx::query!("DELETE FROM tokens WHERE id = ?", id)
        .execute(&state.db)
        .await?
        .rows_affected();

    if affected == 0 {
        return Err(AppError::NotFound);
    }
    Ok(StatusCode::NO_CONTENT)
}
