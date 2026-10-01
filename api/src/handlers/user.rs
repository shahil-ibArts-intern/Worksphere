use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use tracing::instrument;
use uuid::Uuid;
use validator::Validate;

use crate::dto::user::*;
use crate::state::ApiState;
use domain::error::{AppError, AppResult};
use domain::ids::UserId;
use domain::traits::*;

/// Get current user profile.
#[instrument]
pub async fn get_current_user(
    State(state): State<ApiState>,
    Path(user_id): Path<String>,
) -> AppResult<impl axum::response::IntoResponse> {
    let uuid = Uuid::parse_str(&user_id)
        .map_err(|e| AppError::Validation(format!("Invalid user ID: {}", e)))?;
    let user = state
        .services
        .user_repo
        .find_by_id(UserId::from_uuid(uuid))
        .await?
        .ok_or_else(|| AppError::NotFound {
            resource: "User".to_string(),
            id: user_id,
        })?;

    let response = UserProfileResponse {
        id: user.id.to_string(),
        email: user.email,
        username: user.username,
        display_name: user.display_name,
        avatar_url: user.avatar_url,
        status: format!("{:?}", user.status).to_lowercase(),
        role: format!("{:?}", user.role).to_lowercase(),
        last_active_at: user.last_active_at.map(|t| t.to_rfc3339()),
        created_at: user.created_at.to_rfc3339(),
    };

    Ok((StatusCode::OK, Json(response)))
}

/// Update current user profile.
#[instrument]
pub async fn update_profile(
    State(state): State<ApiState>,
    Path(user_id): Path<String>,
    Json(req): Json<UpdateProfileRequest>,
) -> AppResult<impl axum::response::IntoResponse> {
    req.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    let uuid = Uuid::parse_str(&user_id)
        .map_err(|e| AppError::Validation(format!("Invalid user ID: {}", e)))?;
    let user = state
        .services
        .user_repo
        .find_by_id(UserId::from_uuid(uuid))
        .await?
        .ok_or_else(|| AppError::NotFound {
            resource: "User".to_string(),
            id: user_id,
        })?;

    let mut updated_user = user.clone();
    if let Some(name) = &req.display_name {
        updated_user.display_name = name.clone();
    }
    if let Some(avatar) = &req.avatar_url {
        updated_user.avatar_url = Some(avatar.clone());
    }

    let updated = state.services.user_repo.update(&updated_user).await?;

    let response = UserProfileResponse {
        id: updated.id.to_string(),
        email: updated.email,
        username: updated.username,
        display_name: updated.display_name,
        avatar_url: updated.avatar_url,
        status: format!("{:?}", updated.status).to_lowercase(),
        role: format!("{:?}", updated.role).to_lowercase(),
        last_active_at: updated.last_active_at.map(|t| t.to_rfc3339()),
        created_at: updated.created_at.to_rfc3339(),
    };

    Ok((StatusCode::OK, Json(response)))
}

/// Get user by ID.
#[instrument]
pub async fn get_user_by_id(
    State(state): State<ApiState>,
    Path(user_id): Path<String>,
) -> AppResult<impl axum::response::IntoResponse> {
    let uuid = Uuid::parse_str(&user_id)
        .map_err(|e| AppError::Validation(format!("Invalid user ID: {}", e)))?;
    let user = state
        .services
        .user_repo
        .find_by_id(UserId::from_uuid(uuid))
        .await?
        .ok_or_else(|| AppError::NotFound {
            resource: "User".to_string(),
            id: user_id,
        })?;

    let response = UserProfileResponse {
        id: user.id.to_string(),
        email: user.email,
        username: user.username,
        display_name: user.display_name,
        avatar_url: user.avatar_url,
        status: format!("{:?}", user.status).to_lowercase(),
        role: format!("{:?}", user.role).to_lowercase(),
        last_active_at: user.last_active_at.map(|t| t.to_rfc3339()),
        created_at: user.created_at.to_rfc3339(),
    };

    Ok((StatusCode::OK, Json(response)))
}

/// List users in an organization.
#[instrument]
pub async fn list_org_users(
    State(state): State<ApiState>,
    Path(org_id): Path<String>,
) -> AppResult<impl axum::response::IntoResponse> {
    let org_uuid = Uuid::parse_str(&org_id)
        .map_err(|e| AppError::Validation(format!("Invalid org ID: {}", e)))?;

    let users = state
        .services
        .user_repo
        .list_by_org(org_uuid, 1, 50)
        .await?;

    let data: Vec<UserProfileResponse> = users
        .into_iter()
        .map(|user| UserProfileResponse {
            id: user.id.to_string(),
            email: user.email,
            username: user.username,
            display_name: user.display_name,
            avatar_url: user.avatar_url,
            status: format!("{:?}", user.status).to_lowercase(),
            role: format!("{:?}", user.role).to_lowercase(),
            last_active_at: user.last_active_at.map(|t| t.to_rfc3339()),
            created_at: user.created_at.to_rfc3339(),
        })
        .collect();

    let response = UserListResponse {
        data,
        meta: PaginationMeta {
            page: 1,
            per_page: 50,
            total: 0,
        },
    };

    Ok((StatusCode::OK, Json(response)))
}
