use crate::dto::message::MessageResponse;
use crate::dto::user::PaginationMeta;
use serde::{Deserialize, Serialize};
use validator::Validate;

/// Request to create a group DM.
#[derive(Debug, Clone, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct CreateGroupDmRequest {
    #[validate(length(min = 1, max = 100, message = "Name must be 1-100 characters"))]
    pub name: String,

    pub participant_ids: Vec<String>,
}

/// Request to send a DM message.
#[derive(Debug, Clone, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct SendDmMessageRequest {
    #[validate(length(min = 1, max = 4000, message = "Content must be 1-4000 characters"))]
    pub content: String,

    /// Optional client-generated message ID for idempotency (UUID v7).
    pub id: Option<String>,
}

/// DM conversation response.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DmConversationResponse {
    pub id: String,
    pub org_id: String,
    pub name: Option<String>,
    pub is_group: bool,
    pub created_by: String,
    pub created_at: String,
    pub updated_at: String,
    pub participant_count: usize,
    pub last_message: Option<MessageResponse>,
}

/// DM conversation list response.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DmConversationListResponse {
    pub data: Vec<DmConversationResponse>,
    pub meta: PaginationMeta,
}

/// DM participant response.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DmParticipantResponse {
    pub id: String,
    pub user_id: String,
    pub joined_at: String,
    pub last_read_at: Option<String>,
}
