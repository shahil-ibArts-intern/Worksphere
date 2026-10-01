use async_trait::async_trait;
use sqlx::{PgPool, Row};
use uuid::Uuid;

use domain::error::{AppError, AppResult};
use domain::ids::{OrgId, UserId};
use domain::traits::OrgRepository;
use domain::types::org::{Invitation, OrgMember, OrgRole, Organization};

/// SQLx implementation of the OrgRepository trait.
/// NOTE: Uses runtime queries instead of compile-time checked queries.
/// TODO: Convert to compile-time checked queries when database is available.
#[derive(Clone)]
pub struct SqlxOrgRepository {
    pool: PgPool,
}

impl SqlxOrgRepository {
    /// Creates a new repository instance.
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl OrgRepository for SqlxOrgRepository {
    async fn create(&self, org: &Organization) -> AppResult<Organization> {
        let result = sqlx::query_as::<_, Organization>(
            r#"
            INSERT INTO organizations (id, name, slug, description, logo_url, plan, max_members)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id, name, slug, description, logo_url, plan, max_members, created_at, updated_at, deleted_at
            "#,
        )
        .bind(org.id.as_uuid())
        .bind(&org.name)
        .bind(&org.slug)
        .bind(&org.description)
        .bind(&org.logo_url)
        .bind(&org.plan)
        .bind(org.max_members)
        .fetch_one(&self.pool)
        .await?;

        Ok(result)
    }

    async fn find_by_id(&self, id: OrgId) -> AppResult<Option<Organization>> {
        let result = sqlx::query_as::<_, Organization>(
            r#"
            SELECT id, name, slug, description, logo_url, plan, max_members, created_at, updated_at, deleted_at
            FROM organizations
            WHERE id = $1 AND deleted_at IS NULL
            "#,
        )
        .bind(id.as_uuid())
        .fetch_optional(&self.pool)
        .await?;

        Ok(result)
    }

    async fn find_by_slug(&self, slug: &str) -> AppResult<Option<Organization>> {
        let result = sqlx::query_as::<_, Organization>(
            r#"
            SELECT id, name, slug, description, logo_url, plan, max_members, created_at, updated_at, deleted_at
            FROM organizations
            WHERE slug = $1 AND deleted_at IS NULL
            "#,
        )
        .bind(slug)
        .fetch_optional(&self.pool)
        .await?;

        Ok(result)
    }

    async fn update(&self, org: &Organization) -> AppResult<Organization> {
        let result = sqlx::query_as::<_, Organization>(
            r#"
            UPDATE organizations
            SET name = $2, description = $3, logo_url = $4
            WHERE id = $1 AND deleted_at IS NULL
            RETURNING id, name, slug, description, logo_url, plan, max_members, created_at, updated_at, deleted_at
            "#,
        )
        .bind(org.id.as_uuid())
        .bind(&org.name)
        .bind(&org.description)
        .bind(&org.logo_url)
        .fetch_one(&self.pool)
        .await?;

        Ok(result)
    }

    async fn soft_delete(&self, id: OrgId) -> AppResult<()> {
        let result = sqlx::query(
            r#"
            UPDATE organizations SET deleted_at = NOW() WHERE id = $1 AND deleted_at IS NULL
            "#,
        )
        .bind(id.as_uuid())
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound {
                resource: "Organization".to_string(),
                id: id.to_string(),
            });
        }

        Ok(())
    }

    async fn list_by_user(&self, user_id: UserId) -> AppResult<Vec<Organization>> {
        let result = sqlx::query_as::<_, Organization>(
            r#"
            SELECT o.id, o.name, o.slug, o.description, o.logo_url, o.plan, o.max_members, o.created_at, o.updated_at, o.deleted_at
            FROM organizations o
            INNER JOIN org_members om ON o.id = om.org_id
            WHERE om.user_id = $1 AND o.deleted_at IS NULL
            ORDER BY o.created_at DESC
            "#,
        )
        .bind(user_id.as_uuid())
        .fetch_all(&self.pool)
        .await?;

        Ok(result)
    }

    async fn add_member(&self, member: &OrgMember) -> AppResult<OrgMember> {
        let result = sqlx::query_as::<_, OrgMember>(
            r#"
            INSERT INTO org_members (id, org_id, user_id, role, joined_at)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id, org_id, user_id, role, joined_at, created_at, updated_at
            "#,
        )
        .bind(member.id)
        .bind(member.org_id.as_uuid())
        .bind(member.user_id.as_uuid())
        .bind(member.role as OrgRole)
        .bind(member.joined_at)
        .fetch_one(&self.pool)
        .await?;

        Ok(result)
    }

    async fn find_member(&self, org_id: OrgId, user_id: UserId) -> AppResult<Option<OrgMember>> {
        let result = sqlx::query_as::<_, OrgMember>(
            r#"
            SELECT id, org_id, user_id, role, joined_at, created_at, updated_at
            FROM org_members
            WHERE org_id = $1 AND user_id = $2
            "#,
        )
        .bind(org_id.as_uuid())
        .bind(user_id.as_uuid())
        .fetch_optional(&self.pool)
        .await?;

        Ok(result)
    }

    async fn update_member_role(
        &self,
        org_id: OrgId,
        user_id: UserId,
        role: OrgRole,
    ) -> AppResult<()> {
        let result = sqlx::query(
            r#"
            UPDATE org_members SET role = $3 WHERE org_id = $1 AND user_id = $2
            "#,
        )
        .bind(org_id.as_uuid())
        .bind(user_id.as_uuid())
        .bind(role as OrgRole)
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound {
                resource: "OrgMember".to_string(),
                id: format!("org_id={}, user_id={}", org_id, user_id),
            });
        }

        Ok(())
    }

    async fn remove_member(&self, org_id: OrgId, user_id: UserId) -> AppResult<()> {
        let result = sqlx::query(
            r#"
            DELETE FROM org_members WHERE org_id = $1 AND user_id = $2
            "#,
        )
        .bind(org_id.as_uuid())
        .bind(user_id.as_uuid())
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound {
                resource: "OrgMember".to_string(),
                id: format!("org_id={}, user_id={}", org_id, user_id),
            });
        }

        Ok(())
    }

    async fn list_members(&self, org_id: OrgId) -> AppResult<Vec<OrgMember>> {
        let result = sqlx::query_as::<_, OrgMember>(
            r#"
            SELECT id, org_id, user_id, role, joined_at, created_at, updated_at
            FROM org_members
            WHERE org_id = $1
            ORDER BY joined_at ASC
            "#,
        )
        .bind(org_id.as_uuid())
        .fetch_all(&self.pool)
        .await?;

        Ok(result)
    }

    async fn count_members(&self, org_id: OrgId) -> AppResult<i64> {
        let result = sqlx::query(
            r#"
            SELECT COUNT(*) as count FROM org_members WHERE org_id = $1
            "#,
        )
        .bind(org_id.as_uuid())
        .fetch_one(&self.pool)
        .await?;

        let count: i64 = result.try_get("count").unwrap_or(0);
        Ok(count)
    }

    async fn create_invitation(&self, invitation: &Invitation) -> AppResult<Invitation> {
        let result = sqlx::query_as::<_, Invitation>(
            r#"
            INSERT INTO invitations (id, org_id, email, role, invited_by, token, expires_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id, org_id, email, role, invited_by, token, expires_at, accepted_at, created_at
            "#,
        )
        .bind(invitation.id)
        .bind(invitation.org_id.as_uuid())
        .bind(&invitation.email)
        .bind(invitation.role as OrgRole)
        .bind(invitation.invited_by.as_uuid())
        .bind(&invitation.token)
        .bind(invitation.expires_at)
        .fetch_one(&self.pool)
        .await?;

        Ok(result)
    }

    async fn find_invitation_by_token(&self, token: &str) -> AppResult<Option<Invitation>> {
        let result = sqlx::query_as::<_, Invitation>(
            r#"
            SELECT id, org_id, email, role, invited_by, token, expires_at, accepted_at, created_at
            FROM invitations
            WHERE token = $1
            "#,
        )
        .bind(token)
        .fetch_optional(&self.pool)
        .await?;

        Ok(result)
    }

    async fn accept_invitation(&self, invitation_id: Uuid) -> AppResult<()> {
        let result = sqlx::query(
            r#"
            UPDATE invitations SET accepted_at = NOW() WHERE id = $1
            "#,
        )
        .bind(invitation_id)
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound {
                resource: "Invitation".to_string(),
                id: invitation_id.to_string(),
            });
        }

        Ok(())
    }

    async fn list_invitations(&self, org_id: OrgId) -> AppResult<Vec<Invitation>> {
        let result = sqlx::query_as::<_, Invitation>(
            r#"
            SELECT id, org_id, email, role, invited_by, token, expires_at, accepted_at, created_at
            FROM invitations
            WHERE org_id = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(org_id.as_uuid())
        .fetch_all(&self.pool)
        .await?;

        Ok(result)
    }
}
