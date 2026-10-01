use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{PgPool, Row};
use uuid::Uuid;

use domain::error::{AppError, AppResult};
use domain::ids::UserId;
use domain::traits::UserRepository;
use domain::types::user::{User, UserSettings, UserStatus};

/// SQLx implementation of the UserRepository trait.
/// NOTE: Uses runtime queries (sqlx::query_as) instead of compile-time checked queries
/// (sqlx::query_as!) because no database is available at compile time.
/// TODO: Convert to compile-time checked queries when database is available.
#[derive(Clone)]
pub struct SqlxUserRepository {
    pool: PgPool,
}

impl SqlxUserRepository {
    /// Creates a new repository instance.
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl UserRepository for SqlxUserRepository {
    async fn create(&self, user: &User) -> AppResult<User> {
        let result = sqlx::query_as::<_, User>(
            r#"
            INSERT INTO users (id, email, username, display_name, avatar_url, status, role, password_hash, last_active_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING id, email, username, display_name, avatar_url, status, role, last_active_at, created_at, updated_at, deleted_at
            "#,
        )
        .bind(user.id.as_uuid())
        .bind(&user.email)
        .bind(&user.username)
        .bind(&user.display_name)
        .bind(&user.avatar_url)
        .bind(user.status as UserStatus)
        .bind(user.role as domain::types::user::UserRole)
        .bind("") // password_hash - stored separately
        .bind(user.last_active_at)
        .fetch_one(&self.pool)
        .await?;

        Ok(result)
    }

    async fn find_by_id(&self, id: UserId) -> AppResult<Option<User>> {
        let result = sqlx::query_as::<_, User>(
            r#"
            SELECT id, email, username, display_name, avatar_url, status, role, last_active_at, created_at, updated_at, deleted_at
            FROM users
            WHERE id = $1 AND deleted_at IS NULL
            "#,
        )
        .bind(id.as_uuid())
        .fetch_optional(&self.pool)
        .await?;

        Ok(result)
    }

    async fn find_by_email(&self, email: &str) -> AppResult<Option<User>> {
        let result = sqlx::query_as::<_, User>(
            r#"
            SELECT id, email, username, display_name, avatar_url, status, role, last_active_at, created_at, updated_at, deleted_at
            FROM users
            WHERE email = $1 AND deleted_at IS NULL
            "#,
        )
        .bind(email)
        .fetch_optional(&self.pool)
        .await?;

        Ok(result)
    }

    async fn find_by_username(&self, username: &str) -> AppResult<Option<User>> {
        let result = sqlx::query_as::<_, User>(
            r#"
            SELECT id, email, username, display_name, avatar_url, status, role, last_active_at, created_at, updated_at, deleted_at
            FROM users
            WHERE username = $1 AND deleted_at IS NULL
            "#,
        )
        .bind(username)
        .fetch_optional(&self.pool)
        .await?;

        Ok(result)
    }

    async fn update(&self, user: &User) -> AppResult<User> {
        let result = sqlx::query_as::<_, User>(
            r#"
            UPDATE users
            SET email = $2, username = $3, display_name = $4, avatar_url = $5, status = $6, role = $7, last_active_at = $8
            WHERE id = $1 AND deleted_at IS NULL
            RETURNING id, email, username, display_name, avatar_url, status, role, last_active_at, created_at, updated_at, deleted_at
            "#,
        )
        .bind(user.id.as_uuid())
        .bind(&user.email)
        .bind(&user.username)
        .bind(&user.display_name)
        .bind(&user.avatar_url)
        .bind(user.status as UserStatus)
        .bind(user.role as domain::types::user::UserRole)
        .bind(user.last_active_at)
        .fetch_one(&self.pool)
        .await?;

        Ok(result)
    }

    async fn update_status(&self, id: UserId, status: UserStatus) -> AppResult<()> {
        let result = sqlx::query(
            r#"
            UPDATE users SET status = $2 WHERE id = $1 AND deleted_at IS NULL
            "#,
        )
        .bind(id.as_uuid())
        .bind(status as UserStatus)
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound {
                resource: "User".to_string(),
                id: id.to_string(),
            });
        }

        Ok(())
    }

    async fn update_last_active(&self, id: UserId, last_active: DateTime<Utc>) -> AppResult<()> {
        sqlx::query(
            r#"
            UPDATE users SET last_active_at = $2 WHERE id = $1 AND deleted_at IS NULL
            "#,
        )
        .bind(id.as_uuid())
        .bind(last_active)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn list_by_org(&self, org_id: Uuid, page: i64, per_page: i64) -> AppResult<Vec<User>> {
        let offset = (page - 1) * per_page;
        let result = sqlx::query_as::<_, User>(
            r#"
            SELECT u.id, u.email, u.username, u.display_name, u.avatar_url, u.status, u.role, u.last_active_at, u.created_at, u.updated_at, u.deleted_at
            FROM users u
            INNER JOIN org_members om ON u.id = om.user_id
            WHERE om.org_id = $1 AND u.deleted_at IS NULL
            ORDER BY u.created_at DESC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(org_id)
        .bind(per_page)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        Ok(result)
    }

    async fn count_by_org(&self, org_id: Uuid) -> AppResult<i64> {
        let result = sqlx::query(
            r#"
            SELECT COUNT(*) as count
            FROM users u
            INNER JOIN org_members om ON u.id = om.user_id
            WHERE om.org_id = $1 AND u.deleted_at IS NULL
            "#,
        )
        .bind(org_id)
        .fetch_one(&self.pool)
        .await?;

        let count: i64 = result.try_get("count").unwrap_or(0);
        Ok(count)
    }

    async fn search(
        &self,
        org_id: Uuid,
        query: &str,
        page: i64,
        per_page: i64,
    ) -> AppResult<Vec<User>> {
        let offset = (page - 1) * per_page;
        let search_pattern = format!("%{}%", query);
        let result = sqlx::query_as::<_, User>(
            r#"
            SELECT u.id, u.email, u.username, u.display_name, u.avatar_url, u.status, u.role, u.last_active_at, u.created_at, u.updated_at, u.deleted_at
            FROM users u
            INNER JOIN org_members om ON u.id = om.user_id
            WHERE om.org_id = $1 AND u.deleted_at IS NULL
            AND (u.display_name ILIKE $2 OR u.email ILIKE $2 OR u.username ILIKE $2)
            ORDER BY u.created_at DESC
            LIMIT $3 OFFSET $4
            "#,
        )
        .bind(org_id)
        .bind(&search_pattern)
        .bind(per_page)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        Ok(result)
    }

    async fn get_settings(&self, _user_id: UserId) -> AppResult<Option<UserSettings>> {
        // User settings stored in a separate table or JSONB column
        // For now, return None as settings table is not in initial migration
        Ok(None)
    }

    async fn update_settings(&self, _settings: &UserSettings) -> AppResult<UserSettings> {
        Err(AppError::Internal(
            "Settings not yet implemented".to_string(),
        ))
    }

    async fn soft_delete(&self, id: UserId, deleted_by: UserId) -> AppResult<()> {
        let result = sqlx::query(
            r#"
            UPDATE users SET deleted_at = NOW(), deleted_by = $2 WHERE id = $1 AND deleted_at IS NULL
            "#,
        )
        .bind(id.as_uuid())
        .bind(deleted_by.as_uuid())
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound {
                resource: "User".to_string(),
                id: id.to_string(),
            });
        }

        Ok(())
    }
}
