use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::ids::{NotificationId, UserId};

/// Represents a notification for a user.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Notification {
    pub id: NotificationId,
    pub user_id: UserId,
    pub notification_type: NotificationType,
    pub title: String,
    pub body: String,
    pub link: Option<String>,
    pub is_read: bool,
    pub read_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

/// Type of notification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[serde(rename_all = "camelCase")]
#[sqlx(type_name = "notification_type", rename_all = "snake_case")]
pub enum NotificationType {
    Mention,
    DirectMessage,
    TaskAssigned,
    TaskDueSoon,
    ReactionAdded,
    ChannelInvite,
    InvitationAccepted,
    System,
}

/// Notification preferences for a user.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NotificationPreferences {
    pub user_id: UserId,
    pub mentions: bool,
    pub direct_messages: bool,
    pub task_assignments: bool,
    pub task_due_reminders: bool,
    pub reactions: bool,
    pub channel_invites: bool,
    pub email_digest: bool,
    pub email_digest_frequency: String,
    pub do_not_disturb: bool,
    pub dnd_start_time: Option<String>,
    pub dnd_end_time: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
