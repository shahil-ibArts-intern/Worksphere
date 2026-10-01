use sqlx::PgPool;
use tracing::instrument;
use uuid::Uuid;

use domain::error::{AppError, AppResult};
use domain::ids::UserId;
use domain::types::user::{User, UserStatus};

/// Service for user-related business logic.
/// NOTE: Uses runtime queries instead of compile-time checked queries.
/// TODO: Convert to compile-time checked queries when database is available.
pub struct UserService {
    pool: PgPool,
}

impl UserService {
    /// Creates a new user service.
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Gets a user by ID.
    #[instrument(skip(self))]
    pub async fn get_user(&self, user_id: UserId) -> AppResult<User> {
        let user = sqlx::query_as::<_, User>(
            r#"
            SELECT id, email, username, display_name, avatar_url, status, role, last_active_at, created_at, updated_at, deleted_at
            FROM users
            WHERE id = $1 AND deleted_at IS NULL
            "#,
        )
        .bind(user_id.as_uuid())
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| AppError::NotFound {
            resource: "User".to_string(),
            id: user_id.to_string(),
        })?;

        Ok(user)
    }

    /// Updates a user's status.
    #[instrument(skip(self))]
    pub async fn update_status(&self, user_id: UserId, status: UserStatus) -> AppResult<()> {
        let result = sqlx::query(
            r#"
            UPDATE users SET status = $2 WHERE id = $1 AND deleted_at IS NULL
            "#,
        )
        .bind(user_id.as_uuid())
        .bind(status as UserStatus)
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound {
                resource: "User".to_string(),
                id: user_id.to_string(),
            });
        }

        Ok(())
    }

    /// Lists users in an organization.
    #[instrument(skip(self))]
    pub async fn list_org_users(
        &self,
        org_id: Uuid,
        page: i64,
        per_page: i64,
    ) -> AppResult<Vec<User>> {
        let offset = (page - 1) * per_page;
        let users = sqlx::query_as::<_, User>(
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

        Ok(users)
    }
}
