use auth::JwtManager;
use axum::Router;
use db::{create_pool, DatabaseConfig};
use sqlx::PgPool;
use tracing::info;

use crate::config::AppConfig;

/// Application state shared across all components.
#[allow(dead_code)]
pub struct AppState {
    pub config: AppConfig,
    pub db: PgPool,
    pub jwt: JwtManager,
}

/// Initializes the application state.
pub async fn initialize() -> anyhow::Result<AppState> {
    let config = AppConfig::from_env()?;

    info!("Creating database connection pool...");
    let db_config = DatabaseConfig {
        url: config.database.url.clone(),
        max_connections: config.database.max_connections,
        min_connections: config.database.min_connections,
        acquire_timeout: std::time::Duration::from_secs(30),
        idle_timeout: std::time::Duration::from_secs(600),
        max_lifetime: std::time::Duration::from_secs(1800),
    };
    let db = create_pool(&db_config).await?;

    info!("Running database migrations...");
    sqlx::migrate!("../db/migrations").run(&db).await?;

    let jwt = JwtManager::new(
        &config.jwt.secret,
        config.jwt.access_token_expiry,
        config.jwt.refresh_token_expiry,
    );

    info!("Application state initialized successfully");

    Ok(AppState { config, db, jwt })
}

/// Creates the API router.
pub fn create_router(state: AppState) -> Router {
    let api_state = api::ApiState::new(state.db.clone(), state.jwt.clone());
    api::create_router(api_state)
}
