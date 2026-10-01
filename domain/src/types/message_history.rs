//! Message history types for edit/delete audit trail.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::ids::{MessageHistoryId, MessageId, UserId};

/// Type of history entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "message_history_type", rename_all = "snake_case")]
pub enum MessageHistoryType {
    Edit,
    Delete,
}

/// Represents a message history entry (edit or deletion).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct MessageHistory {
    pub id: MessageHistoryId,
    pub message_id: MessageId,
    pub user_id: UserId,
    pub history_type: MessageHistoryType,
    pub previous_content: Option<String>,
    pub new_content: Option<String>,
    pub created_at: DateTime<Utc>,
}
