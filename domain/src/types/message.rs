use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::ids::{ChannelId, MessageId, PinId, ReactionId, UserId};

/// Represents a message in a channel.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Message {
    pub id: MessageId,
    pub channel_id: ChannelId,
    pub user_id: UserId,
    pub content: String,
    pub message_type: MessageType,
    pub parent_id: Option<MessageId>,
    pub edited_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub deleted_by: Option<UserId>,
}

/// Type of message.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[serde(rename_all = "camelCase")]
#[sqlx(type_name = "message_type", rename_all = "snake_case")]
pub enum MessageType {
    Text,
    System,
    File,
    ThreadReply,
}

/// Represents an emoji reaction on a message.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Reaction {
    pub id: ReactionId,
    pub message_id: MessageId,
    pub user_id: UserId,
    pub emoji: String,
    pub created_at: DateTime<Utc>,
}

/// Represents a pinned message in a channel.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Pin {
    pub id: PinId,
    pub channel_id: ChannelId,
    pub message_id: MessageId,
    pub pinned_by: UserId,
    pub created_at: DateTime<Utc>,
}
