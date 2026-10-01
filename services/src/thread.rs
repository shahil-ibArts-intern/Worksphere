//! Thread service — business logic for message threads.

use std::sync::Arc;
use tracing::{info, instrument};

use domain::error::{AppError, AppResult};
use domain::ids::{ChannelId, MessageId, OrgId, UserId};
use domain::traits::MessageRepository;
use domain::types::message::{Message, MessageType};
use realtime::{EventBuilder, RealtimeEvent};

/// Service for thread-related business logic.
pub struct ThreadService {
    message_repo: Arc<dyn MessageRepository>,
    realtime: Option<Arc<dyn RealtimePublisher>>,
}

/// Trait for publishing real-time events.
#[async_trait::async_trait]
pub trait RealtimePublisher: Send + Sync {
    async fn publish(&self, event: RealtimeEvent) -> Result<(), String>;
}

impl ThreadService {
    /// Creates a new thread service.
    pub fn new(message_repo: Arc<dyn MessageRepository>) -> Self {
        Self {
            message_repo,
            realtime: None,
        }
    }

    /// Creates a new thread service with real-time publishing.
    pub fn with_realtime(
        message_repo: Arc<dyn MessageRepository>,
        realtime: Arc<dyn RealtimePublisher>,
    ) -> Self {
        Self {
            message_repo,
            realtime: Some(realtime),
        }
    }

    /// Gets all replies in a thread.
    #[instrument(skip(self))]
    pub async fn get_thread_replies(
        &self,
        parent_id: MessageId,
        page: i64,
        per_page: i64,
    ) -> AppResult<Vec<Message>> {
        self.message_repo
            .list_thread(parent_id, page, per_page)
            .await
    }

    /// Replies to a message in a thread.
    #[instrument(skip(self))]
    pub async fn reply_in_thread(
        &self,
        channel_id: ChannelId,
        org_id: OrgId,
        user_id: UserId,
        parent_id: MessageId,
        content: &str,
        client_message_id: Option<MessageId>,
    ) -> AppResult<Message> {
        // Verify parent message exists
        let parent = self
            .message_repo
            .find_by_id(parent_id)
            .await?
            .ok_or_else(|| AppError::NotFound {
                resource: "Message".to_string(),
                id: parent_id.to_string(),
            })?;

        // Check if parent is in a thread (has parent_id) or is a root message
        let thread_root_id = parent.parent_id.unwrap_or(parent_id);

        let message = self
            .message_repo
            .create_with_id(
                channel_id,
                user_id,
                content,
                Some(thread_root_id),
                client_message_id,
            )
            .await?;

        // Update message type to ThreadReply
        let mut updated_message = message.clone();
        updated_message.message_type = MessageType::ThreadReply;
        let updated = self.message_repo.update(&updated_message).await?;

        info!(
            thread_root_id = %thread_root_id,
            parent_id = %parent_id,
            user_id = %user_id,
            "Thread reply created"
        );

        // Broadcast real-time event
        if let Some(ref realtime) = self.realtime {
            let event = EventBuilder::message_new(
                org_id,
                channel_id,
                updated.id,
                user_id,
                &updated.content,
            );
            let _ = realtime.publish(event).await;
        }

        Ok(updated)
    }

    /// Gets thread reply count for a message.
    #[instrument(skip(self))]
    pub async fn get_thread_reply_count(&self, parent_id: MessageId) -> AppResult<i64> {
        // We'll count messages that have this parent_id
        // This could be optimized with a dedicated query
        let replies = self.message_repo.list_thread(parent_id, 1, 1000).await?;
        Ok(replies.len() as i64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_thread_service_creation() {
        // Placeholder - real tests would need a database connection
        assert!(true);
    }
}
