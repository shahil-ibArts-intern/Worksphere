use async_trait::async_trait;
use sqlx::{PgPool, Row};
use uuid::Uuid;

use domain::error::{AppError, AppResult};
use domain::ids::{MessageId, UserId};
use domain::traits::ReactionRepository;
use domain::types::message::Reaction;

/// SQLx implementation of the ReactionRepository trait.
#[derive(Clone)]
pub struct SqlxReactionRepository {
    pool: PgPool,
}

impl SqlxReactionRepository {
    /// Creates a new repository instance.
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ReactionRepository for SqlxReactionRepository {
    async fn add_reaction(&self, reaction: &Reaction) -> AppResult<Reaction> {
        let result = sqlx::query_as::<_, Reaction>(
            r#"
            INSERT INTO reactions (id, message_id, user_id, org_id, emoji)
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (message_id, user_id, emoji) DO NOTHING
            RETURNING id, message_id, user_id, emoji, created_at
            "#,
        )
        .bind(reaction.id.as_uuid())
        .bind(reaction.message_id.as_uuid())
        .bind(reaction.user_id.as_uuid())
        .bind(Uuid::nil()) // org_id - will be set by trigger or derived
        .bind(&reaction.emoji)
        .fetch_optional(&self.pool)
        .await?;

        match result {
            Some(r) => Ok(r),
            None => {
                // Reaction already exists, fetch it
                self.get_reaction(reaction.message_id, reaction.user_id, &reaction.emoji)
                    .await
            }
        }
    }

    async fn remove_reaction(
        &self,
        message_id: MessageId,
        user_id: UserId,
        emoji: &str,
    ) -> AppResult<()> {
        let result = sqlx::query(
            r#"
            DELETE FROM reactions WHERE message_id = $1 AND user_id = $2 AND emoji = $3
            "#,
        )
        .bind(message_id.as_uuid())
        .bind(user_id.as_uuid())
        .bind(emoji)
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound {
                resource: "Reaction".to_string(),
                id: format!("message={}, user={}, emoji={}", message_id, user_id, emoji),
            });
        }

        Ok(())
    }

    async fn get_reactions_by_message(&self, message_id: MessageId) -> AppResult<Vec<Reaction>> {
        let reactions = sqlx::query_as::<_, Reaction>(
            r#"
            SELECT id, message_id, user_id, emoji, created_at
            FROM reactions
            WHERE message_id = $1
            ORDER BY created_at ASC
            "#,
        )
        .bind(message_id.as_uuid())
        .fetch_all(&self.pool)
        .await?;

        Ok(reactions)
    }

    async fn get_reactions_by_user(&self, user_id: UserId) -> AppResult<Vec<Reaction>> {
        let reactions = sqlx::query_as::<_, Reaction>(
            r#"
            SELECT id, message_id, user_id, emoji, created_at
            FROM reactions
            WHERE user_id = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(user_id.as_uuid())
        .fetch_all(&self.pool)
        .await?;

        Ok(reactions)
    }

    async fn count_reactions_by_message(
        &self,
        message_id: MessageId,
    ) -> AppResult<Vec<(String, i64)>> {
        let rows = sqlx::query(
            r#"
            SELECT emoji, COUNT(*) as count
            FROM reactions
            WHERE message_id = $1
            GROUP BY emoji
            ORDER BY count DESC, emoji ASC
            "#,
        )
        .bind(message_id.as_uuid())
        .fetch_all(&self.pool)
        .await?;

        let result = rows
            .iter()
            .map(|row| {
                let emoji: String = row.try_get("emoji").unwrap_or_default();
                let count: i64 = row.try_get("count").unwrap_or(0);
                (emoji, count)
            })
            .collect();

        Ok(result)
    }

    async fn has_user_reacted(
        &self,
        message_id: MessageId,
        user_id: UserId,
        emoji: &str,
    ) -> AppResult<bool> {
        let result = sqlx::query(
            r#"
            SELECT 1 FROM reactions WHERE message_id = $1 AND user_id = $2 AND emoji = $3
            "#,
        )
        .bind(message_id.as_uuid())
        .bind(user_id.as_uuid())
        .bind(emoji)
        .fetch_optional(&self.pool)
        .await?;

        Ok(result.is_some())
    }
}

impl SqlxReactionRepository {
    /// Internal helper to fetch a specific reaction.
    async fn get_reaction(
        &self,
        message_id: MessageId,
        user_id: UserId,
        emoji: &str,
    ) -> AppResult<Reaction> {
        let reaction = sqlx::query_as::<_, Reaction>(
            r#"
            SELECT id, message_id, user_id, emoji, created_at
            FROM reactions
            WHERE message_id = $1 AND user_id = $2 AND emoji = $3
            "#,
        )
        .bind(message_id.as_uuid())
        .bind(user_id.as_uuid())
        .bind(emoji)
        .fetch_one(&self.pool)
        .await?;

        Ok(reaction)
    }
}
