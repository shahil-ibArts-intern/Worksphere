use async_trait::async_trait;

use crate::error::AppResult;
use crate::ids::MessageId;
use crate::types::message_history::MessageHistory;

/// Repository trait for message history data access.
#[async_trait]
pub trait MessageHistoryRepository: Send + Sync {
    /// Creates a history entry.
    async fn create(&self, history: &MessageHistory) -> AppResult<MessageHistory>;

    /// Lists history entries for a message.
    async fn list_by_message(
        &self,
        message_id: MessageId,
        page: i64,
        per_page: i64,
    ) -> AppResult<Vec<MessageHistory>>;

    /// Counts history entries for a message.
    async fn count_by_message(&self, message_id: MessageId) -> AppResult<i64>;
}
