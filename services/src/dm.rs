//! Direct message service — business logic for 1:1 and group DMs.

use std::sync::Arc;
use tracing::{info, instrument};
use uuid::Uuid;

use domain::error::{AppError, AppResult};
use domain::ids::{ChannelId, DmConversationId, MessageId, OrgId, UserId};
use domain::traits::{DmRepository, MessageRepository};
use domain::types::dm::{DmConversation, DmParticipant};
use domain::types::message::Message;
use realtime::{EventBuilder, RealtimeEvent};

/// Service for DM-related business logic.
pub struct DmService {
    dm_repo: Arc<dyn DmRepository>,
    message_repo: Arc<dyn MessageRepository>,
    realtime: Option<Arc<dyn RealtimePublisher>>,
}

/// Trait for publishing real-time events.
#[async_trait::async_trait]
pub trait RealtimePublisher: Send + Sync {
    async fn publish(&self, event: RealtimeEvent) -> Result<(), String>;
}

impl DmService {
    /// Creates a new DM service.
    pub fn new(dm_repo: Arc<dyn DmRepository>, message_repo: Arc<dyn MessageRepository>) -> Self {
        Self {
            dm_repo,
            message_repo,
            realtime: None,
        }
    }

    /// Creates a new DM service with real-time publishing.
    pub fn with_realtime(
        dm_repo: Arc<dyn DmRepository>,
        message_repo: Arc<dyn MessageRepository>,
        realtime: Arc<dyn RealtimePublisher>,
    ) -> Self {
        Self {
            dm_repo,
            message_repo,
            realtime: Some(realtime),
        }
    }

    /// Gets or creates a 1:1 DM conversation between two users.
    #[instrument(skip(self))]
    pub async fn get_or_create_direct_conversation(
        &self,
        org_id: OrgId,
        current_user: UserId,
        other_user: UserId,
    ) -> AppResult<DmConversation> {
        // Try to find existing 1:1 conversation
        if let Some(existing) = self
            .dm_repo
            .find_direct_conversation(org_id, current_user, other_user)
            .await?
        {
            return Ok(existing);
        }

        // Create new 1:1 conversation
        let conversation = DmConversation::new_direct(org_id, current_user, other_user);
        let conversation = self.dm_repo.create_conversation(&conversation).await?;

        // Add both users as participants
        let participant1 = DmParticipant {
            id: Uuid::now_v7(),
            conversation_id: conversation.id,
            user_id: current_user,
            org_id,
            joined_at: chrono::Utc::now(),
            last_read_at: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        let participant2 = DmParticipant {
            id: Uuid::now_v7(),
            conversation_id: conversation.id,
            user_id: other_user,
            org_id,
            joined_at: chrono::Utc::now(),
            last_read_at: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        self.dm_repo.add_participant(&participant1).await?;
        self.dm_repo.add_participant(&participant2).await?;

        info!(conversation_id = %conversation.id, "1:1 DM conversation created");
        Ok(conversation)
    }

    /// Creates a group DM conversation.
    #[instrument(skip(self))]
    pub async fn create_group_conversation(
        &self,
        org_id: OrgId,
        name: String,
        created_by: UserId,
        participant_ids: Vec<UserId>,
    ) -> AppResult<DmConversation> {
        let conversation = DmConversation::new_group(org_id, name, created_by);
        let conversation = self.dm_repo.create_conversation(&conversation).await?;

        // Add creator as participant
        let creator_participant = DmParticipant {
            id: Uuid::now_v7(),
            conversation_id: conversation.id,
            user_id: created_by,
            org_id,
            joined_at: chrono::Utc::now(),
            last_read_at: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        self.dm_repo.add_participant(&creator_participant).await?;

        // Add other participants
        for user_id in participant_ids {
            if user_id != created_by {
                let participant = DmParticipant {
                    id: Uuid::now_v7(),
                    conversation_id: conversation.id,
                    user_id,
                    org_id,
                    joined_at: chrono::Utc::now(),
                    last_read_at: None,
                    created_at: chrono::Utc::now(),
                    updated_at: chrono::Utc::now(),
                };
                self.dm_repo.add_participant(&participant).await?;
            }
        }

        info!(conversation_id = %conversation.id, "Group DM conversation created");
        Ok(conversation)
    }

    /// Lists DM conversations for a user.
    #[instrument(skip(self))]
    pub async fn list_conversations(
        &self,
        user_id: UserId,
        page: i64,
        per_page: i64,
    ) -> AppResult<Vec<DmConversation>> {
        self.dm_repo
            .list_user_conversations(user_id, page, per_page)
            .await
    }

    /// Gets a DM conversation by ID.
    #[instrument(skip(self))]
    pub async fn get_conversation(
        &self,
        conversation_id: DmConversationId,
    ) -> AppResult<DmConversation> {
        self.dm_repo
            .find_conversation_by_id(conversation_id)
            .await?
            .ok_or_else(|| AppError::NotFound {
                resource: "DmConversation".to_string(),
                id: conversation_id.to_string(),
            })
    }

    /// Gets participants of a DM conversation.
    #[instrument(skip(self))]
    pub async fn get_participants(
        &self,
        conversation_id: DmConversationId,
    ) -> AppResult<Vec<DmParticipant>> {
        self.dm_repo.get_participants(conversation_id).await
    }

    /// Sends a message in a DM conversation.
    #[instrument(skip(self))]
    pub async fn send_message(
        &self,
        conversation_id: DmConversationId,
        org_id: OrgId,
        user_id: UserId,
        content: &str,
        client_message_id: Option<MessageId>,
    ) -> AppResult<Message> {
        // Verify user is a participant
        let participants = self.dm_repo.get_participants(conversation_id).await?;
        if !participants.iter().any(|p| p.user_id == user_id) {
            return Err(AppError::Authorization(
                "You are not a participant in this conversation".to_string(),
            ));
        }

        // For DMs, the conversation_id is used as the channel_id in messages table
        let message = self
            .message_repo
            .create_with_id(
                ChannelId::from_uuid(conversation_id.as_uuid()),
                user_id,
                content,
                None,
                client_message_id,
            )
            .await?;

        info!(conversation_id = %conversation_id, user_id = %user_id, "DM message sent");

        // Broadcast real-time event
        if let Some(ref realtime) = self.realtime {
            let event = EventBuilder::message_new(
                org_id,
                ChannelId::from_uuid(conversation_id.as_uuid()),
                message.id,
                user_id,
                content,
            );
            let _ = realtime.publish(event).await;
        }

        Ok(message)
    }

    /// Lists messages in a DM conversation.
    #[instrument(skip(self))]
    pub async fn list_messages(
        &self,
        conversation_id: DmConversationId,
        cursor: Option<MessageId>,
        limit: i64,
    ) -> AppResult<Vec<Message>> {
        self.dm_repo
            .list_conversation_messages(conversation_id, cursor, limit)
            .await
    }

    /// Marks messages as read up to a certain point.
    #[instrument(skip(self))]
    pub async fn mark_as_read(
        &self,
        conversation_id: DmConversationId,
        user_id: UserId,
    ) -> AppResult<()> {
        self.dm_repo
            .update_last_read(conversation_id, user_id)
            .await
    }

    /// Leaves a DM conversation.
    #[instrument(skip(self))]
    pub async fn leave_conversation(
        &self,
        conversation_id: DmConversationId,
        user_id: UserId,
    ) -> AppResult<()> {
        self.dm_repo
            .remove_participant(conversation_id, user_id)
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn test_dm_conversation_creation() {
        let org_id = OrgId::new();
        let user1 = UserId::new();
        let user2 = UserId::new();

        let conv = DmConversation::new_direct(org_id, user1, user2);
        assert!(!conv.is_group);
        assert_eq!(conv.created_by, user1);
        assert!(conv.name.is_none());
    }

    #[test]
    fn test_group_dm_creation() {
        let org_id = OrgId::new();
        let creator = UserId::new();

        let conv = DmConversation::new_group(org_id, "Team Chat".to_string(), creator);
        assert!(conv.is_group);
        assert_eq!(conv.name, Some("Team Chat".to_string()));
        assert_eq!(conv.created_by, creator);
    }
}
