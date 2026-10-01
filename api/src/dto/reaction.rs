use serde::{Deserialize, Serialize};
use validator::Validate;

/// Request to add a reaction.
#[derive(Debug, Clone, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct AddReactionRequest {
    #[validate(length(min = 1, max = 8, message = "Emoji must be 1-8 characters"))]
    pub emoji: String,
}

/// Reaction response.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReactionResponse {
    pub id: String,
    pub message_id: String,
    pub user_id: String,
    pub emoji: String,
    pub created_at: String,
}

/// Reaction count with users.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReactionCountResponse {
    pub emoji: String,
    pub count: i64,
    pub users: Vec<String>,
}

/// List of reactions on a message.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReactionListResponse {
    pub data: Vec<ReactionCountResponse>,
}
