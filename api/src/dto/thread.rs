use crate::dto::message::MessageResponse;
use crate::dto::user::PaginationMeta;
use serde::{Deserialize, Serialize};
use validator::Validate;

/// Request to reply in a thread.
#[derive(Debug, Clone, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct ThreadReplyRequest {
    #[validate(length(min = 1, max = 4000, message = "Content must be 1-4000 characters"))]
    pub content: String,

    /// Optional client-generated message ID for idempotency (UUID v7).
    pub id: Option<String>,
}

/// Thread reply list response.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ThreadReplyListResponse {
    pub data: Vec<MessageResponse>,
    pub meta: PaginationMeta,
    pub reply_count: i64,
}
