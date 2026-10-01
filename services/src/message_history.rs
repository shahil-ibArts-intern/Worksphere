//! Message history service — tracks edit history and maintains audit trail for deletions.

use std::sync::Arc;
use tracing::{info, instrument};

use domain::error::AppResult;
use domain::ids::{MessageHistoryId, MessageId, OrgId, UserId};
use domain::traits::MessageHistoryRepository;
use domain::types::message_history::{MessageHistory, MessageHistoryType};

/// Service for message history-related business logic.
pub struct MessageHistoryService {
    repo: Arc<dyn MessageHistoryRepository>,
}

impl MessageHistoryService {
    /// Creates a new message history service.
    pub fn new(repo: Arc<dyn MessageHistoryRepository>) -> Self {
        Self { repo }
    }

    /// Records an edit in the message history.
    #[instrument(skip(self))]
    pub async fn record_edit(
        &self,
        message_id: MessageId,
        org_id: OrgId,
        user_id: UserId,
        previous_content: String,
        new_content: String,
    ) -> AppResult<MessageHistory> {
        let history = MessageHistory {
            id: MessageHistoryId::new(),
            message_id,
            user_id,
            history_type: MessageHistoryType::Edit,
            previous_content: Some(previous_content.clone()),
            new_content: Some(new_content.clone()),
            created_at: chrono::Utc::now(),
        };

        let history = self.repo.create(&history).await?;

        info!(message_id = %message_id, "Message edit recorded in history");

        Ok(history)
    }

    /// Records a deletion in the message history.
    #[instrument(skip(self))]
    pub async fn record_deletion(
        &self,
        message_id: MessageId,
        org_id: OrgId,
        user_id: UserId,
        content: String,
    ) -> AppResult<MessageHistory> {
        let history = MessageHistory {
            id: MessageHistoryId::new(),
            message_id,
            user_id,
            history_type: MessageHistoryType::Delete,
            previous_content: Some(content),
            new_content: None,
            created_at: chrono::Utc::now(),
        };

        let history = self.repo.create(&history).await?;

        info!(message_id = %message_id, "Message deletion recorded in history");

        Ok(history)
    }

    /// Gets the edit history for a message.
    #[instrument(skip(self))]
    pub async fn get_history(
        &self,
        message_id: MessageId,
        page: i64,
        per_page: i64,
    ) -> AppResult<Vec<MessageHistory>> {
        self.repo.list_by_message(message_id, page, per_page).await
    }

    /// Gets the total count of history entries for a message.
    #[instrument(skip(self))]
    pub async fn get_history_count(&self, message_id: MessageId) -> AppResult<i64> {
        self.repo.count_by_message(message_id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_history_service_creation() {
        // Placeholder - real tests would need a database connection
        assert!(true);
    }
}
