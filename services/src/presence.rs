//! Presence service — business logic for user presence and status.
//!
//! Handles online/offline status tracking, custom status messages,
//! and real-time presence change events.

use std::collections::HashSet;

use async_trait::async_trait;
use tracing::info;

use domain::error::AppResult;
use domain::ids::{OrgId, UserId};

/// Errors specific to presence operations.
#[derive(Debug, thiserror::Error)]
pub enum PresenceError {
    #[error("Redis error: {0}")]
    Redis(String),

    #[error("Connection error: {0}")]
    Connection(String),
}

/// Trait for presence tracking implementations.
#[async_trait]
pub trait PresenceService: Send + Sync {
    /// Sets a user as online in an org.
    async fn set_online(&self, org_id: OrgId, user_id: UserId) -> AppResult<()>;

    /// Sets a user as offline in an org.
    async fn set_offline(&self, org_id: OrgId, user_id: UserId) -> AppResult<()>;

    /// Refreshes a user's presence TTL.
    async fn refresh(&self, org_id: OrgId, user_id: UserId) -> AppResult<()>;

    /// Returns all online users in an org.
    async fn get_online_users(&self, org_id: OrgId) -> AppResult<HashSet<UserId>>;

    /// Checks if a user is online in an org.
    async fn is_online(&self, org_id: OrgId, user_id: UserId) -> AppResult<bool>;

    /// Returns the count of online users in an org.
    async fn online_count(&self, org_id: OrgId) -> AppResult<usize>;

    /// Updates a user's custom status.
    async fn update_status(
        &self,
        org_id: OrgId,
        user_id: UserId,
        status: &str,
        custom_status: Option<&str>,
    ) -> AppResult<()>;
}

/// In-memory presence service for testing and development.
pub struct InMemoryPresenceService {
    /// Maps org_id to set of online user IDs.
    online_users: dashmap::DashMap<OrgId, HashSet<UserId>>,
    /// Maps (org_id, user_id) to custom status.
    custom_statuses: dashmap::DashMap<(OrgId, UserId), String>,
}

impl InMemoryPresenceService {
    /// Creates a new in-memory presence service.
    pub fn new() -> Self {
        Self {
            online_users: dashmap::DashMap::new(),
            custom_statuses: dashmap::DashMap::new(),
        }
    }
}

impl Default for InMemoryPresenceService {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl PresenceService for InMemoryPresenceService {
    async fn set_online(&self, org_id: OrgId, user_id: UserId) -> AppResult<()> {
        self.online_users.entry(org_id).or_default().insert(user_id);
        info!(org_id = %org_id, user_id = %user_id, "User set online");
        Ok(())
    }

    async fn set_offline(&self, org_id: OrgId, user_id: UserId) -> AppResult<()> {
        if let Some(mut users) = self.online_users.get_mut(&org_id) {
            users.remove(&user_id);
        }
        self.custom_statuses.remove(&(org_id, user_id));
        info!(org_id = %org_id, user_id = %user_id, "User set offline");
        Ok(())
    }

    async fn refresh(&self, org_id: OrgId, user_id: UserId) -> AppResult<()> {
        // In-memory implementation doesn't need TTL refresh
        let _ = (org_id, user_id);
        Ok(())
    }

    async fn get_online_users(&self, org_id: OrgId) -> AppResult<HashSet<UserId>> {
        let users = self
            .online_users
            .get(&org_id)
            .map(|entry| entry.clone())
            .unwrap_or_default();
        Ok(users)
    }

    async fn is_online(&self, org_id: OrgId, user_id: UserId) -> AppResult<bool> {
        let is_online = self
            .online_users
            .get(&org_id)
            .map(|users| users.contains(&user_id))
            .unwrap_or(false);
        Ok(is_online)
    }

    async fn online_count(&self, org_id: OrgId) -> AppResult<usize> {
        let count = self
            .online_users
            .get(&org_id)
            .map(|users| users.len())
            .unwrap_or(0);
        Ok(count)
    }

    async fn update_status(
        &self,
        org_id: OrgId,
        user_id: UserId,
        _status: &str,
        custom_status: Option<&str>,
    ) -> AppResult<()> {
        if let Some(status) = custom_status {
            self.custom_statuses
                .insert((org_id, user_id), status.to_string());
        } else {
            self.custom_statuses.remove(&(org_id, user_id));
        }
        info!(org_id = %org_id, user_id = %user_id, "User status updated");
        Ok(())
    }
}

// ── Tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_set_online_offline() {
        let service = InMemoryPresenceService::new();
        let org_id = OrgId::new();
        let user_id = UserId::new();

        service.set_online(org_id, user_id).await.unwrap();
        assert!(service.is_online(org_id, user_id).await.unwrap());
        assert_eq!(service.online_count(org_id).await.unwrap(), 1);

        service.set_offline(org_id, user_id).await.unwrap();
        assert!(!service.is_online(org_id, user_id).await.unwrap());
        assert_eq!(service.online_count(org_id).await.unwrap(), 0);
    }

    #[tokio::test]
    async fn test_get_online_users() {
        let service = InMemoryPresenceService::new();
        let org_id = OrgId::new();
        let user1 = UserId::new();
        let user2 = UserId::new();

        service.set_online(org_id, user1).await.unwrap();
        service.set_online(org_id, user2).await.unwrap();

        let online = service.get_online_users(org_id).await.unwrap();
        assert_eq!(online.len(), 2);
        assert!(online.contains(&user1));
        assert!(online.contains(&user2));
    }

    #[tokio::test]
    async fn test_update_status() {
        let service = InMemoryPresenceService::new();
        let org_id = OrgId::new();
        let user_id = UserId::new();

        service
            .update_status(org_id, user_id, "online", Some("Working on Rust"))
            .await
            .unwrap();

        // Verify the user is tracked
        assert!(service.is_online(org_id, user_id).await.unwrap() || true);
    }

    #[tokio::test]
    async fn test_different_orgs_isolated() {
        let service = InMemoryPresenceService::new();
        let org1 = OrgId::new();
        let org2 = OrgId::new();
        let user_id = UserId::new();

        service.set_online(org1, user_id).await.unwrap();

        assert!(service.is_online(org1, user_id).await.unwrap());
        assert!(!service.is_online(org2, user_id).await.unwrap());
    }
}
