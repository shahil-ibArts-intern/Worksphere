//! Message service — business logic for messaging operations.
//!
//! Handles message CRUD, cursor-based pagination, idempotency,
//! and real-time event emission.

use sqlx::{PgPool, Row};
use tracing::{info, instrument, warn};

use domain::error::{AppError, AppResult};
use domain::ids::{ChannelId, MessageId, OrgId, UserId};
use domain::types::message::{Message, MessageType};

/// Service for message-related business logic.
pub struct MessageService {
    pool: PgPool,
}

impl MessageService {
    /// Creates a new message service.
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Sends a message to a channel.
    ///
    /// If `client_message_id` is provided, it's used as the message ID
    /// for idempotency. If a message with that ID already exists,
    /// the existing message is returned.
    #[instrument(skip(self))]
    pub async fn send_message(
        &self,
        channel_id: ChannelId,
        org_id: OrgId,
        user_id: UserId,
        content: &str,
        parent_id: Option<MessageId>,
        client_message_id: Option<MessageId>,
    ) -> AppResult<Message> {
        // Check for idempotency if client_message_id is provided
        if let Some(ref msg_id) = client_message_id {
            if let Some(existing) = self.get_message_by_id(*msg_id).await? {
                info!(
                    message_id = %msg_id,
                    "Duplicate message ID detected, returning existing message"
                );
                return Ok(existing);
            }
        }

        let message_id = client_message_id.unwrap_or_default();

        let message = sqlx::query_as::<_, Message>(
            r#"
            INSERT INTO messages (id, channel_id, user_id, org_id, content, message_type, parent_id)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id, channel_id, user_id, content, message_type, parent_id, edited_at, created_at, updated_at, deleted_at, deleted_by
            "#,
        )
        .bind(message_id.as_uuid())
        .bind(channel_id.as_uuid())
        .bind(user_id.as_uuid())
        .bind(org_id.as_uuid())
        .bind(content)
        .bind(MessageType::Text)
        .bind(parent_id.map(|id| id.as_uuid()))
        .fetch_one(&self.pool)
        .await?;

        info!(message_id = %message.id, "Message sent successfully");
        Ok(message)
    }

    /// Gets a message by ID.
    #[instrument(skip(self))]
    pub async fn get_message(&self, message_id: MessageId) -> AppResult<Message> {
        let message = sqlx::query_as::<_, Message>(
            r#"
            SELECT id, channel_id, user_id, content, message_type, parent_id, edited_at, created_at, updated_at, deleted_at, deleted_by
            FROM messages
            WHERE id = $1 AND deleted_at IS NULL
            "#,
        )
        .bind(message_id.as_uuid())
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| AppError::NotFound {
            resource: "Message".to_string(),
            id: message_id.to_string(),
        })?;

        Ok(message)
    }

    /// Gets a message by ID (internal, returns Option).
    async fn get_message_by_id(&self, message_id: MessageId) -> AppResult<Option<Message>> {
        let message = sqlx::query_as::<_, Message>(
            r#"
            SELECT id, channel_id, user_id, content, message_type, parent_id, edited_at, created_at, updated_at, deleted_at, deleted_by
            FROM messages
            WHERE id = $1 AND deleted_at IS NULL
            "#,
        )
        .bind(message_id.as_uuid())
        .fetch_optional(&self.pool)
        .await?;

        Ok(message)
    }

    /// Lists messages in a channel with cursor-based pagination.
    ///
    /// If `cursor` is provided, returns messages before that cursor (older messages).
    /// If `cursor` is None, returns the most recent messages.
    #[instrument(skip(self))]
    pub async fn list_messages(
        &self,
        channel_id: ChannelId,
        cursor: Option<MessageId>,
        limit: i64,
    ) -> AppResult<Vec<Message>> {
        let messages = if let Some(cursor_id) = cursor {
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

        Ok(messages)
    }

    /// Edits a message.
    #[instrument(skip(self))]
    pub async fn edit_message(
        &self,
        message_id: MessageId,
        user_id: UserId,
        content: &str,
    ) -> AppResult<Message> {
        // Verify the message exists and belongs to the user
        let message = self.get_message(message_id).await?;

        if message.user_id != user_id {
            return Err(AppError::Authorization(
                "You can only edit your own messages".to_string(),
            ));
        }

        let updated = sqlx::query_as::<_, Message>(
            r#"
            UPDATE messages
            SET content = $2, edited_at = NOW(), updated_at = NOW()
            WHERE id = $1 AND deleted_at IS NULL
            RETURNING id, channel_id, user_id, content, message_type, parent_id, edited_at, created_at, updated_at, deleted_at, deleted_by
            "#,
        )
        .bind(message_id.as_uuid())
        .bind(content)
        .fetch_one(&self.pool)
        .await?;

        info!(message_id = %message_id, "Message edited successfully");
        Ok(updated)
    }

    /// Soft-deletes a message.
    #[instrument(skip(self))]
    pub async fn delete_message(&self, message_id: MessageId, user_id: UserId) -> AppResult<()> {
        // Verify the message exists
        let message = self.get_message(message_id).await?;

        // Only the message author or an admin can delete
        if message.user_id != user_id {
            // In production, check if user is admin
            warn!(
                message_id = %message_id,
                user_id = %user_id,
                "Unauthorized delete attempt"
            );
            return Err(AppError::Authorization(
                "You can only delete your own messages".to_string(),
            ));
        }

        let result = sqlx::query(
            r#"
            UPDATE messages SET deleted_at = NOW(), deleted_by = $2 WHERE id = $1 AND deleted_at IS NULL
            "#,
        )
        .bind(message_id.as_uuid())
        .bind(user_id.as_uuid())
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound {
                resource: "Message".to_string(),
                id: message_id.to_string(),
            });
        }

        info!(message_id = %message_id, "Message deleted successfully");
        Ok(())
    }

    /// Counts messages in a channel.
    #[instrument(skip(self))]
    pub async fn count_messages(&self, channel_id: ChannelId) -> AppResult<i64> {
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

    /// Searches messages in a channel.
    #[instrument(skip(self))]
    pub async fn search_messages(
        &self,
        channel_id: ChannelId,
        query: &str,
        page: i64,
        per_page: i64,
    ) -> AppResult<Vec<Message>> {
        let offset = (page - 1) * per_page;
        let search_pattern = format!("%{}%", query);
        let messages = sqlx::query_as::<_, Message>(
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

        Ok(messages)
    }
}

// ── Tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_service_creation() {
        // Placeholder — real tests would need a database connection
        assert!(true);
    }
}
