use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use chrono::Utc;
use tracing::{info, instrument};
use uuid::Uuid;
use validator::Validate;

use crate::dto::org::*;
use crate::dto::user::PaginationMeta;
use crate::state::ApiState;
use domain::error::{AppError, AppResult};
use domain::ids::{OrgId, UserId};
use domain::traits::*;
use domain::types::org::{OrgRole, Organization};

/// Create a new organization.
#[instrument]
pub async fn create_org(
    State(state): State<ApiState>,
    Json(req): Json<CreateOrgRequest>,
) -> AppResult<impl axum::response::IntoResponse> {
    req.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    let org = Organization {
        id: OrgId::new(),
        name: req.name.clone(),
        slug: req.slug.clone(),
        description: req.description.clone(),
        logo_url: None,
        plan: "free".to_string(),
        max_members: 50,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        deleted_at: None,
    };

    state.services.org_repo.create(&org).await?;

    info!(org_id = %org.id, "Organization created successfully");

    let response = OrgResponse {
        id: org.id.to_string(),
        name: org.name,
        slug: org.slug,
        description: org.description,
        logo_url: org.logo_url,
        plan: org.plan,
        max_members: org.max_members,
        created_at: org.created_at.to_rfc3339(),
    };

    Ok((StatusCode::CREATED, Json(response)))
}

/// Get organization by ID.
#[instrument]
pub async fn get_org(
    State(state): State<ApiState>,
    Path(org_id): Path<String>,
) -> AppResult<impl axum::response::IntoResponse> {
    let uuid = Uuid::parse_str(&org_id)
        .map_err(|e| AppError::Validation(format!("Invalid org ID: {}", e)))?;
    let org = state
        .services
        .org_repo
        .find_by_id(OrgId::from_uuid(uuid))
        .await?
        .ok_or_else(|| AppError::NotFound {
            resource: "Organization".to_string(),
            id: org_id,
        })?;

    let response = OrgResponse {
        id: org.id.to_string(),
        name: org.name,
        slug: org.slug,
        description: org.description,
        logo_url: org.logo_url,
        plan: org.plan,
        max_members: org.max_members,
        created_at: org.created_at.to_rfc3339(),
    };

    Ok((StatusCode::OK, Json(response)))
}

/// Update organization.
#[instrument]
pub async fn update_org(
    State(state): State<ApiState>,
    Path(org_id): Path<String>,
    Json(req): Json<UpdateOrgRequest>,
) -> AppResult<impl axum::response::IntoResponse> {
    req.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    let uuid = Uuid::parse_str(&org_id)
        .map_err(|e| AppError::Validation(format!("Invalid org ID: {}", e)))?;
    let org = state
        .services
        .org_repo
        .find_by_id(OrgId::from_uuid(uuid))
        .await?
        .ok_or_else(|| AppError::NotFound {
            resource: "Organization".to_string(),
            id: org_id,
        })?;

    let mut updated_org = org.clone();
    if let Some(name) = &req.name {
        updated_org.name = name.clone();
    }
    if let Some(desc) = &req.description {
        updated_org.description = Some(desc.clone());
    }
    if let Some(logo) = &req.logo_url {
        updated_org.logo_url = Some(logo.clone());
    }

    let updated = state.services.org_repo.update(&updated_org).await?;

    let response = OrgResponse {
        id: updated.id.to_string(),
        name: updated.name,
        slug: updated.slug,
        description: updated.description,
        logo_url: updated.logo_url,
        plan: updated.plan,
        max_members: updated.max_members,
        created_at: updated.created_at.to_rfc3339(),
    };

    Ok((StatusCode::OK, Json(response)))
}

/// Delete organization (owner only).
#[instrument]
pub async fn delete_org(
    State(state): State<ApiState>,
    Path(org_id): Path<String>,
) -> AppResult<impl axum::response::IntoResponse> {
    let uuid = Uuid::parse_str(&org_id)
        .map_err(|e| AppError::Validation(format!("Invalid org ID: {}", e)))?;
    state
        .services
        .org_repo
        .soft_delete(OrgId::from_uuid(uuid))
        .await?;

    Ok(StatusCode::NO_CONTENT)
}

/// List organization members.
#[instrument]
pub async fn list_members(
    State(state): State<ApiState>,
    Path(org_id): Path<String>,
) -> AppResult<impl axum::response::IntoResponse> {
    let uuid = Uuid::parse_str(&org_id)
        .map_err(|e| AppError::Validation(format!("Invalid org ID: {}", e)))?;
    let members = state
        .services
        .org_repo
        .list_members(OrgId::from_uuid(uuid))
        .await?;

    let data: Vec<OrgMemberResponse> = members
        .into_iter()
        .map(|m| OrgMemberResponse {
            id: m.id.to_string(),
            user_id: m.user_id.to_string(),
            role: format!("{:?}", m.role).to_lowercase(),
            joined_at: m.joined_at.to_rfc3339(),
        })
        .collect();

    let response = OrgMemberListResponse {
        data,
        meta: PaginationMeta {
            page: 1,
            per_page: 50,
            total: 0,
        },
    };

    Ok((StatusCode::OK, Json(response)))
}

/// Invite a user to the organization.
#[instrument]
pub async fn invite_user(
    State(_state): State<ApiState>,
    Path(_org_id): Path<String>,
    Json(req): Json<InviteUserRequest>,
) -> AppResult<impl axum::response::IntoResponse> {
    req.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    // In a full implementation, this would create an invitation and send an email
    let response = InvitationResponse {
        id: Uuid::new_v4().to_string(),
        email: req.email,
        role: req.role.unwrap_or_else(|| "member".to_string()),
        expires_at: (Utc::now() + chrono::Duration::days(7)).to_rfc3339(),
        created_at: Utc::now().to_rfc3339(),
    };

    Ok((StatusCode::CREATED, Json(response)))
}

/// Change a member's role.
#[instrument]
pub async fn change_member_role(
    State(state): State<ApiState>,
    Path((org_id, user_id)): Path<(String, String)>,
    Json(req): Json<ChangeRoleRequest>,
) -> AppResult<impl axum::response::IntoResponse> {
    let org_uuid = Uuid::parse_str(&org_id)
        .map_err(|e| AppError::Validation(format!("Invalid org ID: {}", e)))?;
    let user_uuid = Uuid::parse_str(&user_id)
        .map_err(|e| AppError::Validation(format!("Invalid user ID: {}", e)))?;

    let role = match req.role.as_str() {
        "owner" => OrgRole::Owner,
        "admin" => OrgRole::Admin,
        "member" => OrgRole::Member,
        "guest" => OrgRole::Guest,
        _ => return Err(AppError::Validation("Invalid role".to_string())),
    };

    state
        .services
        .org_repo
        .update_member_role(
            OrgId::from_uuid(org_uuid),
            UserId::from_uuid(user_uuid),
            role,
        )
        .await?;

    Ok(StatusCode::NO_CONTENT)
}

/// Remove a member from the organization.
#[instrument]
pub async fn remove_member(
    State(state): State<ApiState>,
    Path((org_id, user_id)): Path<(String, String)>,
) -> AppResult<impl axum::response::IntoResponse> {
    let org_uuid = Uuid::parse_str(&org_id)
        .map_err(|e| AppError::Validation(format!("Invalid org ID: {}", e)))?;
    let user_uuid = Uuid::parse_str(&user_id)
        .map_err(|e| AppError::Validation(format!("Invalid user ID: {}", e)))?;

    state
        .services
        .org_repo
        .remove_member(OrgId::from_uuid(org_uuid), UserId::from_uuid(user_uuid))
        .await?;

    Ok(StatusCode::NO_CONTENT)
}
