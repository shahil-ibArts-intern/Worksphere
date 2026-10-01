use async_trait::async_trait;

use crate::error::AppResult;
use crate::ids::{MessageId, UserId};
use crate::types::message::Reaction;

/// Repository trait for reaction data access.
#[async_trait]
pub trait ReactionRepository: Send + Sync {
    /// Adds a reaction to a message.
    async fn add_reaction(&self, reaction: &Reaction) -> AppResult<Reaction>;

    /// Removes a reaction from a message.
    async fn remove_reaction(
        &self,
        message_id: MessageId,
        user_id: UserId,
        emoji: &str,
    ) -> AppResult<()>;

    /// Gets all reactions for a message.
    async fn get_reactions_by_message(&self, message_id: MessageId) -> AppResult<Vec<Reaction>>;

    /// Gets reactions by a user.
    async fn get_reactions_by_user(&self, user_id: UserId) -> AppResult<Vec<Reaction>>;

    /// Counts reactions on a message grouped by emoji.
    async fn count_reactions_by_message(
        &self,
        message_id: MessageId,
    ) -> AppResult<Vec<(String, i64)>>;

    /// Checks if a user has reacted with a specific emoji.
    async fn has_user_reacted(
        &self,
        message_id: MessageId,
        user_id: UserId,
        emoji: &str,
    ) -> AppResult<bool>;
}
