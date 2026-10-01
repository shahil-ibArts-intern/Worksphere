use sqlx::{PgPool, Row};
use tracing::instrument;
use uuid::Uuid;

use domain::error::AppResult;
use domain::ids::{NotificationId, UserId};
use domain::types::notification::{Notification, NotificationType};

/// Service for notification-related business logic.
/// NOTE: Uses runtime queries instead of compile-time checked queries.
/// TODO: Convert to compile-time checked queries when database is available.
pub struct NotificationService {
    pool: PgPool,
}

impl NotificationService {
    /// Creates a new notification service.
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Creates a new notification for a user.
    #[instrument(skip(self))]
    pub async fn create_notification(
        &self,
        user_id: UserId,
        notification_type: NotificationType,
        title: &str,
        body: &str,
        link: Option<&str>,
    ) -> AppResult<Notification> {
        let notification_id = NotificationId::new();

        let notification = sqlx::query_as::<_, Notification>(
            r#"
            INSERT INTO notifications (id, user_id, org_id, notification_type, title, body, link)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id, user_id, notification_type, title, body, link, is_read, read_at, created_at
            "#,
        )
        .bind(notification_id.as_uuid())
        .bind(user_id.as_uuid())
        .bind(Uuid::nil())
        .bind(notification_type as domain::types::notification::NotificationType)
        .bind(title)
        .bind(body)
        .bind(link)
        .fetch_one(&self.pool)
        .await?;

        Ok(notification)
    }

    /// Gets unread notification count for a user.
    #[instrument(skip(self))]
    pub async fn get_unread_count(&self, user_id: UserId) -> AppResult<i64> {
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
}
