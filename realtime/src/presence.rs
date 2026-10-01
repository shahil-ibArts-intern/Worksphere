//! Presence tracking with Redis SET with TTL.
//!
//! Presence is stored in Redis as a SET per org with TTL-based expiration.
//! - Key: `presence:{org_id}`
//! - Value: SET of user IDs
//! - TTL: 60 seconds, refreshed by heartbeat

use std::collections::HashSet;

use async_trait::async_trait;
use fred::prelude::*;
use tracing::{debug, instrument};

use domain::ids::{OrgId, UserId};

/// Default presence TTL in seconds (W10 rule).
pub const PRESENCE_TTL_SECS: u64 = 60;

/// Errors that can occur in presence operations.
#[derive(Debug, thiserror::Error)]
pub enum PresenceError {
    #[error("Redis error: {0}")]
    Redis(String),

    #[error("Connection error: {0}")]
    Connection(String),
}

/// Trait for presence tracking implementations.
#[async_trait]
pub trait PresenceTracker: Send + Sync {
    /// Marks a user as online in an org.
    async fn set_online(&self, org_id: OrgId, user_id: UserId) -> Result<(), PresenceError>;

    /// Marks a user as offline in an org.
    async fn set_offline(&self, org_id: OrgId, user_id: UserId) -> Result<(), PresenceError>;

    /// Refreshes a user's presence TTL.
    async fn refresh(&self, org_id: OrgId, user_id: UserId) -> Result<(), PresenceError>;

    /// Returns all online users in an org.
    async fn get_online_users(&self, org_id: OrgId) -> Result<HashSet<UserId>, PresenceError>;

    /// Checks if a user is online in an org.
    async fn is_online(&self, org_id: OrgId, user_id: UserId) -> Result<bool, PresenceError>;

    /// Returns the count of online users in an org.
    async fn online_count(&self, org_id: OrgId) -> Result<usize, PresenceError>;
}

/// Redis-backed presence tracker.
#[derive(Clone)]
pub struct RedisPresenceTracker {
    client: RedisClient,
    ttl_secs: u64,
}

impl RedisPresenceTracker {
    /// Creates a new Redis presence tracker.
    pub async fn new(redis_url: &str) -> Result<Self, PresenceError> {
        let config = RedisConfig::from_url(redis_url)
            .map_err(|e| PresenceError::Connection(format!("Invalid Redis URL: {}", e)))?;

        let client = RedisClient::new(config, None, None, None);
        client.connect();
        client
            .wait_for_connect()
            .await
            .map_err(|e| PresenceError::Connection(format!("Failed to connect: {}", e)))?;

        debug!("Connected to Redis for presence tracking");
        Ok(Self {
            client,
            ttl_secs: PRESENCE_TTL_SECS,
        })
    }

    /// Creates a new Redis presence tracker with custom TTL.
    pub async fn with_ttl(redis_url: &str, ttl_secs: u64) -> Result<Self, PresenceError> {
        let tracker = Self::new(redis_url).await?;
        Ok(Self {
            client: tracker.client,
            ttl_secs,
        })
    }

    /// Creates a new Redis presence tracker from an existing client.
    pub fn from_client(client: RedisClient) -> Self {
        Self {
            client,
            ttl_secs: PRESENCE_TTL_SECS,
        }
    }

    /// Returns the Redis key for an org's presence set.
    fn presence_key(org_id: OrgId) -> String {
        format!("presence:{}", org_id.as_uuid())
    }
}

#[async_trait]
impl PresenceTracker for RedisPresenceTracker {
    #[instrument(skip(self))]
    async fn set_online(&self, org_id: OrgId, user_id: UserId) -> Result<(), PresenceError> {
        let key = Self::presence_key(org_id);
        let user_id_str = user_id.to_string();

        // Add user to the presence set
        let _: i64 = self
            .client
            .sadd(&key, &user_id_str)
            .await
            .map_err(|e| PresenceError::Redis(e.to_string()))?;

        // Set/refresh TTL on the key
        let _: i64 = self
            .client
            .expire(&key, self.ttl_secs as i64)
            .await
            .map_err(|e| PresenceError::Redis(e.to_string()))?;

        debug!(org_id = %org_id, user_id = %user_id, "User set online");
        Ok(())
    }

    #[instrument(skip(self))]
    async fn set_offline(&self, org_id: OrgId, user_id: UserId) -> Result<(), PresenceError> {
        let key = Self::presence_key(org_id);
        let user_id_str = user_id.to_string();

        let _: i64 = self
            .client
            .srem(&key, &user_id_str)
            .await
            .map_err(|e| PresenceError::Redis(e.to_string()))?;

        debug!(org_id = %org_id, user_id = %user_id, "User set offline");
        Ok(())
    }

    #[instrument(skip(self))]
    async fn refresh(&self, org_id: OrgId, user_id: UserId) -> Result<(), PresenceError> {
        let key = Self::presence_key(org_id);

        // Refresh TTL on the key
        let _: i64 = self
            .client
            .expire(&key, self.ttl_secs as i64)
            .await
            .map_err(|e| PresenceError::Redis(e.to_string()))?;

        debug!(org_id = %org_id, user_id = %user_id, "Presence refreshed");
        Ok(())
    }

    #[instrument(skip(self))]
    async fn get_online_users(&self, org_id: OrgId) -> Result<HashSet<UserId>, PresenceError> {
        let key = Self::presence_key(org_id);

        let members: Vec<String> = self
            .client
            .smembers(&key)
            .await
            .map_err(|e| PresenceError::Redis(e.to_string()))?;

        let users: HashSet<UserId> = members
            .iter()
            .filter_map(|s| uuid::Uuid::parse_str(s).ok())
            .map(UserId::from_uuid)
            .collect();

        debug!(org_id = %org_id, count = users.len(), "Retrieved online users");
        Ok(users)
    }

    #[instrument(skip(self))]
    async fn is_online(&self, org_id: OrgId, user_id: UserId) -> Result<bool, PresenceError> {
        let key = Self::presence_key(org_id);
        let user_id_str = user_id.to_string();

        let result: bool = self
            .client
            .sismember(&key, &user_id_str)
            .await
            .map_err(|e| PresenceError::Redis(e.to_string()))?;

        Ok(result)
    }

    #[instrument(skip(self))]
    async fn online_count(&self, org_id: OrgId) -> Result<usize, PresenceError> {
        let key = Self::presence_key(org_id);

        let count: i64 = self
            .client
            .scard(&key)
            .await
            .map_err(|e| PresenceError::Redis(e.to_string()))?;

        Ok(count as usize)
    }
}

// ── Tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_presence_key_format() {
        let org_id = OrgId::new();
        let key = RedisPresenceTracker::presence_key(org_id);
        assert!(key.starts_with("presence:"));
        assert!(key.contains(&org_id.to_string()));
    }

    #[test]
    fn test_presence_ttl_constant() {
        assert_eq!(PRESENCE_TTL_SECS, 60);
    }

    #[test]
    fn test_different_orgs_different_keys() {
        let org1 = OrgId::new();
        let org2 = OrgId::new();
        let key1 = RedisPresenceTracker::presence_key(org1);
        let key2 = RedisPresenceTracker::presence_key(org2);
        assert_ne!(key1, key2);
    }
}
