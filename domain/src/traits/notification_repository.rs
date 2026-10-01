use async_trait::async_trait;

use crate::error::AppResult;
use crate::ids::{NotificationId, UserId};
use crate::types::notification::{Notification, NotificationPreferences};

/// Repository trait for notification data access.
#[async_trait]
pub trait NotificationRepository: Send + Sync {
    /// Creates a new notification.
    async fn create(&self, notification: &Notification) -> AppResult<Notification>;

    /// Finds a notification by ID.
    async fn find_by_id(&self, id: NotificationId) -> AppResult<Option<Notification>>;

    /// Lists notifications for a user.
    async fn list_by_user(
        &self,
        user_id: UserId,
        page: i64,
        per_page: i64,
    ) -> AppResult<Vec<Notification>>;

    /// Marks a notification as read.
    async fn mark_as_read(&self, id: NotificationId) -> AppResult<()>;

    /// Marks all notifications as read for a user.
    async fn mark_all_as_read(&self, user_id: UserId) -> AppResult<()>;

    /// Deletes a notification.
    async fn delete(&self, id: NotificationId) -> AppResult<()>;

    /// Counts unread notifications for a user.
    async fn count_unread(&self, user_id: UserId) -> AppResult<i64>;

    /// Gets notification preferences for a user.
    async fn get_preferences(&self, user_id: UserId) -> AppResult<Option<NotificationPreferences>>;

    /// Updates notification preferences.
    async fn update_preferences(
        &self,
        preferences: &NotificationPreferences,
    ) -> AppResult<NotificationPreferences>;
}
