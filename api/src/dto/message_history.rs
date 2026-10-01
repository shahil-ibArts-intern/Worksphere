use crate::dto::user::PaginationMeta;
use serde::Serialize;

/// Message history entry response.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MessageHistoryResponse {
    pub id: String,
    pub message_id: String,
    pub user_id: String,
    pub history_type: String,
    pub previous_content: Option<String>,
    pub new_content: Option<String>,
    pub created_at: String,
}

/// Message history list response.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MessageHistoryListResponse {
    pub data: Vec<MessageHistoryResponse>,
    pub meta: PaginationMeta,
    pub total_count: i64,
}
