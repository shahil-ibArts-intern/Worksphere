use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::ids::{BoardId, OrgId, UserId};

/// Represents a task board.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Board {
    pub id: BoardId,
    pub org_id: OrgId,
    pub name: String,
    pub description: Option<String>,
    pub visibility: BoardVisibility,
    pub created_by: UserId,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// Board visibility level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[serde(rename_all = "camelCase")]
#[sqlx(type_name = "board_visibility", rename_all = "snake_case")]
pub enum BoardVisibility {
    Public,
    Private,
    OrgWide,
}

/// Represents a member of a board.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct BoardMember {
    pub id: Uuid,
    pub board_id: BoardId,
    pub user_id: UserId,
    pub org_id: OrgId,
    pub can_edit: bool,
    pub joined_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
