//! WebSocket server with Axum upgrade, connection management, and event routing.
//!
//! This module implements the WebSocket endpoint at `WS /api/v1/ws?token={jwt}`.
//! It handles connection authentication, message routing, and graceful shutdown.

use std::net::SocketAddr;
use std::sync::Arc;

use axum::extract::ws::{Message, WebSocket};
use axum::extract::{ConnectInfo, Query, State, WebSocketUpgrade};
use axum::response::IntoResponse;
use dashmap::DashMap;
use serde::Deserialize;
use tokio::sync::mpsc;
use tracing::{debug, info, instrument, warn};
use uuid::Uuid;

use auth::JwtManager;
use domain::error::AppError;
use domain::ids::{ChannelId, OrgId, UserId};

use crate::connection::{ConnectionId, ConnectionState};
use crate::events::{EventBuilder, RealtimeEvent, MAX_MESSAGE_SIZE};
use crate::heartbeat::Heartbeat;
use crate::pubsub::PubSub;
use crate::typing::TypingTracker;

/// Maximum number of messages per second per connection (W5 rule).
#[allow(dead_code)]
const MAX_MESSAGES_PER_SEC: u32 = 100;

/// Query parameters for the WebSocket endpoint.
#[derive(Debug, Deserialize)]
pub struct WsQueryParams {
    /// JWT access token for authentication.
    pub token: String,
}

/// Shared state for the WebSocket server.
#[derive(Clone)]
pub struct WsServerState {
    /// JWT manager for token validation.
    pub jwt: JwtManager,
    /// Connection registry — maps connection IDs to connection states.
    pub connections: Arc<DashMap<ConnectionId, Arc<ConnectionState>>>,
    /// Pub/sub for cross-instance fan-out.
    pub pubsub: Option<Arc<dyn PubSub>>,
    /// Typing indicator tracker.
    pub typing_tracker: Arc<TypingTracker>,
}

impl WsServerState {
    /// Creates a new WebSocket server state.
    pub fn new(jwt: JwtManager) -> Self {
        Self {
            jwt,
            connections: Arc::new(DashMap::new()),
            pubsub: None,
            typing_tracker: Arc::new(TypingTracker::new()),
        }
    }

    /// Creates a new WebSocket server state with pub/sub.
    pub fn with_pubsub(jwt: JwtManager, pubsub: Arc<dyn PubSub>) -> Self {
        Self {
            jwt,
            connections: Arc::new(DashMap::new()),
            pubsub: Some(pubsub),
            typing_tracker: Arc::new(TypingTracker::new()),
        }
    }

    /// Creates a new WebSocket server state with custom typing tracker.
    pub fn with_typing_tracker(jwt: JwtManager, typing_tracker: Arc<TypingTracker>) -> Self {
        Self {
            jwt,
            connections: Arc::new(DashMap::new()),
            pubsub: None,
            typing_tracker,
        }
    }

    /// Returns the number of active connections.
    pub fn connection_count(&self) -> usize {
        self.connections.len()
    }

    /// Broadcasts an event to all connections subscribed to a channel.
    pub fn broadcast_to_channel(&self, channel_id: ChannelId, event: &RealtimeEvent) {
        let mut sent = 0;
        for entry in self.connections.iter() {
            let conn = entry.value();
            if conn.is_subscribed(&channel_id) && conn.send_event(event) {
                sent += 1;
            }
        }
        debug!(
            channel_id = %channel_id,
            recipients = sent,
            "Broadcast event to channel"
        );
    }

    /// Broadcasts an event to all connections in an org.
    pub fn broadcast_to_org(&self, org_id: OrgId, event: &RealtimeEvent) {
        let mut sent = 0;
        for entry in self.connections.iter() {
            let conn = entry.value();
            if conn.org_id() == org_id && conn.send_event(event) {
                sent += 1;
            }
        }
        debug!(
            org_id = %org_id,
            recipients = sent,
            "Broadcast event to org"
        );
    }

    /// Sends an event to a specific user.
    pub fn send_to_user(&self, user_id: UserId, event: &RealtimeEvent) -> bool {
        for entry in self.connections.iter() {
            let conn = entry.value();
            if conn.user_id() == user_id && conn.send_event(event) {
                return true;
            }
        }
        false
    }

    /// Removes a connection from the registry.
    pub fn remove_connection(&self, id: ConnectionId) {
        self.connections.remove(&id);
        debug!(connection_id = %id, "Connection removed from registry");
    }
}

/// WebSocket upgrade handler.
///
/// This is the entry point for WebSocket connections. It:
/// 1. Extracts the JWT token from query parameters
/// 2. Validates the token
/// 3. Upgrades the connection
/// 4. Spawns the connection handler task
#[instrument(skip(ws, state, params))]
pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<WsServerState>,
    Query(params): Query<WsQueryParams>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> Result<impl IntoResponse, AppError> {
    // Validate the JWT token
    let claims = state
        .jwt
        .validate_access_token(&params.token)
        .map_err(|e| {
            warn!(error = %e, "WebSocket connection rejected: invalid token");
            AppError::Authentication("Invalid or expired token".to_string())
        })?;

    let user_id = UserId::from_uuid(
        Uuid::parse_str(&claims.sub)
            .map_err(|e| AppError::Authentication(format!("Invalid user ID: {}", e)))?,
    );
    let org_id = OrgId::from_uuid(
        Uuid::parse_str(&claims.org_id)
            .map_err(|e| AppError::Authentication(format!("Invalid org ID: {}", e)))?,
    );

    info!(
        user_id = %user_id,
        org_id = %org_id,
        remote_addr = %addr,
        "WebSocket connection authenticated"
    );

    // Configure the WebSocket with max message size
    let ws = ws.max_message_size(MAX_MESSAGE_SIZE);

    Ok(ws.on_upgrade(move |socket| handle_connection(socket, state, user_id, org_id, Some(addr))))
}

/// Handles an established WebSocket connection.
#[instrument(skip(socket, state))]
async fn handle_connection(
    socket: WebSocket,
    state: WsServerState,
    user_id: UserId,
    org_id: OrgId,
    remote_addr: Option<SocketAddr>,
) {
    let (sender, mut receiver) = mpsc::unbounded_channel::<String>();

    let conn = ConnectionState::new(user_id, org_id, sender, remote_addr);
    let conn_id = conn.id();

    // Register the connection
    state.connections.insert(conn_id, conn.clone());
    info!(
        connection_id = %conn_id,
        user_id = %user_id,
        org_id = %org_id,
        "WebSocket connection established"
    );

    // Start heartbeat
    let heartbeat = Heartbeat::new(conn.clone());
    heartbeat.start();

    // Main event loop: handle receiving from WebSocket and sending to WebSocket
    let mut socket = socket;
    loop {
        tokio::select! {
            // Receive messages from the WebSocket
            result = socket.recv() => {
                match result {
                    Some(Ok(Message::Text(text))) => {
                        // Enforce max message size
                        if text.len() > MAX_MESSAGE_SIZE {
                            warn!(
                                connection_id = %conn_id,
                                size = text.len(),
                                "Message exceeds max size"
                            );
                            let error_event =
                                EventBuilder::error(4131, "Message too large");
                            conn.send_event(&error_event);
                            continue;
                        }

                        // Update heartbeat on any message
                        conn.update_heartbeat();

                        // Parse and handle the event
                        match serde_json::from_str::<RealtimeEvent>(&text) {
                            Ok(event) => {
                                handle_client_event(&state, &conn, event).await;
                            }
                            Err(e) => {
                                warn!(
                                    connection_id = %conn_id,
                                    error = %e,
                                    "Failed to parse client event"
                                );
                                let error_event =
                                    EventBuilder::error(4001, "Invalid event format");
                                conn.send_event(&error_event);
                            }
                        }
                    }
                    Some(Ok(Message::Binary(bin))) => {
                        if bin.len() > MAX_MESSAGE_SIZE {
                            warn!(
                                connection_id = %conn_id,
                                size = bin.len(),
                                "Binary message exceeds max size"
                            );
                            continue;
                        }
                        // Convert binary to text and process
                        if let Ok(text) = String::from_utf8(bin.to_vec()) {
                            conn.update_heartbeat();
                            if let Ok(event) = serde_json::from_str::<RealtimeEvent>(&text) {
                                handle_client_event(&state, &conn, event).await;
                            }
                        }
                    }
                    Some(Ok(Message::Ping(_))) => {
                        conn.update_heartbeat();
                        // Pong is handled automatically by the WebSocket protocol
                    }
                    Some(Ok(Message::Pong(_))) => {
                        conn.update_heartbeat();
                    }
                    Some(Ok(Message::Close(_))) => {
                        info!(connection_id = %conn_id, "Client closed connection");
                        conn.deactivate();
                        break;
                    }
                    Some(Err(e)) => {
                        warn!(connection_id = %conn_id, error = %e, "WebSocket receive error");
                        conn.deactivate();
                        break;
                    }
                    None => {
                        info!(connection_id = %conn_id, "WebSocket connection closed");
                        conn.deactivate();
                        break;
                    }
                }
            }
            // Send messages to the WebSocket
            msg = receiver.recv() => {
                match msg {
                    Some(text) => {
                        if conn.is_active() {
                            if let Err(e) = socket.send(Message::Text(text)).await {
                                warn!(connection_id = %conn_id, error = %e, "Failed to send message");
                                conn.deactivate();
                                break;
                            }
                        }
                    }
                    None => {
                        // Channel closed, connection is done
                        conn.deactivate();
                        break;
                    }
                }
            }
        }
    }

    // Cleanup
    conn.deactivate();
    state.remove_connection(conn_id);
    info!(connection_id = %conn_id, "WebSocket connection closed");
}

/// Handles an event received from a client.
#[instrument(skip(state, conn))]
async fn handle_client_event(
    state: &WsServerState,
    conn: &Arc<ConnectionState>,
    event: RealtimeEvent,
) {
    match &event {
        RealtimeEvent::Ping => {
            conn.send_event(&EventBuilder::pong());
        }
        RealtimeEvent::Pong => {
            conn.update_heartbeat();
        }
        RealtimeEvent::TypingStarted(e) => {
            // Check throttle before broadcasting
            if state
                .typing_tracker
                .record_typing_started(e.org_id, e.channel_id, e.user_id)
            {
                // Broadcast typing indicator to channel
                state.broadcast_to_channel(e.channel_id, &event);
            }
        }
        RealtimeEvent::TypingStopped(e) => {
            state
                .typing_tracker
                .record_typing_stopped(e.org_id, e.channel_id, e.user_id);
            // Broadcast typing stopped to channel
            state.broadcast_to_channel(e.channel_id, &event);
        }
        RealtimeEvent::PresenceChanged(e) => {
            // Broadcast presence to org
            state.broadcast_to_org(e.org_id, &event);
        }
        _ => {
            debug!(
                connection_id = %conn.id(),
                event_type = event.event_type(),
                "Received unhandled event type from client"
            );
        }
    }
}

// ── Tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ws_server_state_creation() {
        let jwt = JwtManager::new("test-secret", 3600, 86400);
        let state = WsServerState::new(jwt);
        assert_eq!(state.connection_count(), 0);
    }

    #[test]
    fn test_ws_query_params_deserialization() {
        let params: WsQueryParams = serde_json::from_str(r#"{"token":"abc123"}"#).unwrap();
        assert_eq!(params.token, "abc123");
    }

    #[test]
    fn test_max_message_size_enforced() {
        assert_eq!(MAX_MESSAGE_SIZE, 1_048_576);
    }

    #[test]
    fn test_max_messages_per_sec() {
        assert_eq!(MAX_MESSAGES_PER_SEC, 100);
    }

    #[test]
    fn test_connection_registry_add_remove() {
        let jwt = JwtManager::new("test-secret", 3600, 86400);
        let state = WsServerState::new(jwt);

        let (tx, _rx) = mpsc::unbounded_channel();
        let conn = ConnectionState::new(
            UserId::new(),
            OrgId::new(),
            tx,
            Some("127.0.0.1:12345".parse().unwrap()),
        );
        let conn_id = conn.id();

        state.connections.insert(conn_id, conn);
        assert_eq!(state.connection_count(), 1);

        state.remove_connection(conn_id);
        assert_eq!(state.connection_count(), 0);
    }

    #[test]
    fn test_broadcast_to_org() {
        let jwt = JwtManager::new("test-secret", 3600, 86400);
        let state = WsServerState::new(jwt);

        let org_id = OrgId::new();
        let (tx, mut rx) = mpsc::unbounded_channel();
        let conn = ConnectionState::new(UserId::new(), org_id, tx, None);
        let conn_id = conn.id();
        state.connections.insert(conn_id, conn);

        let event = EventBuilder::presence_changed(org_id, UserId::new(), "online", None);
        state.broadcast_to_org(org_id, &event);

        // The message should be in the channel
        assert!(rx.try_recv().is_ok());
    }

    #[test]
    fn test_send_to_user() {
        let jwt = JwtManager::new("test-secret", 3600, 86400);
        let state = WsServerState::new(jwt);

        let user_id = UserId::new();
        let (tx, mut rx) = mpsc::unbounded_channel();
        let conn = ConnectionState::new(user_id, OrgId::new(), tx, None);
        let conn_id = conn.id();
        state.connections.insert(conn_id, conn);

        let event = EventBuilder::presence_changed(OrgId::new(), user_id, "online", None);
        let sent = state.send_to_user(user_id, &event);
        assert!(sent);
        assert!(rx.try_recv().is_ok());
    }
}
