use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;

use crate::{AppState, auth::AdminSession, error::{AppError, Result}, models::Project};

#[derive(Deserialize)]
pub struct ProjectBody {
    pub name: String,
    pub slug: String,
}

pub async fn list(_: AdminSession, State(state): State<AppState>) -> Result<Json<Vec<Project>>> {
    let rows = sqlx::query_as!(
        Project,
        r#"SELECT id as "id!", name as "name!", slug as "slug!", created_at as "created_at!" FROM projects ORDER BY name"#
    )
    .fetch_all(&state.db)
    .await?;
    Ok(Json(rows))
}

pub async fn get(
    _: AdminSession,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Project>> {
    let row = sqlx::query_as!(
        Project,
        r#"SELECT id as "id!", name as "name!", slug as "slug!", created_at as "created_at!" FROM projects WHERE id = ?"#,
        id
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::NotFound)?;
    Ok(Json(row))
}

pub async fn create(
    _: AdminSession,
    State(state): State<AppState>,
    Json(body): Json<ProjectBody>,
) -> Result<(StatusCode, Json<Project>)> {
    let id = uuid::Uuid::new_v4().to_string();
    sqlx::query!(
        "INSERT INTO projects (id, name, slug) VALUES (?, ?, ?)",
        id, body.name, body.slug
    )
    .execute(&state.db)
    .await?;

    let row = sqlx::query_as!(
        Project,
        r#"SELECT id as "id!", name as "name!", slug as "slug!", created_at as "created_at!" FROM projects WHERE id = ?"#,
        id
    )
    .fetch_one(&state.db)
    .await?;
    Ok((StatusCode::CREATED, Json(row)))
}

pub async fn update(
    _: AdminSession,
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<ProjectBody>,
) -> Result<Json<Project>> {
    let affected = sqlx::query!(
        "UPDATE projects SET name = ?, slug = ? WHERE id = ?",
        body.name, body.slug, id
    )
    .execute(&state.db)
    .await?
    .rows_affected();

    if affected == 0 {
        return Err(AppError::NotFound);
    }

    let row = sqlx::query_as!(
        Project,
        r#"SELECT id as "id!", name as "name!", slug as "slug!", created_at as "created_at!" FROM projects WHERE id = ?"#,
        id
    )
    .fetch_one(&state.db)
    .await?;
    Ok(Json(row))
}

pub async fn delete(
    _: AdminSession,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode> {
    let affected = sqlx::query!("DELETE FROM projects WHERE id = ?", id)
        .execute(&state.db)
        .await?
        .rows_affected();

    if affected == 0 {
        return Err(AppError::NotFound);
    }
    Ok(StatusCode::NO_CONTENT)
}
