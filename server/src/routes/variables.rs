use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::{
    AppState,
    auth::{AdminSession, TokenAuth},
    crypto,
    error::{AppError, Result},
};

#[derive(Deserialize)]
pub struct UpsertVariable {
    pub key: String,
    pub value: String,
}

#[derive(Serialize)]
pub struct VariableResponse {
    pub id: String,
    pub key: String,
    pub value: String,
    pub updated_at: String,
}

#[derive(Deserialize)]
pub struct EnvQuery {
    pub environment: Option<String>,
}

pub async fn list(
    _: AdminSession,
    State(state): State<AppState>,
    Path(env_id): Path<String>,
) -> Result<Json<Vec<VariableResponse>>> {
    let rows = sqlx::query!(
        r#"SELECT id as "id!", key as "key!", value_encrypted as "value_encrypted!", updated_at as "updated_at!" FROM variables WHERE environment_id = ? ORDER BY key"#,
        env_id
    )
    .fetch_all(&state.db)
    .await?;

    let mut result = Vec::new();
    for row in rows {
        let value = crypto::decrypt(&state.master_key, &row.value_encrypted)
            .map_err(AppError::Internal)?;
        result.push(VariableResponse {
            id: row.id,
            key: row.key,
            value,
            updated_at: row.updated_at,
        });
    }
    Ok(Json(result))
}

pub async fn upsert(
    _: AdminSession,
    State(state): State<AppState>,
    Path(env_id): Path<String>,
    Json(body): Json<UpsertVariable>,
) -> Result<StatusCode> {
    let encrypted = crypto::encrypt(&state.master_key, &body.value)
        .map_err(AppError::Internal)?;

    let id = uuid::Uuid::new_v4().to_string();
    sqlx::query!(
        r#"INSERT INTO variables (id, environment_id, key, value_encrypted)
           VALUES (?, ?, ?, ?)
           ON CONFLICT(environment_id, key)
           DO UPDATE SET value_encrypted = excluded.value_encrypted, updated_at = datetime('now')"#,
        id, env_id, body.key, encrypted
    )
    .execute(&state.db)
    .await?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn delete_var(
    _: AdminSession,
    State(state): State<AppState>,
    Path((env_id, key)): Path<(String, String)>,
) -> Result<StatusCode> {
    let affected = sqlx::query!(
        "DELETE FROM variables WHERE environment_id = ? AND key = ?",
        env_id, key
    )
    .execute(&state.db)
    .await?
    .rows_affected();

    if affected == 0 {
        return Err(AppError::NotFound);
    }
    Ok(StatusCode::NO_CONTENT)
}

/// CLI endpoint: GET /api/projects/:slug/env?environment=dev
/// Accepts Bearer token auth. Returns vars as KEY=VALUE map.
pub async fn export_env(
    auth: TokenAuth,
    State(state): State<AppState>,
    Path(slug): Path<String>,
    Query(query): Query<EnvQuery>,
) -> Result<Json<HashMap<String, String>>> {
    let env_name = query.environment.unwrap_or_else(|| "development".into());

    let project = sqlx::query!(
        r#"SELECT id as "id!" FROM projects WHERE slug = ?"#,
        slug
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::NotFound)?;

    // Check token has access to this project
    let token = sqlx::query!(
        r#"SELECT project_ids as "project_ids!" FROM tokens WHERE id = ?"#,
        auth.0
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::Unauthorized)?;

    let allowed: Vec<String> = serde_json::from_str(&token.project_ids).unwrap_or_default();
    if !allowed.is_empty() && !allowed.contains(&project.id) {
        return Err(AppError::Unauthorized);
    }

    let environment = sqlx::query!(
        r#"SELECT id as "id!" FROM environments WHERE project_id = ? AND name = ?"#,
        project.id, env_name
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::NotFound)?;

    let rows = sqlx::query!(
        r#"SELECT key as "key!", value_encrypted as "value_encrypted!" FROM variables WHERE environment_id = ? ORDER BY key"#,
        environment.id
    )
    .fetch_all(&state.db)
    .await?;

    let mut map = HashMap::new();
    for row in rows {
        let value = crypto::decrypt(&state.master_key, &row.value_encrypted)
            .map_err(AppError::Internal)?;
        map.insert(row.key, value);
    }

    Ok(Json(map))
}
