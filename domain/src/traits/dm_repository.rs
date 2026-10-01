use async_trait::async_trait;

use crate::error::AppResult;
use crate::ids::{DmConversationId, MessageId, OrgId, UserId};
use crate::types::dm::{DmConversation, DmParticipant};
use crate::types::message::Message;

/// Repository trait for direct message data access.
#[async_trait]
pub trait DmRepository: Send + Sync {
    /// Creates a new DM conversation.
    async fn create_conversation(&self, conversation: &DmConversation)
        -> AppResult<DmConversation>;

    /// Finds a DM conversation by ID.
    async fn find_conversation_by_id(
        &self,
        id: DmConversationId,
    ) -> AppResult<Option<DmConversation>>;

    /// Finds a 1:1 DM conversation between two users.
    async fn find_direct_conversation(
        &self,
        org_id: OrgId,
        user1: UserId,
        user2: UserId,
    ) -> AppResult<Option<DmConversation>>;

    /// Lists DM conversations for a user.
    async fn list_user_conversations(
        &self,
        user_id: UserId,
        page: i64,
        per_page: i64,
    ) -> AppResult<Vec<DmConversation>>;

    /// Adds a participant to a DM conversation.
    async fn add_participant(&self, participant: &DmParticipant) -> AppResult<DmParticipant>;

    /// Removes a participant from a DM conversation.
    async fn remove_participant(
        &self,
        conversation_id: DmConversationId,
        user_id: UserId,
    ) -> AppResult<()>;

    /// Gets participants of a DM conversation.
    async fn get_participants(
        &self,
        conversation_id: DmConversationId,
    ) -> AppResult<Vec<DmParticipant>>;

    /// Updates last read timestamp for a participant.
    async fn update_last_read(
        &self,
        conversation_id: DmConversationId,
        user_id: UserId,
    ) -> AppResult<()>;

    /// Creates a message in a DM conversation.
    async fn create_message(&self, message: &Message) -> AppResult<Message>;

    /// Lists messages in a DM conversation.
    async fn list_conversation_messages(
        &self,
        conversation_id: DmConversationId,
        cursor: Option<MessageId>,
        limit: i64,
    ) -> AppResult<Vec<Message>>;
}
