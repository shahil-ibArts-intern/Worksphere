use serde::Deserialize;

/// Application configuration loaded from environment variables.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub struct AppConfig {
    pub app: AppSection,
    pub database: DatabaseSection,
    pub redis: RedisSection,
    pub jwt: JwtSection,
}

/// Application section.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub struct AppSection {
    pub env: String,
    pub host: String,
    pub port: u16,
}

/// Database section.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub struct DatabaseSection {
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
}

/// Redis section.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub struct RedisSection {
    pub url: String,
}

/// JWT section.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JwtSection {
    pub secret: String,
    pub access_token_expiry: i64,
    pub refresh_token_expiry: i64,
}

impl AppConfig {
    /// Loads configuration from environment variables.
    pub fn from_env() -> anyhow::Result<Self> {
        dotenvy::dotenv().ok();

        let settings = config::Config::builder()
            .add_source(config::Environment::default().separator("__"))
            .build()?;

        let config: AppConfig = settings.try_deserialize()?;
        Ok(config)
    }
}
