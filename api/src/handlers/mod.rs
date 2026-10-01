pub mod auth;
pub mod channel;
pub mod dm;
pub mod health;
pub mod message;
pub mod message_history;
pub mod org;
pub mod pin;
pub mod presence;
pub mod reaction;
pub mod thread;
pub mod user;
pub mod ws;

use db::repositories::{
    SqlxBoardRepository, SqlxChannelRepository, SqlxDmRepository, SqlxMessageHistoryRepository,
    SqlxMessageRepository, SqlxNotificationRepository, SqlxOrgRepository, SqlxPinRepository,
    SqlxReactionRepository, SqlxTaskRepository, SqlxUserRepository,
};
use services::dm::DmService;
use services::message_history::MessageHistoryService;
use services::pin::PinService;
use services::reaction::ReactionService;
use services::thread::ThreadService;
use sqlx::PgPool;
use std::sync::Arc;

/// Registry of all repositories used by handlers.
#[derive(Clone)]
pub struct ServiceRegistry {
    pub db: PgPool,
    pub user_repo: SqlxUserRepository,
    pub org_repo: SqlxOrgRepository,
    pub channel_repo: SqlxChannelRepository,
    pub message_repo: SqlxMessageRepository,
    pub reaction_repo: SqlxReactionRepository,
    pub dm_repo: SqlxDmRepository,
    pub pin_repo: SqlxPinRepository,
    pub message_history_repo: SqlxMessageHistoryRepository,
    pub reaction_service: Arc<ReactionService>,
    pub thread_service: Arc<ThreadService>,
    pub dm_service: Arc<DmService>,
    pub pin_service: Arc<PinService>,
    pub message_history_service: Arc<MessageHistoryService>,
    pub board_repo: SqlxBoardRepository,
    pub task_repo: SqlxTaskRepository,
    pub notification_repo: SqlxNotificationRepository,
}

impl ServiceRegistry {
    /// Creates a new service registry.
    pub fn new(db: PgPool) -> Self {
        let reaction_repo = SqlxReactionRepository::new(db.clone());
        let reaction_service = Arc::new(ReactionService::new(Arc::new(reaction_repo.clone())));
        let thread_service = Arc::new(ThreadService::new(Arc::new(SqlxMessageRepository::new(
            db.clone(),
        ))));
        let dm_repo = SqlxDmRepository::new(db.clone());
        let dm_service = Arc::new(DmService::new(
            Arc::new(dm_repo.clone()),
            Arc::new(SqlxMessageRepository::new(db.clone())),
        ));
        let pin_repo = SqlxPinRepository::new(db.clone());
        let pin_service = Arc::new(PinService::new(Arc::new(pin_repo.clone())));
        let message_history_repo = SqlxMessageHistoryRepository::new(db.clone());
        let message_history_service = Arc::new(MessageHistoryService::new(Arc::new(
            message_history_repo.clone(),
        )));
        Self {
            user_repo: SqlxUserRepository::new(db.clone()),
            org_repo: SqlxOrgRepository::new(db.clone()),
            channel_repo: SqlxChannelRepository::new(db.clone()),
            message_repo: SqlxMessageRepository::new(db.clone()),
            reaction_repo: reaction_repo.clone(),
            dm_repo: dm_repo.clone(),
            pin_repo: pin_repo.clone(),
            message_history_repo: message_history_repo.clone(),
            reaction_service,
            thread_service,
            dm_service,
            pin_service,
            message_history_service,
            board_repo: SqlxBoardRepository::new(db.clone()),
            task_repo: SqlxTaskRepository::new(db.clone()),
            notification_repo: SqlxNotificationRepository::new(db.clone()),
            db,
        }
    }
}
