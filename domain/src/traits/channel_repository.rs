use async_trait::async_trait;

use crate::error::AppResult;
use crate::ids::{ChannelId, OrgId, UserId};
use crate::types::channel::{Channel, ChannelMember};

/// Repository trait for channel data access.
#[async_trait]
pub trait ChannelRepository: Send + Sync {
    /// Creates a new channel.
    async fn create(&self, channel: &Channel) -> AppResult<Channel>;

    /// Finds a channel by ID.
    async fn find_by_id(&self, id: ChannelId, org_id: OrgId) -> AppResult<Option<Channel>>;

    /// Updates a channel.
    async fn update(&self, channel: &Channel) -> AppResult<Channel>;

    /// Soft-deletes a channel.
    async fn soft_delete(&self, id: ChannelId, org_id: OrgId) -> AppResult<()>;

    /// Lists channels in an organization.
    async fn list_by_org(&self, org_id: OrgId, page: i64, per_page: i64)
        -> AppResult<Vec<Channel>>;

    /// Lists channels a user is a member of.
    async fn list_by_user(
        &self,
        org_id: OrgId,
        user_id: UserId,
        page: i64,
        per_page: i64,
    ) -> AppResult<Vec<Channel>>;

    /// Adds a member to a channel.
    async fn add_member(&self, member: &ChannelMember) -> AppResult<ChannelMember>;

    /// Removes a member from a channel.
    async fn remove_member(&self, channel_id: ChannelId, user_id: UserId) -> AppResult<()>;

    /// Lists members of a channel.
    async fn list_members(&self, channel_id: ChannelId) -> AppResult<Vec<ChannelMember>>;

    /// Checks if a user is a member of a channel.
    async fn is_member(&self, channel_id: ChannelId, user_id: UserId) -> AppResult<bool>;

    /// Updates last read timestamp for a channel member.
    async fn update_last_read(
        &self,
        channel_id: ChannelId,
        user_id: UserId,
        last_read_at: chrono::DateTime<chrono::Utc>,
    ) -> AppResult<()>;

    /// Searches channels by name in an organization.
    async fn search_by_name(
        &self,
        org_id: OrgId,
        query: &str,
        page: i64,
        per_page: i64,
    ) -> AppResult<Vec<Channel>>;
}
