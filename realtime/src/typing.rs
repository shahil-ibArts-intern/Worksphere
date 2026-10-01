//! Typing indicator tracking with throttling and auto-expiry.
//!
//! - Throttle: max 1 typing event per 3 seconds per user per channel
//! - Auto-expire: typing indicator expires after 5 seconds of inactivity

use std::sync::Arc;
use std::time::{Duration, Instant};

use dashmap::DashMap;
use tracing::{debug, instrument};

use domain::ids::{ChannelId, OrgId, UserId};

/// Key for tracking typing state: (org_id, channel_id, user_id)
type TypingKey = (OrgId, ChannelId, UserId);

/// Tracks typing state for users in channels.
#[derive(Clone)]
pub struct TypingTracker {
    /// Map of typing key to last typing timestamp
    typing_state: Arc<DashMap<TypingKey, Instant>>,
    /// Throttle duration (3 seconds)
    throttle_duration: Duration,
    /// Expiry duration (5 seconds)
    expiry_duration: Duration,
}

impl TypingTracker {
    /// Creates a new typing tracker with default durations.
    pub fn new() -> Self {
        Self {
            typing_state: Arc::new(DashMap::new()),
            throttle_duration: Duration::from_secs(3),
            expiry_duration: Duration::from_secs(5),
        }
    }

    /// Creates a new typing tracker with custom durations.
    pub fn with_durations(throttle_secs: u64, expiry_secs: u64) -> Self {
        Self {
            typing_state: Arc::new(DashMap::new()),
            throttle_duration: Duration::from_secs(throttle_secs),
            expiry_duration: Duration::from_secs(expiry_secs),
        }
    }

    /// Records a typing started event.
    ///
    /// Returns `true` if the event should be broadcast (not throttled),
    /// `false` if the event was throttled.
    #[instrument(skip(self))]
    pub fn record_typing_started(
        &self,
        org_id: OrgId,
        channel_id: ChannelId,
        user_id: UserId,
    ) -> bool {
        let key = (org_id, channel_id, user_id);
        let now = Instant::now();

        if let Some(mut entry) = self.typing_state.get_mut(&key) {
            let last_typing = *entry.value();
            if now.duration_since(last_typing) < self.throttle_duration {
                debug!(
                    org_id = %org_id,
                    channel_id = %channel_id,
                    user_id = %user_id,
                    "Typing event throttled"
                );
                return false;
            }
            *entry = now;
        } else {
            self.typing_state.insert(key, now);
        }

        debug!(
            org_id = %org_id,
            channel_id = %channel_id,
            user_id = %user_id,
            "Typing started recorded"
        );
        true
    }

    /// Records a typing stopped event.
    #[instrument(skip(self))]
    pub fn record_typing_stopped(&self, org_id: OrgId, channel_id: ChannelId, user_id: UserId) {
        let key = (org_id, channel_id, user_id);
        self.typing_state.remove(&key);
        debug!(
            org_id = %org_id,
            channel_id = %channel_id,
            user_id = %user_id,
            "Typing stopped recorded"
        );
    }

    /// Checks if a user is currently typing in a channel.
    pub fn is_typing(&self, org_id: OrgId, channel_id: ChannelId, user_id: UserId) -> bool {
        let key = (org_id, channel_id, user_id);
        if let Some(entry) = self.typing_state.get(&key) {
            let last_typing = *entry.value();
            Instant::now().duration_since(last_typing) < self.expiry_duration
        } else {
            false
        }
    }

    /// Gets all users currently typing in a channel.
    pub fn get_typing_users(&self, org_id: OrgId, channel_id: ChannelId) -> Vec<UserId> {
        let now = Instant::now();
        self.typing_state
            .iter()
            .filter_map(|entry| {
                let (entry_org_id, entry_channel_id, user_id) = entry.key();
                if *entry_org_id == org_id && *entry_channel_id == channel_id {
                    let last_typing = *entry.value();
                    if now.duration_since(last_typing) < self.expiry_duration {
                        return Some(*user_id);
                    }
                }
                None
            })
            .collect()
    }

    /// Cleans up expired typing states.
    /// Should be called periodically (e.g., every 10 seconds).
    pub fn cleanup_expired(&self) {
        let now = Instant::now();
        self.typing_state
            .retain(|_, last_typing| now.duration_since(*last_typing) < self.expiry_duration);
    }

    /// Starts a background cleanup task.
    pub fn start_cleanup_task(self) {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(10));
            loop {
                interval.tick().await;
                self.cleanup_expired();
            }
        });
    }
}

impl Default for TypingTracker {
    fn default() -> Self {
        Self::new()
    }
}

// ── Tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_typing_tracker_creation() {
        let tracker = TypingTracker::new();
        assert_eq!(tracker.throttle_duration, Duration::from_secs(3));
        assert_eq!(tracker.expiry_duration, Duration::from_secs(5));
    }

    #[test]
    fn test_typing_throttle() {
        let tracker = TypingTracker::new();
        let org_id = OrgId::new();
        let channel_id = ChannelId::new();
        let user_id = UserId::new();

        // First typing event should pass
        assert!(tracker.record_typing_started(org_id, channel_id, user_id));

        // Immediate second event should be throttled
        assert!(!tracker.record_typing_started(org_id, channel_id, user_id));

        // Wait for throttle to expire
        thread::sleep(Duration::from_secs(4));

        // Third event after throttle should pass
        assert!(tracker.record_typing_started(org_id, channel_id, user_id));
    }

    #[test]
    fn test_typing_stopped() {
        let tracker = TypingTracker::new();
        let org_id = OrgId::new();
        let channel_id = ChannelId::new();
        let user_id = UserId::new();

        tracker.record_typing_started(org_id, channel_id, user_id);
        assert!(tracker.is_typing(org_id, channel_id, user_id));

        tracker.record_typing_stopped(org_id, channel_id, user_id);
        assert!(!tracker.is_typing(org_id, channel_id, user_id));
    }

    #[test]
    fn test_typing_expiry() {
        let tracker = TypingTracker::with_durations(3, 1); // 1 second expiry
        let org_id = OrgId::new();
        let channel_id = ChannelId::new();
        let user_id = UserId::new();

        tracker.record_typing_started(org_id, channel_id, user_id);
        assert!(tracker.is_typing(org_id, channel_id, user_id));

        thread::sleep(Duration::from_secs(2));

        assert!(!tracker.is_typing(org_id, channel_id, user_id));
    }

    #[test]
    fn test_get_typing_users() {
        let tracker = TypingTracker::new();
        let org_id = OrgId::new();
        let channel_id = ChannelId::new();
        let user1 = UserId::new();
        let user2 = UserId::new();

        tracker.record_typing_started(org_id, channel_id, user1);
        tracker.record_typing_started(org_id, channel_id, user2);

        let typing_users = tracker.get_typing_users(org_id, channel_id);
        assert_eq!(typing_users.len(), 2);
        assert!(typing_users.contains(&user1));
        assert!(typing_users.contains(&user2));
    }

    #[test]
    fn test_different_channels_isolated() {
        let tracker = TypingTracker::new();
        let org_id = OrgId::new();
        let channel1 = ChannelId::new();
        let channel2 = ChannelId::new();
        let user_id = UserId::new();

        tracker.record_typing_started(org_id, channel1, user_id);
        assert!(tracker.is_typing(org_id, channel1, user_id));
        assert!(!tracker.is_typing(org_id, channel2, user_id));
    }

    #[test]
    fn test_cleanup_expired() {
        let tracker = TypingTracker::with_durations(3, 1); // 1 second expiry
        let org_id = OrgId::new();
        let channel_id = ChannelId::new();
        let user_id = UserId::new();

        tracker.record_typing_started(org_id, channel_id, user_id);
        assert_eq!(tracker.typing_state.len(), 1);

        thread::sleep(Duration::from_secs(2));
        tracker.cleanup_expired();
        assert_eq!(tracker.typing_state.len(), 0);
    }
}
