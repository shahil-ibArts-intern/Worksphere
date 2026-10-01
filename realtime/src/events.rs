//! Realtime event types — the typed envelope for all WebSocket events.
//!
//! All events use `#[serde(tag = "type", content = "payload")]` for typed
//! serialization. Every event payload includes `org_id` for tenant isolation
//! and `timestamp` for ordering.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use domain::events::{
    ChannelCreatedEvent, ChannelMemberJoinedEvent, MessageDeletedEvent, MessageEditedEvent,
    MessageNewEvent, NotificationNewEvent, PresenceChangedEvent, ReactionAddedEvent,
    ReactionRemovedEvent, TaskAssignedEvent, TaskCreatedEvent, TaskUpdatedEvent,
    TypingStartedEvent, TypingStoppedEvent, WhiteboardErrorEvent, WhiteboardHistoryChangedEvent,
    WhiteboardJoinEvent, WhiteboardJoinedEvent, WhiteboardLeaveEvent,
    WhiteboardPresenceChangedEvent, WhiteboardPresenceUpdateEvent, WhiteboardRedoEvent,
    WhiteboardSyncEvent, WhiteboardUndoEvent, WhiteboardUpdateEvent, WhiteboardUserLeftEvent,
    WsErrorEvent,
};
use domain::ids::{ChannelId, MessageId, OrgId, ReactionId, TaskId, UserId, WhiteboardId};

/// Maximum allowed WebSocket message size in bytes (1 MB).
pub const MAX_MESSAGE_SIZE: usize = 1_048_576;

/// The typed envelope for all WebSocket events.
///
/// Events are serialized with `type` as the tag field and `payload` as the
/// content, e.g. `{"type":"MessageNew","payload":{...}}`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum RealtimeEvent {
    // ── Communication ─────────────────────────────────────────────────
    /// A new message was created.
    MessageNew(MessageNewEvent),
    /// A message was edited.
    MessageEdited(MessageEditedEvent),
    /// A message was deleted.
    MessageDeleted(MessageDeletedEvent),
    /// A reaction was added to a message.
    ReactionAdded(ReactionAddedEvent),
    /// A reaction was removed from a message.
    ReactionRemoved(ReactionRemovedEvent),
    /// A user started typing in a channel.
    TypingStarted(TypingStartedEvent),
    /// A user stopped typing in a channel.
    TypingStopped(TypingStoppedEvent),

    // ── Presence ──────────────────────────────────────────────────────
    /// A user's presence status changed.
    PresenceChanged(PresenceChangedEvent),

    // ── Channels ──────────────────────────────────────────────────────
    /// A channel was created.
    ChannelCreated(ChannelCreatedEvent),
    /// A member joined a channel.
    ChannelMemberJoined(ChannelMemberJoinedEvent),

    // ── Tasks ─────────────────────────────────────────────────────────
    /// A task was created.
    TaskCreated(TaskCreatedEvent),
    /// A task was updated.
    TaskUpdated(TaskUpdatedEvent),
    /// A task was assigned to a user.
    TaskAssigned(TaskAssignedEvent),

    // ── Notifications ─────────────────────────────────────────────────
    /// A new notification was created.
    NotificationNew(NotificationNewEvent),

    // ── Whiteboard ────────────────────────────────────────────────────
    /// A user joined a whiteboard.
    WhiteboardJoin(WhiteboardJoinEvent),
    /// A user joined a whiteboard (acknowledgment).
    WhiteboardJoined(WhiteboardJoinedEvent),
    /// Whiteboard state was synced.
    WhiteboardSync(WhiteboardSyncEvent),
    /// Whiteboard was updated.
    WhiteboardUpdate(WhiteboardUpdateEvent),
    /// Whiteboard presence was updated.
    WhiteboardPresenceUpdate(WhiteboardPresenceUpdateEvent),
    /// Whiteboard presence changed.
    WhiteboardPresenceChanged(WhiteboardPresenceChangedEvent),
    /// A user left a whiteboard.
    WhiteboardLeave(WhiteboardLeaveEvent),
    /// A user left a whiteboard (broadcast).
    WhiteboardUserLeft(WhiteboardUserLeftEvent),
    /// Whiteboard undo action.
    WhiteboardUndo(WhiteboardUndoEvent),
    /// Whiteboard redo action.
    WhiteboardRedo(WhiteboardRedoEvent),
    /// Whiteboard history changed.
    WhiteboardHistoryChanged(WhiteboardHistoryChangedEvent),
    /// Whiteboard error occurred.
    WhiteboardError(WhiteboardErrorEvent),

    // ── System ────────────────────────────────────────────────────────
    /// Ping event for heartbeat.
    Ping,
    /// Pong event for heartbeat response.
    Pong,
    /// Error event.
    Error(WsErrorEvent),
}

impl RealtimeEvent {
    /// Returns the event type name as a string.
    pub fn event_type(&self) -> &'static str {
        match self {
            RealtimeEvent::MessageNew(_) => "MessageNew",
            RealtimeEvent::MessageEdited(_) => "MessageEdited",
            RealtimeEvent::MessageDeleted(_) => "MessageDeleted",
            RealtimeEvent::ReactionAdded(_) => "ReactionAdded",
            RealtimeEvent::ReactionRemoved(_) => "ReactionRemoved",
            RealtimeEvent::TypingStarted(_) => "TypingStarted",
            RealtimeEvent::TypingStopped(_) => "TypingStopped",
            RealtimeEvent::PresenceChanged(_) => "PresenceChanged",
            RealtimeEvent::ChannelCreated(_) => "ChannelCreated",
            RealtimeEvent::ChannelMemberJoined(_) => "ChannelMemberJoined",
            RealtimeEvent::TaskCreated(_) => "TaskCreated",
            RealtimeEvent::TaskUpdated(_) => "TaskUpdated",
            RealtimeEvent::TaskAssigned(_) => "TaskAssigned",
            RealtimeEvent::NotificationNew(_) => "NotificationNew",
            RealtimeEvent::WhiteboardJoin(_) => "WhiteboardJoin",
            RealtimeEvent::WhiteboardJoined(_) => "WhiteboardJoined",
            RealtimeEvent::WhiteboardSync(_) => "WhiteboardSync",
            RealtimeEvent::WhiteboardUpdate(_) => "WhiteboardUpdate",
            RealtimeEvent::WhiteboardPresenceUpdate(_) => "WhiteboardPresenceUpdate",
            RealtimeEvent::WhiteboardPresenceChanged(_) => "WhiteboardPresenceChanged",
            RealtimeEvent::WhiteboardLeave(_) => "WhiteboardLeave",
            RealtimeEvent::WhiteboardUserLeft(_) => "WhiteboardUserLeft",
            RealtimeEvent::WhiteboardUndo(_) => "WhiteboardUndo",
            RealtimeEvent::WhiteboardRedo(_) => "WhiteboardRedo",
            RealtimeEvent::WhiteboardHistoryChanged(_) => "WhiteboardHistoryChanged",
            RealtimeEvent::WhiteboardError(_) => "WhiteboardError",
            RealtimeEvent::Ping => "Ping",
            RealtimeEvent::Pong => "Pong",
            RealtimeEvent::Error(_) => "Error",
        }
    }

    /// Returns the organization ID this event belongs to, if applicable.
    pub fn org_id(&self) -> Option<OrgId> {
        match self {
            RealtimeEvent::MessageNew(e) => Some(e.org_id),
            RealtimeEvent::MessageEdited(e) => Some(e.org_id),
            RealtimeEvent::MessageDeleted(e) => Some(e.org_id),
            RealtimeEvent::ReactionAdded(e) => Some(e.org_id),
            RealtimeEvent::ReactionRemoved(e) => Some(e.org_id),
            RealtimeEvent::TypingStarted(e) => Some(e.org_id),
            RealtimeEvent::TypingStopped(e) => Some(e.org_id),
            RealtimeEvent::PresenceChanged(e) => Some(e.org_id),
            RealtimeEvent::ChannelCreated(e) => Some(e.org_id),
            RealtimeEvent::ChannelMemberJoined(e) => Some(e.org_id),
            RealtimeEvent::TaskCreated(e) => Some(e.org_id),
            RealtimeEvent::TaskUpdated(e) => Some(e.org_id),
            RealtimeEvent::TaskAssigned(e) => Some(e.org_id),
            RealtimeEvent::NotificationNew(e) => Some(e.org_id),
            RealtimeEvent::WhiteboardJoin(e) => Some(e.org_id),
            RealtimeEvent::WhiteboardJoined(e) => Some(e.org_id),
            RealtimeEvent::WhiteboardSync(e) => Some(e.org_id),
            RealtimeEvent::WhiteboardUpdate(e) => Some(e.org_id),
            RealtimeEvent::WhiteboardPresenceUpdate(e) => Some(e.org_id),
            RealtimeEvent::WhiteboardPresenceChanged(e) => Some(e.org_id),
            RealtimeEvent::WhiteboardLeave(e) => Some(e.org_id),
            RealtimeEvent::WhiteboardUserLeft(e) => Some(e.org_id),
            RealtimeEvent::WhiteboardUndo(e) => Some(e.org_id),
            RealtimeEvent::WhiteboardRedo(e) => Some(e.org_id),
            RealtimeEvent::WhiteboardHistoryChanged(e) => Some(e.org_id),
            RealtimeEvent::WhiteboardError(e) => Some(e.org_id),
            RealtimeEvent::Ping | RealtimeEvent::Pong | RealtimeEvent::Error(_) => None,
        }
    }

    /// Returns the timestamp of the event, if applicable.
    pub fn timestamp(&self) -> Option<DateTime<Utc>> {
        match self {
            RealtimeEvent::MessageNew(e) => Some(e.timestamp),
            RealtimeEvent::MessageEdited(e) => Some(e.timestamp),
            RealtimeEvent::MessageDeleted(e) => Some(e.timestamp),
            RealtimeEvent::ReactionAdded(e) => Some(e.timestamp),
            RealtimeEvent::ReactionRemoved(e) => Some(e.timestamp),
            RealtimeEvent::TypingStarted(e) => Some(e.timestamp),
            RealtimeEvent::TypingStopped(e) => Some(e.timestamp),
            RealtimeEvent::PresenceChanged(e) => Some(e.timestamp),
            RealtimeEvent::ChannelCreated(e) => Some(e.timestamp),
            RealtimeEvent::ChannelMemberJoined(e) => Some(e.timestamp),
            RealtimeEvent::TaskCreated(e) => Some(e.timestamp),
            RealtimeEvent::TaskUpdated(e) => Some(e.timestamp),
            RealtimeEvent::TaskAssigned(e) => Some(e.timestamp),
            RealtimeEvent::NotificationNew(e) => Some(e.timestamp),
            RealtimeEvent::WhiteboardJoin(e) => Some(e.timestamp),
            RealtimeEvent::WhiteboardJoined(e) => Some(e.timestamp),
            RealtimeEvent::WhiteboardSync(e) => Some(e.timestamp),
            RealtimeEvent::WhiteboardUpdate(e) => Some(e.timestamp),
            RealtimeEvent::WhiteboardPresenceUpdate(e) => Some(e.timestamp),
            RealtimeEvent::WhiteboardPresenceChanged(e) => Some(e.timestamp),
            RealtimeEvent::WhiteboardLeave(e) => Some(e.timestamp),
            RealtimeEvent::WhiteboardUserLeft(e) => Some(e.timestamp),
            RealtimeEvent::WhiteboardUndo(e) => Some(e.timestamp),
            RealtimeEvent::WhiteboardRedo(e) => Some(e.timestamp),
            RealtimeEvent::WhiteboardHistoryChanged(e) => Some(e.timestamp),
            RealtimeEvent::WhiteboardError(e) => Some(e.timestamp),
            RealtimeEvent::Ping | RealtimeEvent::Pong | RealtimeEvent::Error(_) => None,
        }
    }
}

// ── Event Constructors ───────────────────────────────────────────────────

/// Builder for creating `RealtimeEvent` instances with current timestamps.
pub struct EventBuilder;

impl EventBuilder {
    /// Creates a `MessageNew` event.
    pub fn message_new(
        org_id: OrgId,
        channel_id: ChannelId,
        message_id: MessageId,
        user_id: UserId,
        content: impl Into<String>,
    ) -> RealtimeEvent {
        RealtimeEvent::MessageNew(MessageNewEvent {
            org_id,
            channel_id,
            message_id,
            user_id,
            content: content.into(),
            timestamp: Utc::now(),
        })
    }

    /// Creates a `MessageEdited` event.
    pub fn message_edited(
        org_id: OrgId,
        channel_id: ChannelId,
        message_id: MessageId,
        user_id: UserId,
        content: impl Into<String>,
    ) -> RealtimeEvent {
        RealtimeEvent::MessageEdited(MessageEditedEvent {
            org_id,
            channel_id,
            message_id,
            user_id,
            content: content.into(),
            timestamp: Utc::now(),
        })
    }

    /// Creates a `MessageDeleted` event.
    pub fn message_deleted(
        org_id: OrgId,
        channel_id: ChannelId,
        message_id: MessageId,
        user_id: UserId,
    ) -> RealtimeEvent {
        RealtimeEvent::MessageDeleted(MessageDeletedEvent {
            org_id,
            channel_id,
            message_id,
            user_id,
            timestamp: Utc::now(),
        })
    }

    /// Creates a `ReactionAdded` event.
    pub fn reaction_added(
        org_id: OrgId,
        message_id: MessageId,
        user_id: UserId,
        reaction_id: ReactionId,
        emoji: impl Into<String>,
    ) -> RealtimeEvent {
        RealtimeEvent::ReactionAdded(ReactionAddedEvent {
            org_id,
            message_id,
            user_id,
            reaction_id,
            emoji: emoji.into(),
            timestamp: Utc::now(),
        })
    }

    /// Creates a `ReactionRemoved` event.
    pub fn reaction_removed(
        org_id: OrgId,
        message_id: MessageId,
        user_id: UserId,
        reaction_id: ReactionId,
        emoji: impl Into<String>,
    ) -> RealtimeEvent {
        RealtimeEvent::ReactionRemoved(ReactionRemovedEvent {
            org_id,
            message_id,
            user_id,
            reaction_id,
            emoji: emoji.into(),
            timestamp: Utc::now(),
        })
    }

    /// Creates a `TypingStarted` event.
    pub fn typing_started(org_id: OrgId, channel_id: ChannelId, user_id: UserId) -> RealtimeEvent {
        RealtimeEvent::TypingStarted(TypingStartedEvent {
            org_id,
            channel_id,
            user_id,
            timestamp: Utc::now(),
        })
    }

    /// Creates a `TypingStopped` event.
    pub fn typing_stopped(org_id: OrgId, channel_id: ChannelId, user_id: UserId) -> RealtimeEvent {
        RealtimeEvent::TypingStopped(TypingStoppedEvent {
            org_id,
            channel_id,
            user_id,
            timestamp: Utc::now(),
        })
    }

    /// Creates a `PresenceChanged` event.
    pub fn presence_changed(
        org_id: OrgId,
        user_id: UserId,
        status: impl Into<String>,
        custom_status: Option<String>,
    ) -> RealtimeEvent {
        RealtimeEvent::PresenceChanged(PresenceChangedEvent {
            org_id,
            user_id,
            status: status.into(),
            custom_status,
            timestamp: Utc::now(),
        })
    }

    /// Creates a `ChannelCreated` event.
    pub fn channel_created(
        org_id: OrgId,
        channel_id: ChannelId,
        name: impl Into<String>,
        created_by: UserId,
    ) -> RealtimeEvent {
        RealtimeEvent::ChannelCreated(ChannelCreatedEvent {
            org_id,
            channel_id,
            name: name.into(),
            created_by,
            timestamp: Utc::now(),
        })
    }

    /// Creates a `ChannelMemberJoined` event.
    pub fn channel_member_joined(
        org_id: OrgId,
        channel_id: ChannelId,
        user_id: UserId,
    ) -> RealtimeEvent {
        RealtimeEvent::ChannelMemberJoined(ChannelMemberJoinedEvent {
            org_id,
            channel_id,
            user_id,
            timestamp: Utc::now(),
        })
    }

    /// Creates a `TaskCreated` event.
    pub fn task_created(
        org_id: OrgId,
        task_id: TaskId,
        board_id: Uuid,
        title: impl Into<String>,
        created_by: UserId,
    ) -> RealtimeEvent {
        RealtimeEvent::TaskCreated(TaskCreatedEvent {
            org_id,
            task_id: task_id.as_uuid(),
            board_id,
            title: title.into(),
            created_by,
            timestamp: Utc::now(),
        })
    }

    /// Creates a `TaskUpdated` event.
    pub fn task_updated(
        org_id: OrgId,
        task_id: TaskId,
        board_id: Uuid,
        updated_by: UserId,
    ) -> RealtimeEvent {
        RealtimeEvent::TaskUpdated(TaskUpdatedEvent {
            org_id,
            task_id: task_id.as_uuid(),
            board_id,
            updated_by,
            timestamp: Utc::now(),
        })
    }

    /// Creates a `TaskAssigned` event.
    pub fn task_assigned(
        org_id: OrgId,
        task_id: TaskId,
        assignee_id: UserId,
        assigned_by: UserId,
    ) -> RealtimeEvent {
        RealtimeEvent::TaskAssigned(TaskAssignedEvent {
            org_id,
            task_id: task_id.as_uuid(),
            assignee_id,
            assigned_by,
            timestamp: Utc::now(),
        })
    }

    /// Creates a `NotificationNew` event.
    pub fn notification_new(
        org_id: OrgId,
        notification_id: Uuid,
        user_id: UserId,
        title: impl Into<String>,
    ) -> RealtimeEvent {
        RealtimeEvent::NotificationNew(NotificationNewEvent {
            org_id,
            notification_id,
            user_id,
            title: title.into(),
            timestamp: Utc::now(),
        })
    }

    /// Creates a `WhiteboardJoin` event.
    pub fn whiteboard_join(
        org_id: OrgId,
        whiteboard_id: WhiteboardId,
        user_id: UserId,
    ) -> RealtimeEvent {
        RealtimeEvent::WhiteboardJoin(WhiteboardJoinEvent {
            org_id,
            whiteboard_id: whiteboard_id.as_uuid(),
            user_id,
            timestamp: Utc::now(),
        })
    }

    /// Creates a `WhiteboardSync` event.
    pub fn whiteboard_sync(
        org_id: OrgId,
        whiteboard_id: WhiteboardId,
        user_id: UserId,
    ) -> RealtimeEvent {
        RealtimeEvent::WhiteboardSync(WhiteboardSyncEvent {
            org_id,
            whiteboard_id: whiteboard_id.as_uuid(),
            user_id,
            timestamp: Utc::now(),
        })
    }

    /// Creates a `WhiteboardUpdate` event.
    pub fn whiteboard_update(
        org_id: OrgId,
        whiteboard_id: WhiteboardId,
        user_id: UserId,
    ) -> RealtimeEvent {
        RealtimeEvent::WhiteboardUpdate(WhiteboardUpdateEvent {
            org_id,
            whiteboard_id: whiteboard_id.as_uuid(),
            user_id,
            timestamp: Utc::now(),
        })
    }

    /// Creates a `Ping` event.
    pub fn ping() -> RealtimeEvent {
        RealtimeEvent::Ping
    }

    /// Creates a `Pong` event.
    pub fn pong() -> RealtimeEvent {
        RealtimeEvent::Pong
    }

    /// Creates an `Error` event.
    pub fn error(code: u16, message: impl Into<String>) -> RealtimeEvent {
        RealtimeEvent::Error(WsErrorEvent {
            code,
            message: message.into(),
        })
    }
}

// ── Tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    fn test_org_id() -> OrgId {
        OrgId::from_uuid(Uuid::now_v7())
    }

    fn test_user_id() -> UserId {
        UserId::from_uuid(Uuid::now_v7())
    }

    fn test_channel_id() -> ChannelId {
        ChannelId::from_uuid(Uuid::now_v7())
    }

    fn test_message_id() -> MessageId {
        MessageId::from_uuid(Uuid::now_v7())
    }

    #[test]
    fn test_message_new_serialization() {
        let event = EventBuilder::message_new(
            test_org_id(),
            test_channel_id(),
            test_message_id(),
            test_user_id(),
            "Hello, world!",
        );

        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("\"type\":\"MessageNew\""));
        assert!(json.contains("\"payload\""));
        assert!(json.contains("Hello, world!"));

        let deserialized: RealtimeEvent = serde_json::from_str(&json).unwrap();
        match deserialized {
            RealtimeEvent::MessageNew(e) => {
                assert_eq!(e.content, "Hello, world!");
            }
            _ => panic!("Wrong event type"),
        }
    }

    #[test]
    fn test_ping_pong_serialization() {
        let ping = EventBuilder::ping();
        let json = serde_json::to_string(&ping).unwrap();
        assert!(json.contains("\"type\":\"Ping\""));

        let pong = EventBuilder::pong();
        let json = serde_json::to_string(&pong).unwrap();
        assert!(json.contains("\"type\":\"Pong\""));
    }

    #[test]
    fn test_error_serialization() {
        let event = EventBuilder::error(4001, "Invalid message format");
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("\"type\":\"Error\""));
        assert!(json.contains("4001"));
        assert!(json.contains("Invalid message format"));
    }

    #[test]
    fn test_event_type_names() {
        assert_eq!(EventBuilder::ping().event_type(), "Ping");
        assert_eq!(EventBuilder::pong().event_type(), "Pong");
        assert_eq!(
            EventBuilder::message_new(
                test_org_id(),
                test_channel_id(),
                test_message_id(),
                test_user_id(),
                "test"
            )
            .event_type(),
            "MessageNew"
        );
    }

    #[test]
    fn test_org_id_extraction() {
        let org_id = test_org_id();
        let event = EventBuilder::message_new(
            org_id,
            test_channel_id(),
            test_message_id(),
            test_user_id(),
            "test",
        );
        assert_eq!(event.org_id(), Some(org_id));

        let ping = EventBuilder::ping();
        assert_eq!(ping.org_id(), None);
    }

    #[test]
    fn test_channel_created_serialization() {
        let event = EventBuilder::channel_created(
            test_org_id(),
            test_channel_id(),
            "general",
            test_user_id(),
        );
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("\"type\":\"ChannelCreated\""));
        assert!(json.contains("general"));
    }

    #[test]
    fn test_presence_changed_serialization() {
        let event = EventBuilder::presence_changed(
            test_org_id(),
            test_user_id(),
            "online",
            Some("Working on Rust".to_string()),
        );
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("\"type\":\"PresenceChanged\""));
        assert!(json.contains("online"));
        assert!(json.contains("Working on Rust"));
    }

    #[test]
    fn test_max_message_size_constant() {
        assert_eq!(MAX_MESSAGE_SIZE, 1_048_576);
    }
}
