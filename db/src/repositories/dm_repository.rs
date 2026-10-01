use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use domain::error::{AppError, AppResult};
use domain::ids::{DmConversationId, MessageId, OrgId, UserId};
use domain::traits::DmRepository;
use domain::types::dm::{DmConversation, DmParticipant};
use domain::types::message::Message;

/// SQLx implementation of the DmRepository trait.
#[derive(Clone)]
pub struct SqlxDmRepository {
    pool: PgPool,
}

impl SqlxDmRepository {
    /// Creates a new repository instance.
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl DmRepository for SqlxDmRepository {
    async fn create_conversation(
        &self,
        conversation: &DmConversation,
    ) -> AppResult<DmConversation> {
        let result = sqlx::query_as::<_, DmConversation>(
            r#"
            INSERT INTO dm_conversations (id, org_id, name, is_group, created_by)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id, org_id, name, is_group, created_by, created_at, updated_at, deleted_at
            "#,
        )
        .bind(conversation.id.as_uuid())
        .bind(conversation.org_id.as_uuid())
        .bind(&conversation.name)
        .bind(conversation.is_group)
        .bind(conversation.created_by.as_uuid())
        .fetch_one(&self.pool)
        .await?;

        Ok(result)
    }

    async fn find_conversation_by_id(
        &self,
        id: DmConversationId,
    ) -> AppResult<Option<DmConversation>> {
        let result = sqlx::query_as::<_, DmConversation>(
            r#"
            SELECT id, org_id, name, is_group, created_by, created_at, updated_at, deleted_at
            FROM dm_conversations
            WHERE id = $1 AND deleted_at IS NULL
            "#,
        )
        .bind(id.as_uuid())
        .fetch_optional(&self.pool)
        .await?;

        Ok(result)
    }

    async fn find_direct_conversation(
        &self,
        org_id: OrgId,
        user1: UserId,
        user2: UserId,
    ) -> AppResult<Option<DmConversation>> {
        let result = sqlx::query_as::<_, DmConversation>(
            r#"
            SELECT dc.id, dc.org_id, dc.name, dc.is_group, dc.created_by, dc.created_at, dc.updated_at, dc.deleted_at
            FROM dm_conversations dc
            INNER JOIN dm_participants dp1 ON dp1.conversation_id = dc.id
            INNER JOIN dm_participants dp2 ON dp2.conversation_id = dc.id
            WHERE dc.org_id = $1
              AND dc.is_group = FALSE
              AND dc.deleted_at IS NULL
              AND dp1.user_id = $2
              AND dp2.user_id = $3
            "#,
        )
        .bind(org_id.as_uuid())
        .bind(user1.as_uuid())
        .bind(user2.as_uuid())
        .fetch_optional(&self.pool)
        .await?;

        Ok(result)
    }

    async fn list_user_conversations(
        &self,
        user_id: UserId,
        page: i64,
        per_page: i64,
    ) -> AppResult<Vec<DmConversation>> {
        let offset = (page - 1) * per_page;
        let conversations = sqlx::query_as::<_, DmConversation>(
            r#"
            SELECT dc.id, dc.org_id, dc.name, dc.is_group, dc.created_by, dc.created_at, dc.updated_at, dc.deleted_at
            FROM dm_conversations dc
            INNER JOIN dm_participants dp ON dp.conversation_id = dc.id
            WHERE dp.user_id = $1 AND dc.deleted_at IS NULL
            ORDER BY dc.updated_at DESC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(user_id.as_uuid())
        .bind(per_page)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        Ok(conversations)
    }

    async fn add_participant(&self, participant: &DmParticipant) -> AppResult<DmParticipant> {
        let result = sqlx::query_as::<_, DmParticipant>(
            r#"
            INSERT INTO dm_participants (id, conversation_id, user_id, org_id)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (conversation_id, user_id) DO NOTHING
            RETURNING id, conversation_id, user_id, org_id, joined_at, last_read_at, created_at, updated_at
            "#,
        )
        .bind(participant.id)
        .bind(participant.conversation_id.as_uuid())
        .bind(participant.user_id.as_uuid())
        .bind(participant.org_id.as_uuid())
        .fetch_optional(&self.pool)
        .await?;

        match result {
            Some(p) => Ok(p),
            None => {
                // Already exists, fetch it
                self.get_participant(participant.conversation_id, participant.user_id)
                    .await
            }
        }
    }

    async fn remove_participant(
        &self,
        conversation_id: DmConversationId,
        user_id: UserId,
    ) -> AppResult<()> {
        let result = sqlx::query(
            r#"
            DELETE FROM dm_participants WHERE conversation_id = $1 AND user_id = $2
            "#,
        )
        .bind(conversation_id.as_uuid())
        .bind(user_id.as_uuid())
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound {
                resource: "DmParticipant".to_string(),
                id: format!("conversation={}, user={}", conversation_id, user_id),
            });
        }

        Ok(())
    }

    async fn get_participants(
        &self,
        conversation_id: DmConversationId,
    ) -> AppResult<Vec<DmParticipant>> {
        let participants = sqlx::query_as::<_, DmParticipant>(
            r#"
            SELECT id, conversation_id, user_id, org_id, joined_at, last_read_at, created_at, updated_at
            FROM dm_participants
            WHERE conversation_id = $1
            "#,
        )
        .bind(conversation_id.as_uuid())
        .fetch_all(&self.pool)
        .await?;

        Ok(participants)
    }

    async fn update_last_read(
        &self,
        conversation_id: DmConversationId,
        user_id: UserId,
    ) -> AppResult<()> {
        sqlx::query(
            r#"
            UPDATE dm_participants SET last_read_at = NOW(), updated_at = NOW()
            WHERE conversation_id = $1 AND user_id = $2
            "#,
        )
        .bind(conversation_id.as_uuid())
        .bind(user_id.as_uuid())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn create_message(&self, message: &Message) -> AppResult<Message> {
        let result = sqlx::query_as::<_, Message>(
            r#"
            INSERT INTO messages (id, channel_id, user_id, org_id, content, message_type, parent_id)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id, channel_id, user_id, content, message_type, parent_id, edited_at, created_at, updated_at, deleted_at, deleted_by
            "#,
        )
        .bind(message.id.as_uuid())
        .bind(message.channel_id.as_uuid())
        .bind(message.user_id.as_uuid())
        .bind(Uuid::nil()) // org_id - derived from conversation
        .bind(&message.content)
        .bind(message.message_type as domain::types::message::MessageType)
        .bind(message.parent_id.map(|id| id.as_uuid()))
        .fetch_one(&self.pool)
        .await?;

        Ok(result)
    }

    async fn list_conversation_messages(
        &self,
        conversation_id: DmConversationId,
        cursor: Option<MessageId>,
        limit: i64,
    ) -> AppResult<Vec<Message>> {
        // For DMs, we use the conversation_id as the channel_id in the messages table
        // since DM conversations are stored as channels with type 'direct_message'
        let result = if let Some(cursor_id) = cursor {
            sqlx::query_as::<_, Message>(
                r#"
                SELECT id, channel_id, user_id, content, message_type, parent_id, edited_at, created_at, updated_at, deleted_at, deleted_by
                FROM messages
                WHERE channel_id = $1 AND deleted_at IS NULL AND id < $2
                ORDER BY id DESC
                LIMIT $3
                "#,
            )
            .bind(conversation_id.as_uuid())
            .bind(cursor_id.as_uuid())
            .bind(limit)
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query_as::<_, Message>(
                r#"
                SELECT id, channel_id, user_id, content, message_type, parent_id, edited_at, created_at, updated_at, deleted_at, deleted_by
                FROM messages
                WHERE channel_id = $1 AND deleted_at IS NULL
                ORDER BY id DESC
                LIMIT $2
                "#,
            )
            .bind(conversation_id.as_uuid())
            .bind(limit)
            .fetch_all(&self.pool)
            .await?
        };

        Ok(result)
    }
}

impl SqlxDmRepository {
    /// Internal helper to fetch a specific participant.
    async fn get_participant(
        &self,
        conversation_id: DmConversationId,
        user_id: UserId,
    ) -> AppResult<DmParticipant> {
        let participant = sqlx::query_as::<_, DmParticipant>(
            r#"
            SELECT id, conversation_id, user_id, org_id, joined_at, last_read_at, created_at, updated_at
            FROM dm_participants
            WHERE conversation_id = $1 AND user_id = $2
            "#,
        )
        .bind(conversation_id.as_uuid())
        .bind(user_id.as_uuid())
        .fetch_one(&self.pool)
        .await?;

        Ok(participant)
    }
}
