use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use domain::error::AppResult;

/// OAuth provider configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OAuthProviderConfig {
    pub provider: String,
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
    pub scopes: Vec<String>,
}

/// OAuth user profile returned by providers.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OAuthUserProfile {
    pub provider: String,
    pub provider_user_id: String,
    pub email: String,
    pub name: String,
    pub avatar_url: Option<String>,
}

/// OAuth token response from providers.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OAuthTokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: Option<i64>,
    pub refresh_token: Option<String>,
    pub scope: Option<String>,
}

/// Trait for OAuth provider implementations.
#[async_trait]
pub trait OAuthProvider: Send + Sync {
    /// Returns the provider name.
    fn name(&self) -> &'static str;

    /// Generates the authorization URL for the OAuth flow.
    fn authorization_url(&self, state: &str) -> String;

    /// Exchanges an authorization code for an access token.
    async fn exchange_code(&self, code: &str) -> AppResult<OAuthTokenResponse>;

    /// Fetches the user profile using an access token.
    async fn fetch_user_profile(&self, access_token: &str) -> AppResult<OAuthUserProfile>;

    /// Refreshes an access token.
    async fn refresh_token(&self, refresh_token: &str) -> AppResult<OAuthTokenResponse>;
}

/// Registry of OAuth providers.
pub struct OAuthRegistry {
    providers: std::collections::HashMap<String, Box<dyn OAuthProvider>>,
}

impl OAuthRegistry {
    /// Creates a new empty registry.
    pub fn new() -> Self {
        Self {
            providers: std::collections::HashMap::new(),
        }
    }

    /// Registers an OAuth provider.
    pub fn register(&mut self, provider: Box<dyn OAuthProvider>) {
        self.providers.insert(provider.name().to_string(), provider);
    }

    /// Gets a provider by name.
    pub fn get(&self, name: &str) -> Option<&dyn OAuthProvider> {
        self.providers.get(name).map(|p| p.as_ref())
    }
}

impl Default for OAuthRegistry {
    fn default() -> Self {
        Self::new()
    }
}
