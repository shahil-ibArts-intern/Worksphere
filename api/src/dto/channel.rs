use crate::dto::user::PaginationMeta;
use serde::{Deserialize, Serialize};

use validator::Validate;

/// Request to create a channel.
#[derive(Debug, Clone, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct CreateChannelRequest {
    #[validate(length(min = 1, max = 80, message = "Name must be 1-80 characters"))]
    pub name: String,

    #[validate(length(max = 255, message = "Description must be at most 255 characters"))]
    pub description: Option<String>,

    pub is_private: bool,
}

/// Request to update a channel.
#[derive(Debug, Clone, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct UpdateChannelRequest {
    #[validate(length(min = 1, max = 80, message = "Name must be 1-80 characters"))]
    pub name: Option<String>,

    #[validate(length(max = 255, message = "Description must be at most 255 characters"))]
    pub description: Option<String>,

    pub is_private: Option<bool>,
}

/// Channel response.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChannelResponse {
    pub id: String,
    pub org_id: String,
    pub name: String,
    pub description: Option<String>,
    pub channel_type: String,
    pub is_private: bool,
    pub created_by: String,
    pub created_at: String,
    pub updated_at: String,
}

/// Paginated channel list.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChannelListResponse {
    pub data: Vec<ChannelResponse>,
    pub meta: PaginationMeta,
}

/// Channel member response.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChannelMemberResponse {
    pub id: String,
    pub user_id: String,
    pub is_admin: bool,
    pub joined_at: String,
}

/// Paginated channel member list.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChannelMemberListResponse {
    pub data: Vec<ChannelMemberResponse>,
    pub meta: PaginationMeta,
}
