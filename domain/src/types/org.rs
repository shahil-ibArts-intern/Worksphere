use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::ids::{OrgId, UserId};

/// Represents an organization (workspace/tenant).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Organization {
    pub id: OrgId,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub logo_url: Option<String>,
    pub plan: String,
    pub max_members: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// Represents a member of an organization.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct OrgMember {
    pub id: Uuid,
    pub org_id: OrgId,
    pub user_id: UserId,
    pub role: OrgRole,
    pub joined_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Role of a user within an organization.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[serde(rename_all = "camelCase")]
#[sqlx(type_name = "org_role", rename_all = "snake_case")]
pub enum OrgRole {
    Guest,
    Member,
    Admin,
    Owner,
}

/// Permission levels for fine-grained access control.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Permission {
    // Channel permissions
    ViewChannels,
    CreateChannels,
    ManageChannels,
    DeleteChannels,

    // Message permissions
    SendMessages,
    EditOwnMessages,
    DeleteOwnMessages,
    DeleteAnyMessages,
    PinMessages,

    // Member permissions
    InviteMembers,
    RemoveMembers,
    ManageRoles,

    // Org permissions
    ManageOrg,
    ManageBilling,
    DeleteOrg,

    // Board permissions
    ViewBoards,
    CreateBoards,
    ManageBoards,
    DeleteBoards,

    // Task permissions
    ViewTasks,
    CreateTasks,
    ManageTasks,
    DeleteTasks,

    // File permissions
    UploadFiles,
    DeleteOwnFiles,
    DeleteAnyFiles,

    // Admin permissions
    ViewAuditLogs,
    ManageWebhooks,
    ManageBots,
}

/// Represents an invitation to join an organization.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Invitation {
    pub id: Uuid,
    pub org_id: OrgId,
    pub email: String,
    pub role: OrgRole,
    pub invited_by: UserId,
    pub token: String,
    pub expires_at: DateTime<Utc>,
    pub accepted_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

/// Request to create an organization.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateOrgRequest {
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
}

/// Request to update an organization.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateOrgRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub logo_url: Option<String>,
}

/// Organization settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrgSettings {
    pub org_id: OrgId,
    pub allow_public_channels: bool,
    pub allow_guests: bool,
    pub default_channel_visibility: String,
    pub retention_days: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
