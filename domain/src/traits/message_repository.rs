use async_trait::async_trait;

use crate::error::AppResult;
use crate::ids::{ChannelId, MessageId, UserId};
use crate::types::message::Message;

/// Repository trait for message data access.
#[async_trait]
pub trait MessageRepository: Send + Sync {
    /// Creates a new message.
    async fn create(&self, message: &Message) -> AppResult<Message>;

    /// Creates a new message with a specific ID (for idempotency).
    ///
    /// If a message with the given ID already exists, returns the existing message.
    async fn create_with_id(
        &self,
        channel_id: ChannelId,
        user_id: UserId,
        content: &str,
        parent_id: Option<MessageId>,
        message_id: Option<MessageId>,
    ) -> AppResult<Message>;

    /// Finds a message by ID.
    async fn find_by_id(&self, id: MessageId) -> AppResult<Option<Message>>;

    /// Updates a message.
    async fn update(&self, message: &Message) -> AppResult<Message>;

    /// Soft-deletes a message.
    async fn soft_delete(&self, id: MessageId, deleted_by: UserId) -> AppResult<()>;

    /// Lists messages in a channel with cursor-based pagination.
    async fn list_by_channel(
        &self,
        channel_id: ChannelId,
        cursor: Option<MessageId>,
        limit: i64,
    ) -> AppResult<Vec<Message>>;

    /// Lists messages in a thread.
    async fn list_thread(
        &self,
        parent_id: MessageId,
        page: i64,
        per_page: i64,
    ) -> AppResult<Vec<Message>>;

    /// Counts messages in a channel.
    async fn count_by_channel(&self, channel_id: ChannelId) -> AppResult<i64>;

    /// Searches messages in a channel.
    async fn search(
        &self,
        channel_id: ChannelId,
        query: &str,
        page: i64,
        per_page: i64,
    ) -> AppResult<Vec<Message>>;
}
