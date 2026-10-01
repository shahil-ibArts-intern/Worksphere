//! Channel service — business logic for channel operations.
//!
//! Handles channel CRUD, membership management, and real-time event emission.

use chrono::Utc;
use sqlx::{PgPool, Row};
use tracing::{info, instrument};
use uuid::Uuid;

use domain::error::{AppError, AppResult};
use domain::ids::{ChannelId, OrgId, UserId};
use domain::types::channel::{Channel, ChannelMember, ChannelType};

/// Service for channel-related business logic.
pub struct ChannelService {
    pool: PgPool,
}

impl ChannelService {
    /// Creates a new channel service.
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Creates a new channel and adds the creator as a member.
    #[instrument(skip(self))]
    pub async fn create_channel(
        &self,
        org_id: OrgId,
        name: &str,
        description: Option<&str>,
        is_private: bool,
        created_by: UserId,
    ) -> AppResult<Channel> {
        let channel_id = ChannelId::new();
        let now = Utc::now();
        let channel_type = if is_private {
            ChannelType::Private
        } else {
            ChannelType::Public
        };

        let channel = sqlx::query_as::<_, Channel>(
            r#"
            INSERT INTO channels (id, org_id, name, description, channel_type, is_private, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id, org_id, name, description, channel_type, is_private, created_by, created_at, updated_at, deleted_at
            "#,
        )
        .bind(channel_id.as_uuid())
        .bind(org_id.as_uuid())
        .bind(name)
        .bind(description)
        .bind(channel_type as ChannelType)
        .bind(is_private)
        .bind(created_by.as_uuid())
        .fetch_one(&self.pool)
        .await?;

        // Add creator as channel member
        let member = ChannelMember {
            id: Uuid::now_v7(),
            channel_id,
            user_id: created_by,
            org_id,
            is_admin: true,
            joined_at: now,
            last_read_at: None,
            created_at: now,
            updated_at: now,
        };

        sqlx::query(
            r#"
            INSERT INTO channel_members (id, channel_id, user_id, org_id, is_admin, joined_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
        )
        .bind(member.id)
        .bind(member.channel_id.as_uuid())
        .bind(member.user_id.as_uuid())
        .bind(member.org_id.as_uuid())
        .bind(member.is_admin)
        .bind(member.joined_at)
        .execute(&self.pool)
        .await?;

        info!(channel_id = %channel.id, "Channel created successfully");
        Ok(channel)
    }

    /// Gets a channel by ID.
    #[instrument(skip(self))]
    pub async fn get_channel(&self, channel_id: ChannelId, org_id: OrgId) -> AppResult<Channel> {
        let channel = sqlx::query_as::<_, Channel>(
            r#"
            SELECT id, org_id, name, description, channel_type, is_private, created_by, created_at, updated_at, deleted_at
            FROM channels
            WHERE id = $1 AND org_id = $2 AND deleted_at IS NULL
            "#,
        )
        .bind(channel_id.as_uuid())
        .bind(org_id.as_uuid())
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| AppError::NotFound {
            resource: "Channel".to_string(),
            id: channel_id.to_string(),
        })?;

        Ok(channel)
    }

    /// Lists channels in an organization with pagination.
    #[instrument(skip(self))]
    pub async fn list_channels(
        &self,
        org_id: OrgId,
        page: i64,
        per_page: i64,
    ) -> AppResult<Vec<Channel>> {
        let offset = (page - 1) * per_page;
        let channels = sqlx::query_as::<_, Channel>(
            r#"
            SELECT id, org_id, name, description, channel_type, is_private, created_by, created_at, updated_at, deleted_at
            FROM channels
            WHERE org_id = $1 AND deleted_at IS NULL
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(org_id.as_uuid())
        .bind(per_page)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        Ok(channels)
    }

    /// Searches channels by name in an organization.
    #[instrument(skip(self))]
    pub async fn search_channels(
        &self,
        org_id: OrgId,
        query: &str,
        page: i64,
        per_page: i64,
    ) -> AppResult<Vec<Channel>> {
        let offset = (page - 1) * per_page;
        let search_pattern = format!("%{}%", query);
        let channels = sqlx::query_as::<_, Channel>(
            r#"
            SELECT id, org_id, name, description, channel_type, is_private, created_by, created_at, updated_at, deleted_at
            FROM channels
            WHERE org_id = $1 AND deleted_at IS NULL AND name ILIKE $2
            ORDER BY name ASC
            LIMIT $3 OFFSET $4
            "#,
        )
        .bind(org_id.as_uuid())
        .bind(&search_pattern)
        .bind(per_page)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        Ok(channels)
    }

    /// Updates a channel.
    #[instrument(skip(self))]
    pub async fn update_channel(
        &self,
        channel_id: ChannelId,
        org_id: OrgId,
        name: Option<&str>,
        description: Option<&str>,
        is_private: Option<bool>,
    ) -> AppResult<Channel> {
        let channel = self.get_channel(channel_id, org_id).await?;

        let new_name = name.unwrap_or(&channel.name);
        let new_description = description
            .map(|d| Some(d.to_string()))
            .unwrap_or(channel.description);
        let new_is_private = is_private.unwrap_or(channel.is_private);

        let updated = sqlx::query_as::<_, Channel>(
            r#"
            UPDATE channels
            SET name = $3, description = $4, is_private = $5, updated_at = NOW()
            WHERE id = $1 AND org_id = $2 AND deleted_at IS NULL
            RETURNING id, org_id, name, description, channel_type, is_private, created_by, created_at, updated_at, deleted_at
            "#,
        )
        .bind(channel_id.as_uuid())
        .bind(org_id.as_uuid())
        .bind(new_name)
        .bind(&new_description)
        .bind(new_is_private)
        .fetch_one(&self.pool)
        .await?;

        info!(channel_id = %channel_id, "Channel updated successfully");
        Ok(updated)
    }

    /// Soft-deletes a channel.
    #[instrument(skip(self))]
    pub async fn delete_channel(&self, channel_id: ChannelId, org_id: OrgId) -> AppResult<()> {
        let result = sqlx::query(
            r#"
            UPDATE channels SET deleted_at = NOW() WHERE id = $1 AND org_id = $2 AND deleted_at IS NULL
            "#,
        )
        .bind(channel_id.as_uuid())
        .bind(org_id.as_uuid())
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound {
                resource: "Channel".to_string(),
                id: channel_id.to_string(),
            });
        }

        info!(channel_id = %channel_id, "Channel deleted successfully");
        Ok(())
    }

    /// Adds a member to a channel.
    #[instrument(skip(self))]
    pub async fn join_channel(
        &self,
        channel_id: ChannelId,
        org_id: OrgId,
        user_id: UserId,
    ) -> AppResult<ChannelMember> {
        // Verify channel exists
        self.get_channel(channel_id, org_id).await?;

        let now = Utc::now();
        let member_id = Uuid::now_v7();

        let member = sqlx::query_as::<_, ChannelMember>(
            r#"
            INSERT INTO channel_members (id, channel_id, user_id, org_id, is_admin, joined_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (channel_id, user_id) DO UPDATE SET joined_at = $6
            RETURNING id, channel_id, user_id, org_id, is_admin, joined_at, last_read_at, created_at, updated_at
            "#,
        )
        .bind(member_id)
        .bind(channel_id.as_uuid())
        .bind(user_id.as_uuid())
        .bind(org_id.as_uuid())
        .bind(false)
        .bind(now)
        .fetch_one(&self.pool)
        .await?;

        info!(channel_id = %channel_id, user_id = %user_id, "User joined channel");
        Ok(member)
    }

    /// Removes a member from a channel.
    #[instrument(skip(self))]
    pub async fn leave_channel(&self, channel_id: ChannelId, user_id: UserId) -> AppResult<()> {
        let result = sqlx::query(
            r#"
            DELETE FROM channel_members WHERE channel_id = $1 AND user_id = $2
            "#,
        )
        .bind(channel_id.as_uuid())
        .bind(user_id.as_uuid())
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound {
                resource: "ChannelMember".to_string(),
                id: format!("channel_id={}, user_id={}", channel_id, user_id),
            });
        }

        info!(channel_id = %channel_id, user_id = %user_id, "User left channel");
        Ok(())
    }

    /// Lists members of a channel.
    #[instrument(skip(self))]
    pub async fn list_members(&self, channel_id: ChannelId) -> AppResult<Vec<ChannelMember>> {
        let members = sqlx::query_as::<_, ChannelMember>(
            r#"
            SELECT id, channel_id, user_id, org_id, is_admin, joined_at, last_read_at, created_at, updated_at
            FROM channel_members
            WHERE channel_id = $1
            ORDER BY joined_at ASC
            "#,
        )
        .bind(channel_id.as_uuid())
        .fetch_all(&self.pool)
        .await?;

        Ok(members)
    }

    /// Checks if a user is a member of a channel.
    #[instrument(skip(self))]
    pub async fn is_member(&self, channel_id: ChannelId, user_id: UserId) -> AppResult<bool> {
        let result = sqlx::query(
            r#"
            SELECT COUNT(*) as count FROM channel_members WHERE channel_id = $1 AND user_id = $2
            "#,
        )
        .bind(channel_id.as_uuid())
        .bind(user_id.as_uuid())
        .fetch_one(&self.pool)
        .await?;

        let count: i64 = result.try_get("count").unwrap_or(0);
        Ok(count > 0)
    }
}

// ── Tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_channel_service_creation() {
        // This is a placeholder — real tests would need a database connection
        // Integration tests with #[sqlx::test] would go in tests/
        assert!(true);
    }
}
