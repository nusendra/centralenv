mod auth;
mod projects;
mod environments;
mod variables;
mod tokens;

use axum::{Router, routing::{delete, get, post, put}};
use crate::AppState;

pub fn auth_router() -> Router<AppState> {
    Router::new()
        .route("/login", post(auth::login))
        .route("/logout", post(auth::logout))
        .route("/me", get(auth::me))
}

pub fn api_router() -> Router<AppState> {
    Router::new()
        // Projects
        .route("/projects", get(projects::list).post(projects::create))
        .route("/projects/:id", get(projects::get).put(projects::update).delete(projects::delete))
        // Environments
        .route("/projects/:project_id/environments", get(environments::list).post(environments::create))
        .route("/projects/:project_id/environments/:env_id", delete(environments::delete))
        // Variables
        .route("/environments/:env_id/variables", get(variables::list).post(variables::upsert))
        .route("/environments/:env_id/variables/:key", delete(variables::delete_var))
        // Env export (used by CLI)
        .route("/projects/:slug/env", get(variables::export_env))
        // Tokens
        .route("/tokens", get(tokens::list).post(tokens::create))
        .route("/tokens/:id", delete(tokens::delete))
}
