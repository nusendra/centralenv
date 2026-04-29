use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;

use crate::{AppState, auth::AdminSession, error::{AppError, Result}, models::Environment};

#[derive(Deserialize)]
pub struct EnvBody {
    pub name: String,
}

pub async fn list(
    _: AdminSession,
    State(state): State<AppState>,
    Path(project_id): Path<String>,
) -> Result<Json<Vec<Environment>>> {
    let rows = sqlx::query_as!(
        Environment,
        r#"SELECT id as "id!", project_id as "project_id!", name as "name!", created_at as "created_at!" FROM environments WHERE project_id = ? ORDER BY name"#,
        project_id
    )
    .fetch_all(&state.db)
    .await?;
    Ok(Json(rows))
}

pub async fn create(
    _: AdminSession,
    State(state): State<AppState>,
    Path(project_id): Path<String>,
    Json(body): Json<EnvBody>,
) -> Result<(StatusCode, Json<Environment>)> {
    let id = uuid::Uuid::new_v4().to_string();
    sqlx::query!(
        "INSERT INTO environments (id, project_id, name) VALUES (?, ?, ?)",
        id, project_id, body.name
    )
    .execute(&state.db)
    .await?;

    let row = sqlx::query_as!(
        Environment,
        r#"SELECT id as "id!", project_id as "project_id!", name as "name!", created_at as "created_at!" FROM environments WHERE id = ?"#,
        id
    )
    .fetch_one(&state.db)
    .await?;
    Ok((StatusCode::CREATED, Json(row)))
}

pub async fn delete(
    _: AdminSession,
    State(state): State<AppState>,
    Path((project_id, env_id)): Path<(String, String)>,
) -> Result<StatusCode> {
    let affected = sqlx::query!(
        "DELETE FROM environments WHERE id = ? AND project_id = ?",
        env_id, project_id
    )
    .execute(&state.db)
    .await?
    .rows_affected();

    if affected == 0 {
        return Err(AppError::NotFound);
    }
    Ok(StatusCode::NO_CONTENT)
}
