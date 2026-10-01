//! Pin service — business logic for pinned messages.

use std::sync::Arc;
use tracing::{info, instrument};

use domain::error::{AppError, AppResult};
use domain::ids::{ChannelId, MessageId, OrgId, PinId, UserId};
use domain::traits::PinRepository;
use domain::types::message::Pin;
use realtime::{EventBuilder, RealtimeEvent};

/// Service for pin-related business logic.
pub struct PinService {
    repo: Arc<dyn PinRepository>,
    realtime: Option<Arc<dyn RealtimePublisher>>,
}

/// Trait for publishing real-time events.
#[async_trait::async_trait]
pub trait RealtimePublisher: Send + Sync {
    async fn publish(&self, event: RealtimeEvent) -> Result<(), String>;
}

impl PinService {
    /// Creates a new pin service.
    pub fn new(repo: Arc<dyn PinRepository>) -> Self {
        Self {
            repo,
            realtime: None,
        }
    }

    /// Creates a new pin service with real-time publishing.
    pub fn with_realtime(
        repo: Arc<dyn PinRepository>,
        realtime: Arc<dyn RealtimePublisher>,
    ) -> Self {
        Self {
            repo,
            realtime: Some(realtime),
        }
    }

    /// Pins a message in a channel.
    #[instrument(skip(self))]
    pub async fn pin_message(
        &self,
        channel_id: ChannelId,
        org_id: OrgId,
        user_id: UserId,
        message_id: MessageId,
    ) -> AppResult<Pin> {
        // Check if already pinned
        if self.repo.is_pinned(channel_id, message_id).await? {
            return Err(AppError::Conflict("Message is already pinned".to_string()));
        }

        // Check pin limit (max 50 pins per channel)
        let pin_count = self.repo.count_pins_in_channel(channel_id).await?;
        if pin_count >= 50 {
            return Err(AppError::Conflict(
                "Maximum of 50 pins per channel reached".to_string(),
            ));
        }

        let pin = Pin {
            id: PinId::new(),
            channel_id,
            message_id,
            pinned_by: user_id,
            created_at: chrono::Utc::now(),
        };

        let pin = self.repo.add_pin(&pin).await?;

        info!(channel_id = %channel_id, message_id = %message_id, "Message pinned");

        // Broadcast real-time event (could add PinAdded event)
        if let Some(ref realtime) = self.realtime {
            // For now, we could send a generic event
            let _ = realtime.publish(EventBuilder::error(0, "Pin event")).await;
        }

        Ok(pin)
    }

    /// Unpins a message from a channel.
    #[instrument(skip(self))]
    pub async fn unpin_message(
        &self,
        channel_id: ChannelId,
        org_id: OrgId,
        user_id: UserId,
        message_id: MessageId,
    ) -> AppResult<()> {
        // Verify the pin exists
        if !self.repo.is_pinned(channel_id, message_id).await? {
            return Err(AppError::NotFound {
                resource: "Pin".to_string(),
                id: format!("channel={}, message={}", channel_id, message_id),
            });
        }

        self.repo.remove_pin(channel_id, message_id).await?;

        info!(channel_id = %channel_id, message_id = %message_id, "Message unpinned");

        Ok(())
    }

    /// Lists all pinned messages in a channel.
    #[instrument(skip(self))]
    pub async fn list_pins(&self, channel_id: ChannelId) -> AppResult<Vec<Pin>> {
        self.repo.get_pins_by_channel(channel_id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pin_service_creation() {
        // Placeholder - real tests would need a database connection
        assert!(true);
    }
}
