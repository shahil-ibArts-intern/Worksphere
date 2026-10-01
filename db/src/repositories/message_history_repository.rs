use async_trait::async_trait;
use sqlx::{PgPool, Row};

use domain::error::AppResult;
use domain::ids::MessageId;
use domain::traits::MessageHistoryRepository;
use domain::types::message_history::MessageHistory;

/// SQLx implementation of the MessageHistoryRepository trait.
#[derive(Clone)]
pub struct SqlxMessageHistoryRepository {
    pool: PgPool,
}

impl SqlxMessageHistoryRepository {
    /// Creates a new repository instance.
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl MessageHistoryRepository for SqlxMessageHistoryRepository {
    async fn create(&self, history: &MessageHistory) -> AppResult<MessageHistory> {
        let result = sqlx::query_as::<_, MessageHistory>(
            r#"
            INSERT INTO message_history (id, message_id, user_id, history_type, previous_content, new_content)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id, message_id, user_id, history_type, previous_content, new_content, created_at
            "#,
        )
        .bind(history.id.as_uuid())
        .bind(history.message_id.as_uuid())
        .bind(history.user_id.as_uuid())
        .bind(history.history_type as domain::types::message_history::MessageHistoryType)
        .bind(&history.previous_content)
        .bind(&history.new_content)
        .fetch_one(&self.pool)
        .await?;

        Ok(result)
    }

    async fn list_by_message(
        &self,
        message_id: MessageId,
        page: i64,
        per_page: i64,
    ) -> AppResult<Vec<MessageHistory>> {
        let offset = (page - 1) * per_page;
        let history = sqlx::query_as::<_, MessageHistory>(
            r#"
            SELECT id, message_id, user_id, history_type, previous_content, new_content, created_at
            FROM message_history
            WHERE message_id = $1
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(message_id.as_uuid())
        .bind(per_page)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        Ok(history)
    }

    async fn count_by_message(&self, message_id: MessageId) -> AppResult<i64> {
        let result = sqlx::query(
            r#"
            SELECT COUNT(*) as count FROM message_history WHERE message_id = $1
            "#,
        )
        .bind(message_id.as_uuid())
        .fetch_one(&self.pool)
        .await?;

        let count: i64 = result.try_get("count").unwrap_or(0);
        Ok(count)
    }
}
