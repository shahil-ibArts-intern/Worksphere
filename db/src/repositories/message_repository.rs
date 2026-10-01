use async_trait::async_trait;
use sqlx::{PgPool, Row};
use uuid::Uuid;

use domain::error::{AppError, AppResult};
use domain::ids::{ChannelId, MessageId, UserId};
use domain::traits::MessageRepository;
use domain::types::message::{Message, MessageType};

/// SQLx implementation of the MessageRepository trait.
/// NOTE: Uses runtime queries instead of compile-time checked queries.
/// TODO: Convert to compile-time checked queries when database is available.
#[derive(Clone)]
pub struct SqlxMessageRepository {
    pool: PgPool,
}

impl SqlxMessageRepository {
    /// Creates a new repository instance.
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl MessageRepository for SqlxMessageRepository {
    async fn create(&self, message: &Message) -> AppResult<Message> {
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
        .bind(Uuid::nil()) // org_id - should be derived from channel
        .bind(&message.content)
        .bind(message.message_type as domain::types::message::MessageType)
        .bind(message.parent_id.map(|id| id.as_uuid()))
        .fetch_one(&self.pool)
        .await?;

        Ok(result)
    }

    async fn create_with_id(
        &self,
        channel_id: ChannelId,
        user_id: UserId,
        content: &str,
        parent_id: Option<MessageId>,
        message_id: Option<MessageId>,
    ) -> AppResult<Message> {
        let msg_id = message_id.unwrap_or_default();

        // Check for idempotency
        if let Some(existing) = self.find_by_id(msg_id).await? {
            return Ok(existing);
        }

        let result = sqlx::query_as::<_, Message>(
            r#"
            INSERT INTO messages (id, channel_id, user_id, org_id, content, message_type, parent_id)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id, channel_id, user_id, content, message_type, parent_id, edited_at, created_at, updated_at, deleted_at, deleted_by
            "#,
        )
        .bind(msg_id.as_uuid())
        .bind(channel_id.as_uuid())
        .bind(user_id.as_uuid())
        .bind(Uuid::nil()) // org_id - should be derived from channel
        .bind(content)
        .bind(MessageType::Text as domain::types::message::MessageType)
        .bind(parent_id.map(|id| id.as_uuid()))
        .fetch_one(&self.pool)
        .await?;

        Ok(result)
    }

    async fn find_by_id(&self, id: MessageId) -> AppResult<Option<Message>> {
        let result = sqlx::query_as::<_, Message>(
            r#"
            SELECT id, channel_id, user_id, content, message_type, parent_id, edited_at, created_at, updated_at, deleted_at, deleted_by
            FROM messages
            WHERE id = $1 AND deleted_at IS NULL
            "#,
        )
        .bind(id.as_uuid())
        .fetch_optional(&self.pool)
        .await?;

        Ok(result)
    }

    async fn update(&self, message: &Message) -> AppResult<Message> {
        let result = sqlx::query_as::<_, Message>(
            r#"
            UPDATE messages
            SET content = $2, edited_at = NOW()
            WHERE id = $1 AND deleted_at IS NULL
            RETURNING id, channel_id, user_id, content, message_type, parent_id, edited_at, created_at, updated_at, deleted_at, deleted_by
            "#,
        )
        .bind(message.id.as_uuid())
        .bind(&message.content)
        .fetch_one(&self.pool)
        .await?;

        Ok(result)
    }

    async fn soft_delete(&self, id: MessageId, deleted_by: UserId) -> AppResult<()> {
        let result = sqlx::query(
            r#"
            UPDATE messages SET deleted_at = NOW(), deleted_by = $2 WHERE id = $1 AND deleted_at IS NULL
            "#,
        )
        .bind(id.as_uuid())
        .bind(deleted_by.as_uuid())
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound {
                resource: "Message".to_string(),
                id: id.to_string(),
            });
        }

        Ok(())
    }

    async fn list_by_channel(
        &self,
        channel_id: ChannelId,
        cursor: Option<MessageId>,
        limit: i64,
    ) -> AppResult<Vec<Message>> {
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
            .bind(channel_id.as_uuid())
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
            .bind(channel_id.as_uuid())
            .bind(limit)
            .fetch_all(&self.pool)
            .await?
        };

        Ok(result)
    }

    async fn list_thread(
        &self,
        parent_id: MessageId,
        page: i64,
        per_page: i64,
    ) -> AppResult<Vec<Message>> {
        let offset = (page - 1) * per_page;
        let result = sqlx::query_as::<_, Message>(
            r#"
            SELECT id, channel_id, user_id, content, message_type, parent_id, edited_at, created_at, updated_at, deleted_at, deleted_by
            FROM messages
            WHERE parent_id = $1 AND deleted_at IS NULL
            ORDER BY created_at ASC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(parent_id.as_uuid())
        .bind(per_page)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        Ok(result)
    }

    async fn count_by_channel(&self, channel_id: ChannelId) -> AppResult<i64> {
        let result = sqlx::query(
            r#"
            SELECT COUNT(*) as count FROM messages WHERE channel_id = $1 AND deleted_at IS NULL
            "#,
        )
        .bind(channel_id.as_uuid())
        .fetch_one(&self.pool)
        .await?;

        let count: i64 = result.try_get("count").unwrap_or(0);
        Ok(count)
    }

    async fn search(
        &self,
        channel_id: ChannelId,
        query: &str,
        page: i64,
        per_page: i64,
    ) -> AppResult<Vec<Message>> {
        let offset = (page - 1) * per_page;
        let search_pattern = format!("%{}%", query);
        let result = sqlx::query_as::<_, Message>(
            r#"
            SELECT id, channel_id, user_id, content, message_type, parent_id, edited_at, created_at, updated_at, deleted_at, deleted_by
            FROM messages
            WHERE channel_id = $1 AND deleted_at IS NULL AND content ILIKE $2
            ORDER BY created_at DESC
            LIMIT $3 OFFSET $4
            "#,
        )
        .bind(channel_id.as_uuid())
        .bind(&search_pattern)
        .bind(per_page)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        Ok(result)
    }
}
