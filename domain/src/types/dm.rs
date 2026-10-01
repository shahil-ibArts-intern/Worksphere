//! Direct message types for 1:1 and group DMs.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::ids::{DmConversationId, OrgId, UserId};

/// Represents a direct message conversation (1:1 or group).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct DmConversation {
    pub id: DmConversationId,
    pub org_id: OrgId,
    pub name: Option<String>, // For group DMs
    pub is_group: bool,
    pub created_by: UserId,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// Represents a participant in a DM conversation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct DmParticipant {
    pub id: Uuid,
    pub conversation_id: DmConversationId,
    pub user_id: UserId,
    pub org_id: OrgId,
    pub joined_at: DateTime<Utc>,
    pub last_read_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl DmConversation {
    /// Creates a 1:1 DM conversation between two users.
    pub fn new_direct(org_id: OrgId, user1: UserId, _user2: UserId) -> Self {
        Self {
            id: DmConversationId::new(),
            org_id,
            name: None,
            is_group: false,
            created_by: user1,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            deleted_at: None,
        }
    }

    /// Creates a group DM conversation.
    pub fn new_group(org_id: OrgId, name: String, created_by: UserId) -> Self {
        Self {
            id: DmConversationId::new(),
            org_id,
            name: Some(name),
            is_group: true,
            created_by,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            deleted_at: None,
        }
    }
}
