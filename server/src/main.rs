mod db;
mod models;
mod routes;
mod crypto;
mod auth;
mod error;

use axum::Router;
use axum::http::{Method, header};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use std::str::FromStr;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub type Db = sqlx::SqlitePool;

#[derive(Clone)]
pub struct AppState {
    pub db: Db,
    pub master_key: Vec<u8>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "centralenv_server=debug,tower_http=debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite://centralenv.db".into());
    let master_key_b64 = std::env::var("MASTER_KEY").expect("MASTER_KEY env var required");
    let master_key = base64::Engine::decode(
        &base64::engine::general_purpose::STANDARD,
        master_key_b64.trim(),
    )?;
    if master_key.len() != 32 {
        anyhow::bail!("MASTER_KEY must be exactly 32 bytes (base64-encoded)");
    }

    let options = SqliteConnectOptions::from_str(&database_url)?
        .create_if_missing(true)
        .foreign_keys(true);

    let db = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(options)
        .await?;

    sqlx::migrate!("./migrations").run(&db).await?;

    db::seed_admin(&db).await?;

    let state = AppState { db, master_key };

    let cors = CorsLayer::new()
        .allow_origin(tower_http::cors::Any)
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE, Method::OPTIONS])
        .allow_headers([header::CONTENT_TYPE, header::AUTHORIZATION]);

    let app = Router::new()
        .nest("/api", routes::api_router())
        .nest("/auth", routes::auth_router())
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        .with_state(state);

    let bind_addr = std::env::var("BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:3001".into());
    let listener = tokio::net::TcpListener::bind(&bind_addr).await?;
    tracing::info!("Listening on {}", bind_addr);

    axum::serve(listener, app).await?;
    Ok(())
}
