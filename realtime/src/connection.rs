//! Connection state management for WebSocket connections.
//!
//! Each connection tracks its user identity, subscribed channels,
//! heartbeat status, and send/receive capabilities.

use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Instant;

use dashmap::DashSet;
use parking_lot::RwLock;
use tokio::sync::mpsc;
use tracing::{debug, instrument, warn};
use uuid::Uuid;

use domain::ids::{ChannelId, OrgId, UserId};

/// Unique identifier for a WebSocket connection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ConnectionId(pub Uuid);

impl ConnectionId {
    /// Generates a new unique connection ID.
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }
}

impl Default for ConnectionId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for ConnectionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Represents the state of a single WebSocket connection.
pub struct ConnectionState {
    /// Unique connection identifier.
    pub id: ConnectionId,
    /// The authenticated user.
    pub user_id: UserId,
    /// The organization this connection belongs to.
    pub org_id: OrgId,
    /// Channels this connection is subscribed to.
    pub subscribed_channels: DashSet<ChannelId>,
    /// Timestamp of the last heartbeat received.
    pub last_heartbeat: RwLock<Instant>,
    /// Timestamp when the connection was established.
    pub connected_at: Instant,
    /// Remote address of the client.
    pub remote_addr: Option<SocketAddr>,
    /// Sender for outbound messages.
    pub sender: mpsc::UnboundedSender<String>,
    /// Whether the connection is currently active.
    pub is_active: RwLock<bool>,
}

impl ConnectionState {
    /// Creates a new connection state.
    #[instrument(skip(sender))]
    pub fn new(
        user_id: UserId,
        org_id: OrgId,
        sender: mpsc::UnboundedSender<String>,
        remote_addr: Option<SocketAddr>,
    ) -> Arc<Self> {
        let id = ConnectionId::new();
        debug!(connection_id = %id, user_id = %user_id, "New WebSocket connection");

        Arc::new(Self {
            id,
            user_id,
            org_id,
            subscribed_channels: DashSet::new(),
            last_heartbeat: RwLock::new(Instant::now()),
            connected_at: Instant::now(),
            remote_addr,
            sender,
            is_active: RwLock::new(true),
        })
    }

    /// Returns the connection ID.
    pub fn id(&self) -> ConnectionId {
        self.id
    }

    /// Returns the user ID.
    pub fn user_id(&self) -> UserId {
        self.user_id
    }

    /// Returns the org ID.
    pub fn org_id(&self) -> OrgId {
        self.org_id
    }

    /// Subscribes this connection to a channel.
    pub fn subscribe(&self, channel_id: ChannelId) {
        self.subscribed_channels.insert(channel_id);
        debug!(
            connection_id = %self.id,
            channel_id = %channel_id,
            "Subscribed to channel"
        );
    }

    /// Unsubscribes this connection from a channel.
    pub fn unsubscribe(&self, channel_id: &ChannelId) {
        self.subscribed_channels.remove(channel_id);
        debug!(
            connection_id = %self.id,
            channel_id = %channel_id,
            "Unsubscribed from channel"
        );
    }

    /// Checks if this connection is subscribed to a channel.
    pub fn is_subscribed(&self, channel_id: &ChannelId) -> bool {
        self.subscribed_channels.contains(channel_id)
    }

    /// Updates the last heartbeat timestamp.
    pub fn update_heartbeat(&self) {
        *self.last_heartbeat.write() = Instant::now();
    }

    /// Returns the duration since the last heartbeat.
    pub fn heartbeat_age(&self) -> std::time::Duration {
        self.last_heartbeat.read().elapsed()
    }

    /// Sends a message to this connection.
    ///
    /// Returns `true` if the message was sent successfully.
    pub fn send(&self, message: String) -> bool {
        if !*self.is_active.read() {
            warn!(connection_id = %self.id, "Attempted send on inactive connection");
            return false;
        }

        match self.sender.send(message) {
            Ok(_) => true,
            Err(e) => {
                warn!(connection_id = %self.id, error = %e, "Failed to send message");
                false
            }
        }
    }

    /// Sends a serialized event to this connection.
    pub fn send_event(&self, event: &crate::events::RealtimeEvent) -> bool {
        match serde_json::to_string(event) {
            Ok(json) => self.send(json),
            Err(e) => {
                warn!(connection_id = %self.id, error = %e, "Failed to serialize event");
                false
            }
        }
    }

    /// Marks this connection as inactive.
    pub fn deactivate(&self) {
        *self.is_active.write() = false;
        debug!(connection_id = %self.id, "Connection deactivated");
    }

    /// Checks if this connection is active.
    pub fn is_active(&self) -> bool {
        *self.is_active.read()
    }

    /// Returns the connection duration.
    pub fn connection_duration(&self) -> std::time::Duration {
        self.connected_at.elapsed()
    }

    /// Returns the number of subscribed channels.
    pub fn subscribed_channel_count(&self) -> usize {
        self.subscribed_channels.len()
    }
}

impl std::fmt::Debug for ConnectionState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ConnectionState")
            .field("id", &self.id)
            .field("user_id", &self.user_id)
            .field("org_id", &self.org_id)
            .field("connected_at", &self.connected_at)
            .field("is_active", &*self.is_active.read())
            .finish()
    }
}

// ── Tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    fn create_test_connection() -> Arc<ConnectionState> {
        let (tx, _rx) = mpsc::unbounded_channel();
        ConnectionState::new(
            UserId::new(),
            OrgId::new(),
            tx,
            Some("127.0.0.1:12345".parse().unwrap()),
        )
    }

    #[test]
    fn test_connection_id_unique() {
        let id1 = ConnectionId::new();
        let id2 = ConnectionId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_connection_state_creation() {
        let conn = create_test_connection();
        assert!(conn.is_active());
        assert_eq!(conn.subscribed_channel_count(), 0);
    }

    #[test]
    fn test_subscribe_unsubscribe() {
        let conn = create_test_connection();
        let channel_id = ChannelId::new();

        conn.subscribe(channel_id);
        assert!(conn.is_subscribed(&channel_id));
        assert_eq!(conn.subscribed_channel_count(), 1);

        conn.unsubscribe(&channel_id);
        assert!(!conn.is_subscribed(&channel_id));
        assert_eq!(conn.subscribed_channel_count(), 0);
    }

    #[test]
    fn test_heartbeat_update() {
        let conn = create_test_connection();
        conn.update_heartbeat();
        let age = conn.heartbeat_age();
        assert!(age < Duration::from_secs(1));
    }

    #[test]
    fn test_deactivate() {
        let conn = create_test_connection();
        assert!(conn.is_active());
        conn.deactivate();
        assert!(!conn.is_active());
    }

    #[test]
    fn test_send_on_inactive_connection() {
        let conn = create_test_connection();
        conn.deactivate();
        assert!(!conn.send("test".to_string()));
    }

    #[test]
    fn test_connection_duration() {
        let conn = create_test_connection();
        let duration = conn.connection_duration();
        assert!(duration < Duration::from_secs(1));
    }
}
