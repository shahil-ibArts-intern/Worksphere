use sqlx::postgres::{PgPool, PgPoolOptions};
use std::time::Duration;

/// Database connection pool configuration.
#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
    pub acquire_timeout: Duration,
    pub idle_timeout: Duration,
    pub max_lifetime: Duration,
}

impl DatabaseConfig {
    /// Creates a new database config with sensible defaults.
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            max_connections: 100,
            min_connections: 5,
            acquire_timeout: Duration::from_secs(30),
            idle_timeout: Duration::from_secs(600),
            max_lifetime: Duration::from_secs(1800),
        }
    }

    /// Creates a new database config with custom pool settings.
    pub fn with_pool_settings(
        url: impl Into<String>,
        max_connections: u32,
        min_connections: u32,
    ) -> Self {
        Self {
            url: url.into(),
            max_connections,
            min_connections,
            acquire_timeout: Duration::from_secs(30),
            idle_timeout: Duration::from_secs(600),
            max_lifetime: Duration::from_secs(1800),
        }
    }
}

/// Creates a new connection pool from the given configuration.
pub async fn create_pool(config: &DatabaseConfig) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(config.max_connections)
        .min_connections(config.min_connections)
        .acquire_timeout(config.acquire_timeout)
        .idle_timeout(config.idle_timeout)
        .max_lifetime(config.max_lifetime)
        .connect(&config.url)
        .await
}

/// Creates a pool from a URL string with default settings.
pub async fn create_pool_from_url(url: &str) -> Result<PgPool, sqlx::Error> {
    create_pool(&DatabaseConfig::new(url)).await
}

/// Creates a pool from the DATABASE_URL environment variable.
pub async fn create_pool_from_env() -> Result<PgPool, sqlx::Error> {
    let url = std::env::var("DATABASE_URL").expect("DATABASE_URL environment variable must be set");
    create_pool_from_url(&url).await
}

/// Creates a pool from the DATABASE_URL_UNPOOLED environment variable.
/// Use this for migrations, LISTEN/NOTIFY, and multi-round-trip transactions.
pub async fn create_pool_from_env_unpooled() -> Result<PgPool, sqlx::Error> {
    let url = std::env::var("DATABASE_URL_UNPOOLED")
        .expect("DATABASE_URL_UNPOOLED environment variable must be set");
    create_pool_from_url(&url).await
}
