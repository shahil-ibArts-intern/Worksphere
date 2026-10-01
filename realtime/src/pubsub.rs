//! Redis pub/sub integration for cross-instance event fan-out.
//!
//! Events are published to Redis channels so that multiple server instances
//! can broadcast events to all connected clients.
//!
//! Channel naming convention (per M7 rule):
//! - Channel events: `org:{org_id}:channel:{channel_id}`
//! - User events: `org:{org_id}:user:{user_id}`
//! - Org-wide events: `org:{org_id}:broadcast`

use async_trait::async_trait;
use fred::prelude::*;
use tracing::{debug, instrument};

use domain::ids::{ChannelId, OrgId, UserId};

use crate::events::RealtimeEvent;

/// Redis channel prefix for org-wide broadcasts.
const ORG_BROADCAST_PREFIX: &str = "org";

/// Trait for pub/sub implementations.
/// Allows mocking in tests.
#[async_trait]
pub trait PubSub: Send + Sync {
    /// Publishes an event to the appropriate Redis channel.
    async fn publish(&self, event: &RealtimeEvent) -> Result<(), PubSubError>;

    /// Subscribes to a channel and returns a stream of events.
    async fn subscribe(&self, channel: &str) -> Result<Subscription, PubSubError>;

    /// Returns the channel name for a given channel ID.
    fn channel_name(org_id: OrgId, channel_id: ChannelId) -> String
    where
        Self: Sized;

    /// Returns the channel name for a given user ID.
    fn user_channel_name(org_id: OrgId, user_id: UserId) -> String
    where
        Self: Sized;

    /// Returns the org broadcast channel name.
    fn org_broadcast_name(org_id: OrgId) -> String
    where
        Self: Sized;
}

/// Errors that can occur in pub/sub operations.
#[derive(Debug, thiserror::Error)]
pub enum PubSubError {
    #[error("Redis error: {0}")]
    Redis(String),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Connection error: {0}")]
    Connection(String),
}

/// Represents an active subscription to a Redis channel.
pub struct Subscription {
    pub channel: String,
    receiver: tokio::sync::mpsc::UnboundedReceiver<RealtimeEvent>,
}

impl Subscription {
    /// Creates a new subscription handle.
    pub fn new(
        channel: String,
        receiver: tokio::sync::mpsc::UnboundedReceiver<RealtimeEvent>,
    ) -> Self {
        Self { channel, receiver }
    }

    /// Returns the channel name.
    pub fn channel(&self) -> &str {
        &self.channel
    }

    /// Receives the next event from the subscription.
    pub async fn recv(&mut self) -> Option<RealtimeEvent> {
        self.receiver.recv().await
    }
}

/// Redis-backed pub/sub implementation.
#[derive(Clone)]
pub struct RedisPubSub {
    client: RedisClient,
}

impl RedisPubSub {
    /// Creates a new Redis pub/sub instance from a URL.
    pub async fn new(redis_url: &str) -> Result<Self, PubSubError> {
        let config = RedisConfig::from_url(redis_url)
            .map_err(|e| PubSubError::Connection(format!("Invalid Redis URL: {}", e)))?;

        let client = RedisClient::new(config, None, None, None);
        client.connect();
        client
            .wait_for_connect()
            .await
            .map_err(|e| PubSubError::Connection(format!("Failed to connect: {}", e)))?;

        debug!("Connected to Redis for pub/sub");
        Ok(Self { client })
    }

    /// Creates a new Redis pub/sub instance from an existing client.
    pub fn from_client(client: RedisClient) -> Self {
        Self { client }
    }

    /// Returns a reference to the underlying Redis client.
    pub fn client(&self) -> &RedisClient {
        &self.client
    }

    /// Returns the channel name for a given channel ID.
    pub fn channel_name(org_id: OrgId, channel_id: ChannelId) -> String {
        format!(
            "{}:{}:channel:{}",
            ORG_BROADCAST_PREFIX,
            org_id.as_uuid(),
            channel_id.as_uuid()
        )
    }

    /// Returns the channel name for a given user ID.
    pub fn user_channel_name(org_id: OrgId, user_id: UserId) -> String {
        format!(
            "{}:{}:user:{}",
            ORG_BROADCAST_PREFIX,
            org_id.as_uuid(),
            user_id.as_uuid()
        )
    }

    /// Returns the org broadcast channel name.
    pub fn org_broadcast_name(org_id: OrgId) -> String {
        format!("{}:{}:broadcast", ORG_BROADCAST_PREFIX, org_id.as_uuid())
    }

    /// Resolves the appropriate Redis channel for an event.
    fn resolve_channel(event: &RealtimeEvent) -> String {
        match event {
            RealtimeEvent::MessageNew(e) => Self::channel_name(e.org_id, e.channel_id),
            RealtimeEvent::MessageEdited(e) => Self::channel_name(e.org_id, e.channel_id),
            RealtimeEvent::MessageDeleted(e) => Self::channel_name(e.org_id, e.channel_id),
            RealtimeEvent::TypingStarted(e) => Self::channel_name(e.org_id, e.channel_id),
            RealtimeEvent::TypingStopped(e) => Self::channel_name(e.org_id, e.channel_id),
            RealtimeEvent::ReactionAdded(e) => Self::org_broadcast_name(e.org_id),
            RealtimeEvent::ReactionRemoved(e) => Self::org_broadcast_name(e.org_id),
            RealtimeEvent::PresenceChanged(e) => Self::org_broadcast_name(e.org_id),
            RealtimeEvent::ChannelCreated(e) => Self::org_broadcast_name(e.org_id),
            RealtimeEvent::ChannelMemberJoined(e) => Self::channel_name(e.org_id, e.channel_id),
            RealtimeEvent::TaskCreated(e) => Self::org_broadcast_name(e.org_id),
            RealtimeEvent::TaskUpdated(e) => Self::org_broadcast_name(e.org_id),
            RealtimeEvent::TaskAssigned(e) => Self::org_broadcast_name(e.org_id),
            RealtimeEvent::NotificationNew(e) => Self::user_channel_name(e.org_id, e.user_id),
            _ => {
                if let Some(org_id) = event.org_id() {
                    Self::org_broadcast_name(org_id)
                } else {
                    "broadcast".to_string()
                }
            }
        }
    }
}

#[async_trait]
impl PubSub for RedisPubSub {
    #[instrument(skip(self, event))]
    async fn publish(&self, event: &RealtimeEvent) -> Result<(), PubSubError> {
        let channel = Self::resolve_channel(event);
        let payload =
            serde_json::to_string(event).map_err(|e| PubSubError::Serialization(e.to_string()))?;

        let result: i64 = self
            .client
            .publish(&channel, &payload)
            .await
            .map_err(|e| PubSubError::Redis(e.to_string()))?;

        debug!(
            channel = %channel,
            subscribers = result,
            "Published event to Redis"
        );

        Ok(())
    }

    async fn subscribe(&self, channel: &str) -> Result<Subscription, PubSubError> {
        let (_tx, rx) = tokio::sync::mpsc::unbounded_channel();

        let client = self.client.clone();
        let channel = channel.to_string();
        let channel_clone = channel.clone();

        tokio::spawn(async move {
            let _ = client.subscribe(&channel_clone).await;

            // In fred 8.x, after subscribing, the client enters subscriber mode
            // and messages are received via the client's message stream.
            // For now, we use a simple loop that checks for messages.
            // TODO: Replace with proper fred 8.x subscriber API when available.
            loop {
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                // Placeholder: in production, use fred's subscriber message stream
                let _ = &client;
            }
        });

        Ok(Subscription::new(channel.to_string(), rx))
    }

    fn channel_name(org_id: OrgId, channel_id: ChannelId) -> String
    where
        Self: Sized,
    {
        Self::channel_name(org_id, channel_id)
    }

    fn user_channel_name(org_id: OrgId, user_id: UserId) -> String
    where
        Self: Sized,
    {
        Self::user_channel_name(org_id, user_id)
    }

    fn org_broadcast_name(org_id: OrgId) -> String
    where
        Self: Sized,
    {
        Self::org_broadcast_name(org_id)
    }
}

// ── Tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use domain::ids::{ChannelId, OrgId, UserId};
    use uuid::Uuid;

    #[test]
    fn test_channel_name_format() {
        let org_id = OrgId::from_uuid(Uuid::now_v7());
        let channel_id = ChannelId::from_uuid(Uuid::now_v7());
        let name = RedisPubSub::channel_name(org_id, channel_id);
        assert!(name.starts_with("org:"));
        assert!(name.contains(":channel:"));
    }

    #[test]
    fn test_user_channel_name_format() {
        let org_id = OrgId::from_uuid(Uuid::now_v7());
        let user_id = UserId::from_uuid(Uuid::now_v7());
        let name = RedisPubSub::user_channel_name(org_id, user_id);
        assert!(name.starts_with("org:"));
        assert!(name.contains(":user:"));
    }

    #[test]
    fn test_org_broadcast_name_format() {
        let org_id = OrgId::from_uuid(Uuid::now_v7());
        let name = RedisPubSub::org_broadcast_name(org_id);
        assert!(name.starts_with("org:"));
        assert!(name.ends_with(":broadcast"));
    }

    #[test]
    fn test_channel_names_include_org_id() {
        let org_id = OrgId::from_uuid(Uuid::now_v7());
        let channel_id = ChannelId::from_uuid(Uuid::now_v7());
        let name = RedisPubSub::channel_name(org_id, channel_id);
        assert!(name.contains(&org_id.to_string()));
    }

    #[test]
    fn test_different_orgs_produce_different_channels() {
        let org1 = OrgId::from_uuid(Uuid::now_v7());
        let org2 = OrgId::from_uuid(Uuid::now_v7());
        let channel_id = ChannelId::from_uuid(Uuid::now_v7());

        let name1 = RedisPubSub::channel_name(org1, channel_id);
        let name2 = RedisPubSub::channel_name(org2, channel_id);
        assert_ne!(name1, name2);
    }
}
