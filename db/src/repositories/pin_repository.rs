use async_trait::async_trait;
use sqlx::{PgPool, Row};

use domain::error::{AppError, AppResult};
use domain::ids::{ChannelId, MessageId};
use domain::traits::PinRepository;
use domain::types::message::Pin;

/// SQLx implementation of the PinRepository trait.
#[derive(Clone)]
pub struct SqlxPinRepository {
    pool: PgPool,
}

impl SqlxPinRepository {
    /// Creates a new repository instance.
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl PinRepository for SqlxPinRepository {
    async fn add_pin(&self, pin: &Pin) -> AppResult<Pin> {
        let result = sqlx::query_as::<_, Pin>(
            r#"
            INSERT INTO pins (id, channel_id, message_id, pinned_by)
            VALUES ($1, $2, $3, $4)
            RETURNING id, channel_id, message_id, pinned_by, created_at
            "#,
        )
        .bind(pin.id.as_uuid())
        .bind(pin.channel_id.as_uuid())
        .bind(pin.message_id.as_uuid())
        .bind(pin.pinned_by.as_uuid())
        .fetch_one(&self.pool)
        .await?;

        Ok(result)
    }

    async fn remove_pin(&self, channel_id: ChannelId, message_id: MessageId) -> AppResult<()> {
        let result = sqlx::query(
            r#"
            DELETE FROM pins WHERE channel_id = $1 AND message_id = $2
            "#,
        )
        .bind(channel_id.as_uuid())
        .bind(message_id.as_uuid())
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound {
                resource: "Pin".to_string(),
                id: format!("channel={}, message={}", channel_id, message_id),
            });
        }

        Ok(())
    }

    async fn get_pins_by_channel(&self, channel_id: ChannelId) -> AppResult<Vec<Pin>> {
        let pins = sqlx::query_as::<_, Pin>(
            r#"
            SELECT id, channel_id, message_id, pinned_by, created_at
            FROM pins
            WHERE channel_id = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(channel_id.as_uuid())
        .fetch_all(&self.pool)
        .await?;

        Ok(pins)
    }

    async fn is_pinned(&self, channel_id: ChannelId, message_id: MessageId) -> AppResult<bool> {
        let result = sqlx::query(
            r#"
            SELECT 1 FROM pins WHERE channel_id = $1 AND message_id = $2
            "#,
        )
        .bind(channel_id.as_uuid())
        .bind(message_id.as_uuid())
        .fetch_optional(&self.pool)
        .await?;

        Ok(result.is_some())
    }

    async fn count_pins_in_channel(&self, channel_id: ChannelId) -> AppResult<i64> {
        let result = sqlx::query(
            r#"
            SELECT COUNT(*) as count FROM pins WHERE channel_id = $1
            "#,
        )
        .bind(channel_id.as_uuid())
        .fetch_one(&self.pool)
        .await?;

        let count: i64 = result.try_get("count").unwrap_or(0);
        Ok(count)
    }
}
