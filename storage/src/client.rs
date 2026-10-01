use std::env;

/// Storage configuration for Neon Object Storage (S3-compatible).
#[derive(Debug, Clone)]
pub struct StorageConfig {
    pub endpoint: String,
    pub access_key: String,
    pub secret_key: String,
    pub region: String,
    pub bucket: String,
}

impl StorageConfig {
    /// Loads storage configuration from environment variables.
    pub fn from_env() -> anyhow::Result<Self> {
        dotenvy::dotenv().ok();

        Ok(Self {
            endpoint: env::var("AWS_ENDPOINT_URL_S3")
                .or_else(|_| env::var("S3_ENDPOINT"))
                .unwrap_or_default(),
            access_key: env::var("AWS_ACCESS_KEY_ID")
                .or_else(|_| env::var("S3_ACCESS_KEY"))
                .unwrap_or_default(),
            secret_key: env::var("AWS_SECRET_ACCESS_KEY")
                .or_else(|_| env::var("S3_SECRET_KEY"))
                .unwrap_or_default(),
            region: env::var("AWS_REGION")
                .or_else(|_| env::var("S3_REGION"))
                .unwrap_or_else(|_| "us-east-2".to_string()),
            bucket: env::var("S3_BUCKET").unwrap_or_else(|_| "uploads".to_string()),
        })
    }
}

/// S3/Neon Object Storage client.
#[derive(Debug, Clone)]
pub struct StorageClient {
    pub config: StorageConfig,
}

impl StorageClient {
    /// Creates a new storage client from environment configuration.
    pub fn new() -> anyhow::Result<Self> {
        let config = StorageConfig::from_env()?;
        Ok(Self { config })
    }

    /// Creates a new storage client with the given configuration.
    pub fn with_config(config: StorageConfig) -> Self {
        Self { config }
    }

    /// Returns the bucket name.
    pub fn bucket(&self) -> &str {
        &self.config.bucket
    }

    /// Returns the endpoint URL.
    pub fn endpoint(&self) -> &str {
        &self.config.endpoint
    }
}

impl Default for StorageClient {
    fn default() -> Self {
        Self {
            config: StorageConfig {
                endpoint: String::new(),
                access_key: String::new(),
                secret_key: String::new(),
                region: "us-east-2".to_string(),
                bucket: "uploads".to_string(),
            },
        }
    }
}
