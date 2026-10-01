use async_trait::async_trait;
use sqlx::{PgPool, Row};
use uuid::Uuid;

use domain::error::{AppError, AppResult};
use domain::ids::{NotificationId, UserId};
use domain::traits::NotificationRepository;
use domain::types::notification::{Notification, NotificationPreferences};

/// SQLx implementation of the NotificationRepository trait.
/// NOTE: Uses runtime queries instead of compile-time checked queries.
/// TODO: Convert to compile-time checked queries when database is available.
#[derive(Clone)]
pub struct SqlxNotificationRepository {
    pool: PgPool,
}

impl SqlxNotificationRepository {
    /// Creates a new repository instance.
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl NotificationRepository for SqlxNotificationRepository {
    async fn create(&self, notification: &Notification) -> AppResult<Notification> {
        let result = sqlx::query_as::<_, Notification>(
            r#"
            INSERT INTO notifications (id, user_id, org_id, notification_type, title, body, link)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id, user_id, notification_type, title, body, link, is_read, read_at, created_at
            "#,
        )
        .bind(notification.id.as_uuid())
        .bind(notification.user_id.as_uuid())
        .bind(Uuid::nil()) // org_id - should be derived from context
        .bind(notification.notification_type as domain::types::notification::NotificationType)
        .bind(&notification.title)
        .bind(&notification.body)
        .bind(&notification.link)
        .fetch_one(&self.pool)
        .await?;

        Ok(result)
    }

    async fn find_by_id(&self, id: NotificationId) -> AppResult<Option<Notification>> {
        let result = sqlx::query_as::<_, Notification>(
            r#"
            SELECT id, user_id, notification_type, title, body, link, is_read, read_at, created_at
            FROM notifications
            WHERE id = $1
            "#,
        )
        .bind(id.as_uuid())
        .fetch_optional(&self.pool)
        .await?;

        Ok(result)
    }

    async fn list_by_user(
        &self,
        user_id: UserId,
        page: i64,
        per_page: i64,
    ) -> AppResult<Vec<Notification>> {
        let offset = (page - 1) * per_page;
        let result = sqlx::query_as::<_, Notification>(
            r#"
            SELECT id, user_id, notification_type, title, body, link, is_read, read_at, created_at
            FROM notifications
            WHERE user_id = $1
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(user_id.as_uuid())
        .bind(per_page)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        Ok(result)
    }

    async fn mark_as_read(&self, id: NotificationId) -> AppResult<()> {
        let result = sqlx::query(
            r#"
            UPDATE notifications SET is_read = TRUE, read_at = NOW() WHERE id = $1
            "#,
        )
        .bind(id.as_uuid())
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound {
                resource: "Notification".to_string(),
                id: id.to_string(),
            });
        }

        Ok(())
    }

    async fn mark_all_as_read(&self, user_id: UserId) -> AppResult<()> {
        sqlx::query(
            r#"
            UPDATE notifications SET is_read = TRUE, read_at = NOW() WHERE user_id = $1 AND is_read = FALSE
            "#,
        )
        .bind(user_id.as_uuid())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn delete(&self, id: NotificationId) -> AppResult<()> {
        let result = sqlx::query(
            r#"
            DELETE FROM notifications WHERE id = $1
            "#,
        )
        .bind(id.as_uuid())
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound {
                resource: "Notification".to_string(),
                id: id.to_string(),
            });
        }

        Ok(())
    }

    async fn count_unread(&self, user_id: UserId) -> AppResult<i64> {
        let result = sqlx::query(
            r#"
            SELECT COUNT(*) as count FROM notifications WHERE user_id = $1 AND is_read = FALSE
            "#,
        )
        .bind(user_id.as_uuid())
        .fetch_one(&self.pool)
        .await?;

        let count: i64 = result.try_get("count").unwrap_or(0);
        Ok(count)
    }

    async fn get_preferences(
        &self,
        _user_id: UserId,
    ) -> AppResult<Option<NotificationPreferences>> {
        // Notification preferences table not in initial migration
        Ok(None)
    }

    async fn update_preferences(
        &self,
        _preferences: &NotificationPreferences,
    ) -> AppResult<NotificationPreferences> {
        Err(AppError::Internal(
            "Notification preferences not yet implemented".to_string(),
        ))
    }
}
