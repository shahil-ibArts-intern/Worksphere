//! Presence DTOs — request and response types for presence operations.

use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::dto::user::PaginationMeta;

/// Request to update presence status.
#[derive(Debug, Clone, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct UpdatePresenceRequest {
    #[validate(length(min = 1, max = 20, message = "Status must be 1-20 characters"))]
    pub status: String,

    #[validate(length(max = 100, message = "Custom status must be at most 100 characters"))]
    pub custom_status: Option<String>,
}

/// Request to set a custom status message.
#[derive(Debug, Clone, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct UpdateStatusRequest {
    #[validate(length(max = 100, message = "Custom status must be at most 100 characters"))]
    pub custom_status: Option<String>,
}

/// Presence response for a single user.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PresenceResponse {
    pub user_id: String,
    pub status: String,
    pub custom_status: Option<String>,
    pub is_online: bool,
}

/// Status response for a single user.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StatusResponse {
    pub user_id: String,
    pub custom_status: Option<String>,
}

/// Paginated presence list.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PresenceListResponse {
    pub data: Vec<PresenceResponse>,
    pub meta: PaginationMeta,
}
