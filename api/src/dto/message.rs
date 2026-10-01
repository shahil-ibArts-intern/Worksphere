use crate::dto::user::PaginationMeta;
use serde::{Deserialize, Serialize};

use validator::Validate;

/// Request to send a message.
#[derive(Debug, Clone, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct SendMessageRequest {
    /// Optional client-generated message ID for idempotency (UUID v7).
    /// If provided and a message with this ID already exists, the existing
    /// message is returned instead of creating a duplicate.
    pub id: Option<String>,

    #[validate(length(min = 1, max = 4000, message = "Content must be 1-4000 characters"))]
    pub content: String,

    pub parent_id: Option<String>,
}

/// Request to edit a message.
#[derive(Debug, Clone, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct EditMessageRequest {
    #[validate(length(min = 1, max = 4000, message = "Content must be 1-4000 characters"))]
    pub content: String,
}

/// Message response.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MessageResponse {
    pub id: String,
    pub channel_id: String,
    pub user_id: String,
    pub content: String,
    pub message_type: String,
    pub parent_id: Option<String>,
    pub edited_at: Option<String>,
    pub created_at: String,
}

/// Paginated message list.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MessageListResponse {
    pub data: Vec<MessageResponse>,
    pub meta: PaginationMeta,
    pub next_cursor: Option<String>,
}
