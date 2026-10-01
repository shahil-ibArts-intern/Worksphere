use crate::dto::user::PaginationMeta;
use serde::{Deserialize, Serialize};

use validator::Validate;

/// Request to create an organization.
#[derive(Debug, Clone, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct CreateOrgRequest {
    #[validate(length(min = 1, max = 100, message = "Name must be 1-100 characters"))]
    pub name: String,

    #[validate(length(min = 1, max = 100, message = "Slug must be 1-100 characters"))]
    pub slug: String,

    #[validate(length(max = 500, message = "Description must be at most 500 characters"))]
    pub description: Option<String>,
}

/// Request to update an organization.
#[derive(Debug, Clone, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct UpdateOrgRequest {
    #[validate(length(min = 1, max = 100, message = "Name must be 1-100 characters"))]
    pub name: Option<String>,

    #[validate(length(max = 500, message = "Description must be at most 500 characters"))]
    pub description: Option<String>,

    pub logo_url: Option<String>,
}

/// Request to invite a user.
#[derive(Debug, Clone, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct InviteUserRequest {
    #[validate(email(message = "Invalid email address"))]
    pub email: String,

    pub role: Option<String>,
}

/// Request to change a member's role.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChangeRoleRequest {
    pub role: String,
}

/// Organization response.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OrgResponse {
    pub id: String,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub logo_url: Option<String>,
    pub plan: String,
    pub max_members: i32,
    pub created_at: String,
}

/// Organization member response.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OrgMemberResponse {
    pub id: String,
    pub user_id: String,
    pub role: String,
    pub joined_at: String,
}

/// Paginated org member list.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OrgMemberListResponse {
    pub data: Vec<OrgMemberResponse>,
    pub meta: PaginationMeta,
}

/// Invitation response.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InvitationResponse {
    pub id: String,
    pub email: String,
    pub role: String,
    pub expires_at: String,
    pub created_at: String,
}

/// Organization settings response.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OrgSettingsResponse {
    pub allow_public_channels: bool,
    pub allow_guests: bool,
    pub default_channel_visibility: String,
    pub retention_days: i32,
}
