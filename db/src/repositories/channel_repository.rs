use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{PgPool, Row};

use domain::error::{AppError, AppResult};
use domain::ids::{ChannelId, OrgId, UserId};
use domain::traits::ChannelRepository;
use domain::types::channel::{Channel, ChannelMember};

/// SQLx implementation of the ChannelRepository trait.
/// NOTE: Uses runtime queries instead of compile-time checked queries.
/// TODO: Convert to compile-time checked queries when database is available.
#[derive(Clone)]
pub struct SqlxChannelRepository {
    pool: PgPool,
}

impl SqlxChannelRepository {
    /// Creates a new repository instance.
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ChannelRepository for SqlxChannelRepository {
    async fn create(&self, channel: &Channel) -> AppResult<Channel> {
        let result = sqlx::query_as::<_, Channel>(
            r#"
            INSERT INTO channels (id, org_id, name, description, channel_type, is_private, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id, org_id, name, description, channel_type, is_private, created_by, created_at, updated_at, deleted_at
            "#,
        )
        .bind(channel.id.as_uuid())
        .bind(channel.org_id.as_uuid())
        .bind(&channel.name)
        .bind(&channel.description)
        .bind(channel.channel_type as domain::types::channel::ChannelType)
        .bind(channel.is_private)
        .bind(channel.created_by.as_uuid())
        .fetch_one(&self.pool)
        .await?;

        Ok(result)
    }

    async fn find_by_id(&self, id: ChannelId, org_id: OrgId) -> AppResult<Option<Channel>> {
        let result = sqlx::query_as::<_, Channel>(
            r#"
            SELECT id, org_id, name, description, channel_type, is_private, created_by, created_at, updated_at, deleted_at
            FROM channels
            WHERE id = $1 AND org_id = $2 AND deleted_at IS NULL
            "#,
        )
        .bind(id.as_uuid())
        .bind(org_id.as_uuid())
        .fetch_optional(&self.pool)
        .await?;

        Ok(result)
    }

    async fn update(&self, channel: &Channel) -> AppResult<Channel> {
        let result = sqlx::query_as::<_, Channel>(
            r#"
            UPDATE channels
            SET name = $3, description = $4, is_private = $5
            WHERE id = $1 AND org_id = $2 AND deleted_at IS NULL
            RETURNING id, org_id, name, description, channel_type, is_private, created_by, created_at, updated_at, deleted_at
            "#,
        )
        .bind(channel.id.as_uuid())
        .bind(channel.org_id.as_uuid())
        .bind(&channel.name)
        .bind(&channel.description)
        .bind(channel.is_private)
        .fetch_one(&self.pool)
        .await?;

        Ok(result)
    }

    async fn soft_delete(&self, id: ChannelId, org_id: OrgId) -> AppResult<()> {
        let result = sqlx::query(
            r#"
            UPDATE channels SET deleted_at = NOW() WHERE id = $1 AND org_id = $2 AND deleted_at IS NULL
            "#,
        )
        .bind(id.as_uuid())
        .bind(org_id.as_uuid())
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound {
                resource: "Channel".to_string(),
                id: id.to_string(),
            });
        }

        Ok(())
    }

    async fn list_by_org(
        &self,
        org_id: OrgId,
        page: i64,
        per_page: i64,
    ) -> AppResult<Vec<Channel>> {
        let offset = (page - 1) * per_page;
        let result = sqlx::query_as::<_, Channel>(
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

        Ok(result)
    }

    async fn list_by_user(
        &self,
        org_id: OrgId,
        user_id: UserId,
        page: i64,
        per_page: i64,
    ) -> AppResult<Vec<Channel>> {
        let offset = (page - 1) * per_page;
        let result = sqlx::query_as::<_, Channel>(
            r#"
            SELECT c.id, c.org_id, c.name, c.description, c.channel_type, c.is_private, c.created_by, c.created_at, c.updated_at, c.deleted_at
            FROM channels c
            INNER JOIN channel_members cm ON c.id = cm.channel_id
            WHERE c.org_id = $1 AND cm.user_id = $2 AND c.deleted_at IS NULL
            ORDER BY c.created_at DESC
            LIMIT $3 OFFSET $4
            "#,
        )
        .bind(org_id.as_uuid())
        .bind(user_id.as_uuid())
        .bind(per_page)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        Ok(result)
    }

    async fn add_member(&self, member: &ChannelMember) -> AppResult<ChannelMember> {
        let result = sqlx::query_as::<_, ChannelMember>(
            r#"
            INSERT INTO channel_members (id, channel_id, user_id, org_id, is_admin, joined_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id, channel_id, user_id, org_id, is_admin, joined_at, last_read_at, created_at, updated_at
            "#,
        )
        .bind(member.id)
        .bind(member.channel_id.as_uuid())
        .bind(member.user_id.as_uuid())
        .bind(member.org_id.as_uuid())
        .bind(member.is_admin)
        .bind(member.joined_at)
        .fetch_one(&self.pool)
        .await?;

        Ok(result)
    }

    async fn remove_member(&self, channel_id: ChannelId, user_id: UserId) -> AppResult<()> {
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

        Ok(())
    }

    async fn list_members(&self, channel_id: ChannelId) -> AppResult<Vec<ChannelMember>> {
        let result = sqlx::query_as::<_, ChannelMember>(
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

        Ok(result)
    }

    async fn is_member(&self, channel_id: ChannelId, user_id: UserId) -> AppResult<bool> {
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

    async fn update_last_read(
        &self,
        channel_id: ChannelId,
        user_id: UserId,
        last_read_at: DateTime<Utc>,
    ) -> AppResult<()> {
        sqlx::query(
            r#"
            UPDATE channel_members SET last_read_at = $3 WHERE channel_id = $1 AND user_id = $2
            "#,
        )
        .bind(channel_id.as_uuid())
        .bind(user_id.as_uuid())
        .bind(last_read_at)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn search_by_name(
        &self,
        org_id: OrgId,
        query: &str,
        page: i64,
        per_page: i64,
    ) -> AppResult<Vec<Channel>> {
        let offset = (page - 1) * per_page;
        let search_pattern = format!("%{}%", query);
        let result = sqlx::query_as::<_, Channel>(
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

        Ok(result)
    }
}
