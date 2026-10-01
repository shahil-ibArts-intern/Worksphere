use chrono::{DateTime, Utc};
use sqlx::{PgPool, Row};
use uuid::Uuid;

use domain::error::{AppError, AppResult};
use domain::ids::{OrgId, SessionId, UserId};

/// Represents a user session stored in the database.
#[derive(Debug, Clone)]
pub struct Session {
    pub id: SessionId,
    pub user_id: UserId,
    pub org_id: Option<OrgId>,
    pub token_hash: String,
    pub refresh_token_hash: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

/// Session manager for creating, validating, and revoking sessions.
/// NOTE: Uses runtime queries instead of compile-time checked queries.
/// TODO: Convert to compile-time checked queries when database is available.
pub struct SessionManager {
    pool: PgPool,
}

impl SessionManager {
    /// Creates a new session manager.
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Creates a new session for a user.
    #[allow(clippy::too_many_arguments)]
    pub async fn create_session(
        &self,
        user_id: UserId,
        org_id: Option<OrgId>,
        token_hash: &str,
        refresh_token_hash: Option<&str>,
        ip_address: Option<&str>,
        user_agent: Option<&str>,
        expires_at: DateTime<Utc>,
    ) -> AppResult<Session> {
        let session_id = SessionId::new();
        let row = sqlx::query(
            r#"
            INSERT INTO sessions (id, user_id, org_id, token_hash, refresh_token_hash, ip_address, user_agent, expires_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING id, user_id, org_id, token_hash, refresh_token_hash, ip_address, user_agent, expires_at, created_at
            "#,
        )
        .bind(session_id.as_uuid())
        .bind(user_id.as_uuid())
        .bind(org_id.map(|id| id.as_uuid()))
        .bind(token_hash)
        .bind(refresh_token_hash)
        .bind(ip_address)
        .bind(user_agent)
        .bind(expires_at)
        .fetch_one(&self.pool)
        .await?;

        Ok(Session {
            id: SessionId::from_uuid(row.try_get("id")?),
            user_id: UserId::from_uuid(row.try_get("user_id")?),
            org_id: row.try_get::<Uuid, _>("org_id").ok().map(OrgId::from_uuid),
            token_hash: row.try_get("token_hash")?,
            refresh_token_hash: row.try_get("refresh_token_hash")?,
            ip_address: row.try_get("ip_address")?,
            user_agent: row.try_get("user_agent")?,
            expires_at: row.try_get("expires_at")?,
            created_at: row.try_get("created_at")?,
        })
    }

    /// Finds a session by its token hash.
    pub async fn find_by_token_hash(&self, token_hash: &str) -> AppResult<Option<Session>> {
        let row = sqlx::query(
            r#"
            SELECT id, user_id, org_id, token_hash, refresh_token_hash, ip_address, user_agent, expires_at, created_at
            FROM sessions
            WHERE token_hash = $1 AND expires_at > NOW()
            "#,
        )
        .bind(token_hash)
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(row) => Ok(Some(Session {
                id: SessionId::from_uuid(row.try_get("id")?),
                user_id: UserId::from_uuid(row.try_get("user_id")?),
                org_id: row.try_get::<Uuid, _>("org_id").ok().map(OrgId::from_uuid),
                token_hash: row.try_get("token_hash")?,
                refresh_token_hash: row.try_get("refresh_token_hash")?,
                ip_address: row.try_get("ip_address")?,
                user_agent: row.try_get("user_agent")?,
                expires_at: row.try_get("expires_at")?,
                created_at: row.try_get("created_at")?,
            })),
            None => Ok(None),
        }
    }

    /// Revokes a session by its ID.
    pub async fn revoke_session(&self, session_id: SessionId) -> AppResult<()> {
        let result = sqlx::query(
            r#"
            DELETE FROM sessions WHERE id = $1
            "#,
        )
        .bind(session_id.as_uuid())
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound {
                resource: "Session".to_string(),
                id: session_id.to_string(),
            });
        }

        Ok(())
    }

    /// Revokes all sessions for a user.
    pub async fn revoke_all_user_sessions(&self, user_id: UserId) -> AppResult<u64> {
        let result = sqlx::query(
            r#"
            DELETE FROM sessions WHERE user_id = $1
            "#,
        )
        .bind(user_id.as_uuid())
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected())
    }

    /// Cleans up expired sessions.
    pub async fn cleanup_expired_sessions(&self) -> AppResult<u64> {
        let result = sqlx::query(
            r#"
            DELETE FROM sessions WHERE expires_at < NOW()
            "#,
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected())
    }
}
