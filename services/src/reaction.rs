//! Reaction service — business logic for emoji reactions on messages.

use std::sync::Arc;
use tracing::{info, instrument};

use domain::error::{AppError, AppResult};
use domain::ids::{MessageId, OrgId, ReactionId, UserId};
use domain::traits::ReactionRepository;
use domain::types::message::Reaction;
use realtime::{EventBuilder, RealtimeEvent};

/// Service for reaction-related business logic.
pub struct ReactionService {
    repo: Arc<dyn ReactionRepository>,
    realtime: Option<Arc<dyn RealtimePublisher>>,
}

/// Trait for publishing real-time events (to avoid circular dependency).
#[async_trait::async_trait]
pub trait RealtimePublisher: Send + Sync {
    async fn publish(&self, event: RealtimeEvent) -> Result<(), String>;
}

impl ReactionService {
    /// Creates a new reaction service.
    pub fn new(repo: Arc<dyn ReactionRepository>) -> Self {
        Self {
            repo,
            realtime: None,
        }
    }

    /// Creates a new reaction service with real-time publishing.
    pub fn with_realtime(
        repo: Arc<dyn ReactionRepository>,
        realtime: Arc<dyn RealtimePublisher>,
    ) -> Self {
        Self {
            repo,
            realtime: Some(realtime),
        }
    }

    /// Adds a reaction to a message.
    #[instrument(skip(self))]
    pub async fn add_reaction(
        &self,
        message_id: MessageId,
        org_id: OrgId,
        user_id: UserId,
        emoji: &str,
    ) -> AppResult<Reaction> {
        // Check if user already reacted with this emoji
        if self
            .repo
            .has_user_reacted(message_id, user_id, emoji)
            .await?
        {
            // Idempotent: return existing reaction
            let reactions = self.repo.get_reactions_by_message(message_id).await?;
            return reactions
                .into_iter()
                .find(|r| r.user_id == user_id && r.emoji == emoji)
                .ok_or_else(|| {
                    AppError::Internal("Reaction should exist but not found".to_string())
                });
        }

        let reaction = Reaction {
            id: ReactionId::new(),
            message_id,
            user_id,
            emoji: emoji.to_string(),
            created_at: chrono::Utc::now(),
        };

        let reaction = self.repo.add_reaction(&reaction).await?;

        info!(message_id = %message_id, user_id = %user_id, emoji = %emoji, "Reaction added");

        // Broadcast real-time event
        if let Some(ref realtime) = self.realtime {
            let event =
                EventBuilder::reaction_added(org_id, message_id, user_id, reaction.id, emoji);
            let _ = realtime.publish(event).await;
        }

        Ok(reaction)
    }

    /// Removes a reaction from a message.
    #[instrument(skip(self))]
    pub async fn remove_reaction(
        &self,
        message_id: MessageId,
        org_id: OrgId,
        user_id: UserId,
        emoji: &str,
    ) -> AppResult<()> {
        // Verify the reaction exists and belongs to the user
        if !self
            .repo
            .has_user_reacted(message_id, user_id, emoji)
            .await?
        {
            return Err(AppError::NotFound {
                resource: "Reaction".to_string(),
                id: format!("message={}, user={}, emoji={}", message_id, user_id, emoji),
            });
        }

        self.repo
            .remove_reaction(message_id, user_id, emoji)
            .await?;

        info!(message_id = %message_id, user_id = %user_id, emoji = %emoji, "Reaction removed");

        // Broadcast real-time event
        if let Some(ref realtime) = self.realtime {
            let event = EventBuilder::reaction_removed(
                org_id,
                message_id,
                user_id,
                ReactionId::new(), // reaction_id not needed for removal event
                emoji,
            );
            let _ = realtime.publish(event).await;
        }

        Ok(())
    }

    /// Gets all reactions for a message with counts.
    #[instrument(skip(self))]
    pub async fn get_reactions(&self, message_id: MessageId) -> AppResult<Vec<ReactionCount>> {
        let counts = self.repo.count_reactions_by_message(message_id).await?;
        let reactions = self.repo.get_reactions_by_message(message_id).await?;

        let mut result = Vec::new();
        for (emoji, count) in counts {
            let users: Vec<UserId> = reactions
                .iter()
                .filter(|r| r.emoji == emoji)
                .map(|r| r.user_id)
                .collect();
            result.push(ReactionCount {
                emoji,
                count,
                users,
            });
        }

        Ok(result)
    }

    /// Gets reactions added by a specific user.
    #[instrument(skip(self))]
    pub async fn get_user_reactions(&self, user_id: UserId) -> AppResult<Vec<Reaction>> {
        self.repo.get_reactions_by_user(user_id).await
    }
}

/// Reaction count with user list.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReactionCount {
    pub emoji: String,
    pub count: i64,
    pub users: Vec<UserId>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain::ids::ReactionId;

    #[test]
    fn test_reaction_creation() {
        let reaction = Reaction {
            id: ReactionId::new(),
            message_id: MessageId::new(),
            user_id: UserId::new(),
            emoji: "👍".to_string(),
            created_at: chrono::Utc::now(),
        };
        assert_eq!(reaction.emoji, "👍");
    }

    #[test]
    fn test_reaction_count_serialization() {
        let count = ReactionCount {
            emoji: "👍".to_string(),
            count: 5,
            users: vec![UserId::new(), UserId::new()],
        };
        let json = serde_json::to_string(&count).unwrap();
        assert!(json.contains("👍"));
        assert!(json.contains("5"));
    }
}
