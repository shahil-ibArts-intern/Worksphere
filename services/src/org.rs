use chrono::Utc;
use sqlx::PgPool;
use tracing::instrument;
use uuid::Uuid;

use domain::error::{AppError, AppResult};
use domain::ids::{OrgId, UserId};
use domain::types::org::{OrgMember, OrgRole, Organization};

/// Service for organization-related business logic.
/// NOTE: Uses runtime queries instead of compile-time checked queries.
/// TODO: Convert to compile-time checked queries when database is available.
pub struct OrgService {
    pool: PgPool,
}

impl OrgService {
    /// Creates a new org service.
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Creates a new organization with the creator as owner.
    #[instrument(skip(self))]
    pub async fn create_org(
        &self,
        name: &str,
        slug: &str,
        description: Option<&str>,
        creator_id: UserId,
    ) -> AppResult<Organization> {
        let org_id = OrgId::new();
        let now = Utc::now();

        let org = sqlx::query_as::<_, Organization>(
            r#"
            INSERT INTO organizations (id, name, slug, description, plan, max_members)
            VALUES ($1, $2, $3, $4, 'free', 50)
            RETURNING id, name, slug, description, logo_url, plan, max_members, created_at, updated_at, deleted_at
            "#,
        )
        .bind(org_id.as_uuid())
        .bind(name)
        .bind(slug)
        .bind(description)
        .fetch_one(&self.pool)
        .await?;

        // Add creator as owner
        let member = OrgMember {
            id: Uuid::now_v7(),
            org_id,
            user_id: creator_id,
            role: OrgRole::Owner,
            joined_at: now,
            created_at: now,
            updated_at: now,
        };

        sqlx::query(
            r#"
            INSERT INTO org_members (id, org_id, user_id, role, joined_at)
            VALUES ($1, $2, $3, $4, $5)
            "#,
        )
        .bind(member.id)
        .bind(member.org_id.as_uuid())
        .bind(member.user_id.as_uuid())
        .bind(member.role as OrgRole)
        .bind(member.joined_at)
        .execute(&self.pool)
        .await?;

        Ok(org)
    }

    /// Gets an organization by ID.
    #[instrument(skip(self))]
    pub async fn get_org(&self, org_id: OrgId) -> AppResult<Organization> {
        let org = sqlx::query_as::<_, Organization>(
            r#"
            SELECT id, name, slug, description, logo_url, plan, max_members, created_at, updated_at, deleted_at
            FROM organizations
            WHERE id = $1 AND deleted_at IS NULL
            "#,
        )
        .bind(org_id.as_uuid())
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| AppError::NotFound {
            resource: "Organization".to_string(),
            id: org_id.to_string(),
        })?;

        Ok(org)
    }

    /// Verifies a user is a member of an organization.
    #[instrument(skip(self))]
    pub async fn verify_membership(&self, org_id: OrgId, user_id: UserId) -> AppResult<OrgMember> {
        let member = sqlx::query_as::<_, OrgMember>(
            r#"
            SELECT id, org_id, user_id, role, joined_at, created_at, updated_at
            FROM org_members
            WHERE org_id = $1 AND user_id = $2
            "#,
        )
        .bind(org_id.as_uuid())
        .bind(user_id.as_uuid())
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| {
            AppError::Authorization("You are not a member of this organization".to_string())
        })?;

        Ok(member)
    }
}
