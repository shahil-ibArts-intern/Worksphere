//! Services crate — business logic services.
//!
//! This crate depends on `core`, `db`, and `auth`.

pub mod channel;
pub mod dm;
pub mod message;
pub mod message_history;
pub mod notification;
pub mod org;
pub mod pin;
pub mod presence;
pub mod reaction;
pub mod task;
pub mod thread;
pub mod user;

pub use channel::ChannelService;
pub use dm::{DmService, RealtimePublisher as DmRealtimePublisher};
pub use message::MessageService;
pub use message_history::MessageHistoryService;
pub use notification::NotificationService;
pub use org::OrgService;
pub use pin::{PinService, RealtimePublisher as PinRealtimePublisher};
pub use presence::{InMemoryPresenceService, PresenceService};
pub use reaction::{ReactionCount, ReactionService, RealtimePublisher};
pub use task::TaskService;
pub use thread::{RealtimePublisher as ThreadRealtimePublisher, ThreadService};
pub use user::UserService;
