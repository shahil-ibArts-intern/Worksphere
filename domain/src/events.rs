use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::ids::{ChannelId, MessageId, OrgId, ReactionId, UserId};

/// Shared event type definitions used across crates.
/// All events include org_id for tenant isolation and timestamp for ordering.
///
/// Base trait for all real-time events.
pub trait RealtimeEvent: Send + Sync {
    /// Returns the event type name.
    fn event_type(&self) -> &'static str;

    /// Returns the organization ID this event belongs to.
    fn org_id(&self) -> OrgId;

    /// Returns the timestamp of when the event occurred.
    fn timestamp(&self) -> DateTime<Utc>;
}

// ── Communication Events ─────────────────────────────────────────────

/// Event emitted when a new message is created.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MessageNewEvent {
    pub org_id: OrgId,
    pub channel_id: ChannelId,
    pub message_id: MessageId,
    pub user_id: UserId,
    pub content: String,
    pub timestamp: DateTime<Utc>,
}

impl RealtimeEvent for MessageNewEvent {
    fn event_type(&self) -> &'static str {
        "MessageNew"
    }
    fn org_id(&self) -> OrgId {
        self.org_id
    }
    fn timestamp(&self) -> DateTime<Utc> {
        self.timestamp
    }
}

/// Event emitted when a message is edited.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MessageEditedEvent {
    pub org_id: OrgId,
    pub channel_id: ChannelId,
    pub message_id: MessageId,
    pub user_id: UserId,
    pub content: String,
    pub timestamp: DateTime<Utc>,
}

impl RealtimeEvent for MessageEditedEvent {
    fn event_type(&self) -> &'static str {
        "MessageEdited"
    }
    fn org_id(&self) -> OrgId {
        self.org_id
    }
    fn timestamp(&self) -> DateTime<Utc> {
        self.timestamp
    }
}

/// Event emitted when a message is deleted.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MessageDeletedEvent {
    pub org_id: OrgId,
    pub channel_id: ChannelId,
    pub message_id: MessageId,
    pub user_id: UserId,
    pub timestamp: DateTime<Utc>,
}

impl RealtimeEvent for MessageDeletedEvent {
    fn event_type(&self) -> &'static str {
        "MessageDeleted"
    }
    fn org_id(&self) -> OrgId {
        self.org_id
    }
    fn timestamp(&self) -> DateTime<Utc> {
        self.timestamp
    }
}

// ── Presence Events ──────────────────────────────────────────────────

/// Event emitted when a user's presence changes.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PresenceChangedEvent {
    pub org_id: OrgId,
    pub user_id: UserId,
    pub status: String,
    pub custom_status: Option<String>,
    pub timestamp: DateTime<Utc>,
}

impl RealtimeEvent for PresenceChangedEvent {
    fn event_type(&self) -> &'static str {
        "PresenceChanged"
    }
    fn org_id(&self) -> OrgId {
        self.org_id
    }
    fn timestamp(&self) -> DateTime<Utc> {
        self.timestamp
    }
}

// ── Channel Events ───────────────────────────────────────────────────

/// Event emitted when a channel is created.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChannelCreatedEvent {
    pub org_id: OrgId,
    pub channel_id: ChannelId,
    pub name: String,
    pub created_by: UserId,
    pub timestamp: DateTime<Utc>,
}

impl RealtimeEvent for ChannelCreatedEvent {
    fn event_type(&self) -> &'static str {
        "ChannelCreated"
    }
    fn org_id(&self) -> OrgId {
        self.org_id
    }
    fn timestamp(&self) -> DateTime<Utc> {
        self.timestamp
    }
}

/// Event emitted when a member joins a channel.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChannelMemberJoinedEvent {
    pub org_id: OrgId,
    pub channel_id: ChannelId,
    pub user_id: UserId,
    pub timestamp: DateTime<Utc>,
}

impl RealtimeEvent for ChannelMemberJoinedEvent {
    fn event_type(&self) -> &'static str {
        "ChannelMemberJoined"
    }
    fn org_id(&self) -> OrgId {
        self.org_id
    }
    fn timestamp(&self) -> DateTime<Utc> {
        self.timestamp
    }
}

// ── Task Events ──────────────────────────────────────────────────────

/// Event emitted when a task is created.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskCreatedEvent {
    pub org_id: OrgId,
    pub task_id: Uuid,
    pub board_id: Uuid,
    pub title: String,
    pub created_by: UserId,
    pub timestamp: DateTime<Utc>,
}

impl RealtimeEvent for TaskCreatedEvent {
    fn event_type(&self) -> &'static str {
        "TaskCreated"
    }
    fn org_id(&self) -> OrgId {
        self.org_id
    }
    fn timestamp(&self) -> DateTime<Utc> {
        self.timestamp
    }
}

/// Event emitted when a task is updated.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskUpdatedEvent {
    pub org_id: OrgId,
    pub task_id: Uuid,
    pub board_id: Uuid,
    pub updated_by: UserId,
    pub timestamp: DateTime<Utc>,
}

impl RealtimeEvent for TaskUpdatedEvent {
    fn event_type(&self) -> &'static str {
        "TaskUpdated"
    }
    fn org_id(&self) -> OrgId {
        self.org_id
    }
    fn timestamp(&self) -> DateTime<Utc> {
        self.timestamp
    }
}

/// Event emitted when a task is assigned.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskAssignedEvent {
    pub org_id: OrgId,
    pub task_id: Uuid,
    pub assignee_id: UserId,
    pub assigned_by: UserId,
    pub timestamp: DateTime<Utc>,
}

impl RealtimeEvent for TaskAssignedEvent {
    fn event_type(&self) -> &'static str {
        "TaskAssigned"
    }
    fn org_id(&self) -> OrgId {
        self.org_id
    }
    fn timestamp(&self) -> DateTime<Utc> {
        self.timestamp
    }
}

// ── Notification Events ──────────────────────────────────────────────

/// Event emitted when a new notification is created.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NotificationNewEvent {
    pub org_id: OrgId,
    pub notification_id: Uuid,
    pub user_id: UserId,
    pub title: String,
    pub timestamp: DateTime<Utc>,
}

impl RealtimeEvent for NotificationNewEvent {
    fn event_type(&self) -> &'static str {
        "NotificationNew"
    }
    fn org_id(&self) -> OrgId {
        self.org_id
    }
    fn timestamp(&self) -> DateTime<Utc> {
        self.timestamp
    }
}

// ── Whiteboard Events ─────────────────────────────────────────────────

/// Event emitted when a user joins a whiteboard.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WhiteboardJoinEvent {
    pub org_id: OrgId,
    pub whiteboard_id: Uuid,
    pub user_id: UserId,
    pub timestamp: DateTime<Utc>,
}

impl RealtimeEvent for WhiteboardJoinEvent {
    fn event_type(&self) -> &'static str {
        "WhiteboardJoin"
    }
    fn org_id(&self) -> OrgId {
        self.org_id
    }
    fn timestamp(&self) -> DateTime<Utc> {
        self.timestamp
    }
}

/// Event emitted when whiteboard state is synced.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WhiteboardSyncEvent {
    pub org_id: OrgId,
    pub whiteboard_id: Uuid,
    pub user_id: UserId,
    pub timestamp: DateTime<Utc>,
}

impl RealtimeEvent for WhiteboardSyncEvent {
    fn event_type(&self) -> &'static str {
        "WhiteboardSync"
    }
    fn org_id(&self) -> OrgId {
        self.org_id
    }
    fn timestamp(&self) -> DateTime<Utc> {
        self.timestamp
    }
}

/// Event emitted when whiteboard is updated.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WhiteboardUpdateEvent {
    pub org_id: OrgId,
    pub whiteboard_id: Uuid,
    pub user_id: UserId,
    pub timestamp: DateTime<Utc>,
}

impl RealtimeEvent for WhiteboardUpdateEvent {
    fn event_type(&self) -> &'static str {
        "WhiteboardUpdate"
    }
    fn org_id(&self) -> OrgId {
        self.org_id
    }
    fn timestamp(&self) -> DateTime<Utc> {
        self.timestamp
    }
}

// ── Reaction Events ───────────────────────────────────────────────────

/// Event emitted when a reaction is added to a message.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReactionAddedEvent {
    pub org_id: OrgId,
    pub message_id: MessageId,
    pub user_id: UserId,
    pub reaction_id: ReactionId,
    pub emoji: String,
    pub timestamp: DateTime<Utc>,
}

impl RealtimeEvent for ReactionAddedEvent {
    fn event_type(&self) -> &'static str {
        "ReactionAdded"
    }
    fn org_id(&self) -> OrgId {
        self.org_id
    }
    fn timestamp(&self) -> DateTime<Utc> {
        self.timestamp
    }
}

/// Event emitted when a reaction is removed from a message.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReactionRemovedEvent {
    pub org_id: OrgId,
    pub message_id: MessageId,
    pub user_id: UserId,
    pub reaction_id: ReactionId,
    pub emoji: String,
    pub timestamp: DateTime<Utc>,
}

impl RealtimeEvent for ReactionRemovedEvent {
    fn event_type(&self) -> &'static str {
        "ReactionRemoved"
    }
    fn org_id(&self) -> OrgId {
        self.org_id
    }
    fn timestamp(&self) -> DateTime<Utc> {
        self.timestamp
    }
}

// ── Typing Events ─────────────────────────────────────────────────────

/// Event emitted when a user starts typing.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TypingStartedEvent {
    pub org_id: OrgId,
    pub channel_id: ChannelId,
    pub user_id: UserId,
    pub timestamp: DateTime<Utc>,
}

impl RealtimeEvent for TypingStartedEvent {
    fn event_type(&self) -> &'static str {
        "TypingStarted"
    }
    fn org_id(&self) -> OrgId {
        self.org_id
    }
    fn timestamp(&self) -> DateTime<Utc> {
        self.timestamp
    }
}

/// Event emitted when a user stops typing.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TypingStoppedEvent {
    pub org_id: OrgId,
    pub channel_id: ChannelId,
    pub user_id: UserId,
    pub timestamp: DateTime<Utc>,
}

impl RealtimeEvent for TypingStoppedEvent {
    fn event_type(&self) -> &'static str {
        "TypingStopped"
    }
    fn org_id(&self) -> OrgId {
        self.org_id
    }
    fn timestamp(&self) -> DateTime<Utc> {
        self.timestamp
    }
}

// ── Whiteboard Events (extended) ───────────────────────────────────────

/// Event emitted when a user joined a whiteboard (acknowledgment).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WhiteboardJoinedEvent {
    pub org_id: OrgId,
    pub whiteboard_id: Uuid,
    pub user_id: UserId,
    pub timestamp: DateTime<Utc>,
}

impl RealtimeEvent for WhiteboardJoinedEvent {
    fn event_type(&self) -> &'static str {
        "WhiteboardJoined"
    }
    fn org_id(&self) -> OrgId {
        self.org_id
    }
    fn timestamp(&self) -> DateTime<Utc> {
        self.timestamp
    }
}

/// Event emitted when whiteboard presence is updated.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WhiteboardPresenceUpdateEvent {
    pub org_id: OrgId,
    pub whiteboard_id: Uuid,
    pub user_id: UserId,
    pub timestamp: DateTime<Utc>,
}

impl RealtimeEvent for WhiteboardPresenceUpdateEvent {
    fn event_type(&self) -> &'static str {
        "WhiteboardPresenceUpdate"
    }
    fn org_id(&self) -> OrgId {
        self.org_id
    }
    fn timestamp(&self) -> DateTime<Utc> {
        self.timestamp
    }
}

/// Event emitted when whiteboard presence changes.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WhiteboardPresenceChangedEvent {
    pub org_id: OrgId,
    pub whiteboard_id: Uuid,
    pub user_id: UserId,
    pub timestamp: DateTime<Utc>,
}

impl RealtimeEvent for WhiteboardPresenceChangedEvent {
    fn event_type(&self) -> &'static str {
        "WhiteboardPresenceChanged"
    }
    fn org_id(&self) -> OrgId {
        self.org_id
    }
    fn timestamp(&self) -> DateTime<Utc> {
        self.timestamp
    }
}

/// Event emitted when a user leaves a whiteboard.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WhiteboardLeaveEvent {
    pub org_id: OrgId,
    pub whiteboard_id: Uuid,
    pub user_id: UserId,
    pub timestamp: DateTime<Utc>,
}

impl RealtimeEvent for WhiteboardLeaveEvent {
    fn event_type(&self) -> &'static str {
        "WhiteboardLeave"
    }
    fn org_id(&self) -> OrgId {
        self.org_id
    }
    fn timestamp(&self) -> DateTime<Utc> {
        self.timestamp
    }
}

/// Event emitted when a user leaves a whiteboard (broadcast).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WhiteboardUserLeftEvent {
    pub org_id: OrgId,
    pub whiteboard_id: Uuid,
    pub user_id: UserId,
    pub timestamp: DateTime<Utc>,
}

impl RealtimeEvent for WhiteboardUserLeftEvent {
    fn event_type(&self) -> &'static str {
        "WhiteboardUserLeft"
    }
    fn org_id(&self) -> OrgId {
        self.org_id
    }
    fn timestamp(&self) -> DateTime<Utc> {
        self.timestamp
    }
}

/// Event emitted on whiteboard undo.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WhiteboardUndoEvent {
    pub org_id: OrgId,
    pub whiteboard_id: Uuid,
    pub user_id: UserId,
    pub timestamp: DateTime<Utc>,
}

impl RealtimeEvent for WhiteboardUndoEvent {
    fn event_type(&self) -> &'static str {
        "WhiteboardUndo"
    }
    fn org_id(&self) -> OrgId {
        self.org_id
    }
    fn timestamp(&self) -> DateTime<Utc> {
        self.timestamp
    }
}

/// Event emitted on whiteboard redo.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WhiteboardRedoEvent {
    pub org_id: OrgId,
    pub whiteboard_id: Uuid,
    pub user_id: UserId,
    pub timestamp: DateTime<Utc>,
}

impl RealtimeEvent for WhiteboardRedoEvent {
    fn event_type(&self) -> &'static str {
        "WhiteboardRedo"
    }
    fn org_id(&self) -> OrgId {
        self.org_id
    }
    fn timestamp(&self) -> DateTime<Utc> {
        self.timestamp
    }
}

/// Event emitted when whiteboard history changes.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WhiteboardHistoryChangedEvent {
    pub org_id: OrgId,
    pub whiteboard_id: Uuid,
    pub user_id: UserId,
    pub timestamp: DateTime<Utc>,
}

impl RealtimeEvent for WhiteboardHistoryChangedEvent {
    fn event_type(&self) -> &'static str {
        "WhiteboardHistoryChanged"
    }
    fn org_id(&self) -> OrgId {
        self.org_id
    }
    fn timestamp(&self) -> DateTime<Utc> {
        self.timestamp
    }
}

/// Event emitted on whiteboard error.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WhiteboardErrorEvent {
    pub org_id: OrgId,
    pub whiteboard_id: Uuid,
    pub user_id: UserId,
    pub message: String,
    pub timestamp: DateTime<Utc>,
}

impl RealtimeEvent for WhiteboardErrorEvent {
    fn event_type(&self) -> &'static str {
        "WhiteboardError"
    }
    fn org_id(&self) -> OrgId {
        self.org_id
    }
    fn timestamp(&self) -> DateTime<Utc> {
        self.timestamp
    }
}

// ── System Events ────────────────────────────────────────────────────

/// Ping event for heartbeat.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PingEvent;

/// Pong event for heartbeat response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PongEvent;

/// WebSocket error event.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WsErrorEvent {
    pub code: u16,
    pub message: String,
}
