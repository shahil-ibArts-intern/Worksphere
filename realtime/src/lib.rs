//! Realtime crate — WebSocket server, pub/sub, and presence.
//!
//! This crate depends on `domain` and `db`.
//! It provides the real-time communication infrastructure including:
//! - WebSocket server with Axum upgrade
//! - Connection state management
//! - Typed event envelope (`RealtimeEvent`)
//! - Redis pub/sub for cross-instance fan-out
//! - Heartbeat ping/pong with timeout
//! - Presence tracking with Redis SET + TTL
//! - Typing indicator tracking with throttling

pub mod connection;
pub mod events;
pub mod heartbeat;
pub mod presence;
pub mod pubsub;
pub mod server;
pub mod typing;

// Re-exports for convenience
pub use connection::{ConnectionId, ConnectionState};
pub use events::{EventBuilder, RealtimeEvent, MAX_MESSAGE_SIZE};
pub use heartbeat::{Heartbeat, DEFAULT_HEARTBEAT_INTERVAL_SECS, DEFAULT_HEARTBEAT_TIMEOUT_SECS};
pub use presence::{PresenceTracker, RedisPresenceTracker, PRESENCE_TTL_SECS};
pub use pubsub::{PubSub, PubSubError, RedisPubSub, Subscription};
pub use server::{ws_handler, WsQueryParams, WsServerState};
pub use typing::TypingTracker;
