use async_trait::async_trait;

use crate::error::AppResult;
use crate::ids::{ChannelId, MessageId};
use crate::types::message::Pin;

/// Repository trait for pin data access.
#[async_trait]
pub trait PinRepository: Send + Sync {
    /// Adds a pin.
    async fn add_pin(&self, pin: &Pin) -> AppResult<Pin>;

    /// Removes a pin.
    async fn remove_pin(&self, channel_id: ChannelId, message_id: MessageId) -> AppResult<()>;

    /// Gets all pins in a channel.
    async fn get_pins_by_channel(&self, channel_id: ChannelId) -> AppResult<Vec<Pin>>;

    /// Checks if a message is pinned in a channel.
    async fn is_pinned(&self, channel_id: ChannelId, message_id: MessageId) -> AppResult<bool>;

    /// Counts pins in a channel.
    async fn count_pins_in_channel(&self, channel_id: ChannelId) -> AppResult<i64>;
}
