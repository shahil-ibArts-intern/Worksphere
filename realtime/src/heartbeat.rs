//! Heartbeat management for WebSocket connections.
//!
//! Implements ping/pong heartbeat with configurable interval and timeout.
//! - Ping interval: 30 seconds
//! - Timeout: 60 seconds without pong response

use std::sync::Arc;
use std::time::Duration;

use tokio::time::interval;
use tracing::{debug, instrument, warn};

use crate::connection::ConnectionState;

/// Default heartbeat ping interval in seconds.
pub const DEFAULT_HEARTBEAT_INTERVAL_SECS: u64 = 30;

/// Default heartbeat timeout in seconds.
pub const DEFAULT_HEARTBEAT_TIMEOUT_SECS: u64 = 60;

/// Manages heartbeat monitoring for a connection.
pub struct Heartbeat {
    /// The connection being monitored.
    connection: Arc<ConnectionState>,
    /// Interval at which to send pings.
    interval_secs: u64,
    /// Timeout after which the connection is considered dead.
    timeout_secs: u64,
}

impl Heartbeat {
    /// Creates a new heartbeat manager with default settings.
    pub fn new(connection: Arc<ConnectionState>) -> Self {
        Self {
            connection,
            interval_secs: DEFAULT_HEARTBEAT_INTERVAL_SECS,
            timeout_secs: DEFAULT_HEARTBEAT_TIMEOUT_SECS,
        }
    }

    /// Creates a new heartbeat manager with custom settings.
    pub fn with_config(
        connection: Arc<ConnectionState>,
        interval_secs: u64,
        timeout_secs: u64,
    ) -> Self {
        Self {
            connection,
            interval_secs,
            timeout_secs,
        }
    }

    /// Returns the ping interval.
    pub fn interval(&self) -> Duration {
        Duration::from_secs(self.interval_secs)
    }

    /// Returns the timeout duration.
    pub fn timeout(&self) -> Duration {
        Duration::from_secs(self.timeout_secs)
    }

    /// Starts the heartbeat monitor.
    ///
    /// This spawns a background task that:
    /// 1. Sends ping messages at the configured interval
    /// 2. Checks if the connection has timed out
    /// 3. Deactivates the connection if timed out
    #[instrument(skip(self))]
    pub fn start(self) {
        let connection = self.connection.clone();
        let interval_secs = self.interval_secs;
        let timeout_secs = self.timeout_secs;

        tokio::spawn(async move {
            let mut ticker = interval(Duration::from_secs(interval_secs));

            loop {
                ticker.tick().await;

                if !connection.is_active() {
                    debug!(
                        connection_id = %connection.id(),
                        "Connection inactive, stopping heartbeat"
                    );
                    break;
                }

                // Check if the connection has timed out
                let heartbeat_age = connection.heartbeat_age();
                if heartbeat_age > Duration::from_secs(timeout_secs) {
                    warn!(
                        connection_id = %connection.id(),
                        heartbeat_age_secs = heartbeat_age.as_secs(),
                        "Connection timed out, deactivating"
                    );
                    connection.deactivate();
                    break;
                }

                // Send ping
                let ping = crate::events::EventBuilder::ping();
                if let Ok(json) = serde_json::to_string(&ping) {
                    connection.send(json);
                    debug!(
                        connection_id = %connection.id(),
                        "Sent ping"
                    );
                }
            }
        });
    }

    /// Records a pong response from the client.
    pub fn record_pong(&self) {
        self.connection.update_heartbeat();
        debug!(connection_id = %self.connection.id(), "Recorded pong");
    }
}

// ── Tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::connection::ConnectionState;
    use domain::ids::{OrgId, UserId};
    use tokio::sync::mpsc;

    fn create_test_connection() -> Arc<ConnectionState> {
        let (tx, _rx) = mpsc::unbounded_channel();
        ConnectionState::new(UserId::new(), OrgId::new(), tx, None)
    }

    #[test]
    fn test_default_heartbeat_config() {
        let conn = create_test_connection();
        let heartbeat = Heartbeat::new(conn);
        assert_eq!(heartbeat.interval(), Duration::from_secs(30));
        assert_eq!(heartbeat.timeout(), Duration::from_secs(60));
    }

    #[test]
    fn test_custom_heartbeat_config() {
        let conn = create_test_connection();
        let heartbeat = Heartbeat::with_config(conn, 15, 30);
        assert_eq!(heartbeat.interval(), Duration::from_secs(15));
        assert_eq!(heartbeat.timeout(), Duration::from_secs(30));
    }

    #[test]
    fn test_record_pong_updates_heartbeat() {
        let conn = create_test_connection();
        let heartbeat = Heartbeat::new(conn.clone());

        // Simulate time passing
        std::thread::sleep(Duration::from_millis(10));
        let age_before = conn.heartbeat_age();

        heartbeat.record_pong();
        let age_after = conn.heartbeat_age();

        assert!(age_after < age_before);
    }

    #[test]
    fn test_constants() {
        assert_eq!(DEFAULT_HEARTBEAT_INTERVAL_SECS, 30);
        assert_eq!(DEFAULT_HEARTBEAT_TIMEOUT_SECS, 60);
    }
}
