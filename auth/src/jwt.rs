use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use domain::error::{AppError, AppResult};
use domain::ids::{OrgId, UserId};

/// JWT claims for access tokens.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessTokenClaims {
    pub sub: String,    // User ID
    pub org_id: String, // Organization ID
    pub role: String,   // User role in org
    pub exp: i64,       // Expiration time
    pub iat: i64,       // Issued at
    pub jti: String,    // JWT ID (unique token identifier)
}

/// JWT claims for refresh tokens.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefreshTokenClaims {
    pub sub: String, // User ID
    pub jti: String, // JWT ID
    pub exp: i64,    // Expiration time
    pub iat: i64,    // Issued at
}

/// Token pair returned after successful authentication.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenPair {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: i64,
    pub token_type: String,
}

/// JWT token manager for creating and validating tokens.
#[derive(Clone)]
pub struct JwtManager {
    secret: String,
    access_token_expiry: Duration,
    refresh_token_expiry: Duration,
}

impl JwtManager {
    /// Creates a new JWT manager with the given configuration.
    pub fn new(
        secret: impl Into<String>,
        access_token_expiry: i64,
        refresh_token_expiry: i64,
    ) -> Self {
        Self {
            secret: secret.into(),
            access_token_expiry: Duration::seconds(access_token_expiry),
            refresh_token_expiry: Duration::seconds(refresh_token_expiry),
        }
    }

    /// Creates a new access token for a user.
    pub fn create_access_token(
        &self,
        user_id: UserId,
        org_id: OrgId,
        role: &str,
    ) -> AppResult<String> {
        let now = Utc::now();
        let exp = now + self.access_token_expiry;
        let claims = AccessTokenClaims {
            sub: user_id.to_string(),
            org_id: org_id.to_string(),
            role: role.to_string(),
            exp: exp.timestamp(),
            iat: now.timestamp(),
            jti: Uuid::now_v7().to_string(),
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.secret.as_bytes()),
        )
        .map_err(|e| AppError::Internal(format!("Failed to create access token: {}", e)))
    }

    /// Creates a new refresh token for a user.
    pub fn create_refresh_token(&self, user_id: UserId) -> AppResult<String> {
        let now = Utc::now();
        let exp = now + self.refresh_token_expiry;
        let claims = RefreshTokenClaims {
            sub: user_id.to_string(),
            jti: Uuid::now_v7().to_string(),
            exp: exp.timestamp(),
            iat: now.timestamp(),
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.secret.as_bytes()),
        )
        .map_err(|e| AppError::Internal(format!("Failed to create refresh token: {}", e)))
    }

    /// Creates a token pair (access + refresh) for a user.
    pub fn create_token_pair(
        &self,
        user_id: UserId,
        org_id: OrgId,
        role: &str,
    ) -> AppResult<TokenPair> {
        let access_token = self.create_access_token(user_id, org_id, role)?;
        let refresh_token = self.create_refresh_token(user_id)?;

        Ok(TokenPair {
            access_token,
            refresh_token,
            expires_in: self.access_token_expiry.num_seconds(),
            token_type: "Bearer".to_string(),
        })
    }

    /// Validates an access token and returns the claims.
    pub fn validate_access_token(&self, token: &str) -> AppResult<AccessTokenClaims> {
        let validation = Validation::default();
        let token_data = decode::<AccessTokenClaims>(
            token,
            &DecodingKey::from_secret(self.secret.as_bytes()),
            &validation,
        )
        .map_err(|e| AppError::Authentication(format!("Invalid access token: {}", e)))?;

        Ok(token_data.claims)
    }

    /// Validates a refresh token and returns the claims.
    pub fn validate_refresh_token(&self, token: &str) -> AppResult<RefreshTokenClaims> {
        let validation = Validation::default();
        let token_data = decode::<RefreshTokenClaims>(
            token,
            &DecodingKey::from_secret(self.secret.as_bytes()),
            &validation,
        )
        .map_err(|e| AppError::Authentication(format!("Invalid refresh token: {}", e)))?;

        Ok(token_data.claims)
    }

    /// Extracts the user ID from an access token.
    pub fn extract_user_id(&self, token: &str) -> AppResult<UserId> {
        let claims = self.validate_access_token(token)?;
        let uuid = Uuid::parse_str(&claims.sub)
            .map_err(|e| AppError::Authentication(format!("Invalid user ID in token: {}", e)))?;
        Ok(UserId::from_uuid(uuid))
    }

    /// Extracts the org ID from an access token.
    pub fn extract_org_id(&self, token: &str) -> AppResult<OrgId> {
        let claims = self.validate_access_token(token)?;
        let uuid = Uuid::parse_str(&claims.org_id)
            .map_err(|e| AppError::Authentication(format!("Invalid org ID in token: {}", e)))?;
        Ok(OrgId::from_uuid(uuid))
    }
}
