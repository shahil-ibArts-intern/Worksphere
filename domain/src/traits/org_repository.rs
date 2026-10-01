use async_trait::async_trait;
use uuid::Uuid;

use crate::error::AppResult;
use crate::ids::{OrgId, UserId};
use crate::types::org::{Invitation, OrgMember, OrgRole, Organization};

/// Repository trait for organization data access.
#[async_trait]
pub trait OrgRepository: Send + Sync {
    /// Creates a new organization.
    async fn create(&self, org: &Organization) -> AppResult<Organization>;

    /// Finds an organization by ID.
    async fn find_by_id(&self, id: OrgId) -> AppResult<Option<Organization>>;

    /// Finds an organization by slug.
    async fn find_by_slug(&self, slug: &str) -> AppResult<Option<Organization>>;

    /// Updates an organization.
    async fn update(&self, org: &Organization) -> AppResult<Organization>;

    /// Soft-deletes an organization.
    async fn soft_delete(&self, id: OrgId) -> AppResult<()>;

    /// Lists all organizations for a user.
    async fn list_by_user(&self, user_id: UserId) -> AppResult<Vec<Organization>>;

    /// Adds a member to an organization.
    async fn add_member(&self, member: &OrgMember) -> AppResult<OrgMember>;

    /// Finds an org member by org and user ID.
    async fn find_member(&self, org_id: OrgId, user_id: UserId) -> AppResult<Option<OrgMember>>;

    /// Updates a member's role.
    async fn update_member_role(
        &self,
        org_id: OrgId,
        user_id: UserId,
        role: OrgRole,
    ) -> AppResult<()>;

    /// Removes a member from an organization.
    async fn remove_member(&self, org_id: OrgId, user_id: UserId) -> AppResult<()>;

    /// Lists all members of an organization.
    async fn list_members(&self, org_id: OrgId) -> AppResult<Vec<OrgMember>>;

    /// Counts members in an organization.
    async fn count_members(&self, org_id: OrgId) -> AppResult<i64>;

    /// Creates an invitation.
    async fn create_invitation(&self, invitation: &Invitation) -> AppResult<Invitation>;

    /// Finds an invitation by token.
    async fn find_invitation_by_token(&self, token: &str) -> AppResult<Option<Invitation>>;

    /// Accepts an invitation.
    async fn accept_invitation(&self, invitation_id: Uuid) -> AppResult<()>;

    /// Lists invitations for an organization.
    async fn list_invitations(&self, org_id: OrgId) -> AppResult<Vec<Invitation>>;
}
