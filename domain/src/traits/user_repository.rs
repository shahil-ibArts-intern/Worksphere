use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::error::AppResult;
use crate::ids::UserId;
use crate::types::user::{User, UserSettings, UserStatus};

/// Repository trait for user data access.
#[async_trait]
pub trait UserRepository: Send + Sync {
    /// Creates a new user.
    async fn create(&self, user: &User) -> AppResult<User>;

    /// Finds a user by ID.
    async fn find_by_id(&self, id: UserId) -> AppResult<Option<User>>;

    /// Finds a user by email.
    async fn find_by_email(&self, email: &str) -> AppResult<Option<User>>;

    /// Finds a user by username.
    async fn find_by_username(&self, username: &str) -> AppResult<Option<User>>;

    /// Updates a user's profile.
    async fn update(&self, user: &User) -> AppResult<User>;

    /// Updates a user's status.
    async fn update_status(&self, id: UserId, status: UserStatus) -> AppResult<()>;

    /// Updates a user's last active timestamp.
    async fn update_last_active(&self, id: UserId, last_active: DateTime<Utc>) -> AppResult<()>;

    /// Lists users in an organization.
    async fn list_by_org(&self, org_id: Uuid, page: i64, per_page: i64) -> AppResult<Vec<User>>;

    /// Counts users in an organization.
    async fn count_by_org(&self, org_id: Uuid) -> AppResult<i64>;

    /// Searches users by name or email within an org.
    async fn search(
        &self,
        org_id: Uuid,
        query: &str,
        page: i64,
        per_page: i64,
    ) -> AppResult<Vec<User>>;

    /// Gets user settings.
    async fn get_settings(&self, user_id: UserId) -> AppResult<Option<UserSettings>>;

    /// Updates user settings.
    async fn update_settings(&self, settings: &UserSettings) -> AppResult<UserSettings>;

    /// Soft-deletes a user.
    async fn soft_delete(&self, id: UserId, deleted_by: UserId) -> AppResult<()>;
}
