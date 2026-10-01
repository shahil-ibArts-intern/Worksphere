use serde::Deserialize;

/// Application configuration loaded from environment.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppConfig {
    pub app: AppSection,
    pub database: DatabaseSection,
    pub redis: RedisSection,
    pub jwt: JwtSection,
    pub meilisearch: MeilisearchSection,
    pub s3: S3Section,
    pub smtp: SmtpSection,
}

/// Application section of config.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSection {
    pub env: String,
    pub host: String,
    pub port: u16,
}

/// Database section of config.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DatabaseSection {
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
}

/// Redis section of config.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RedisSection {
    pub url: String,
}

/// JWT section of config.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JwtSection {
    pub secret: String,
    pub access_token_expiry: i64,
    pub refresh_token_expiry: i64,
}

/// Meilisearch section of config.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MeilisearchSection {
    pub url: String,
    pub api_key: Option<String>,
}

/// S3 section of config.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct S3Section {
    pub endpoint: String,
    pub access_key: String,
    pub secret_key: String,
    pub bucket: String,
    pub region: String,
}

/// SMTP section of config.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SmtpSection {
    pub host: String,
    pub port: u16,
    pub username: Option<String>,
    pub password: Option<String>,
    pub from: String,
}

impl AppConfig {
    /// Loads configuration from environment variables.
    pub fn from_env() -> Result<Self, config::ConfigError> {
        let settings = config::Config::builder()
            .add_source(config::Environment::default().separator("__"))
            .build()?;
        settings.try_deserialize()
    }
}
