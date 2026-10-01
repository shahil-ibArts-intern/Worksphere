use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use chrono::Utc;
use serde::Deserialize;
use tracing::{info, instrument};
use uuid::Uuid;
use validator::Validate;

use crate::dto::channel::*;
use crate::dto::user::PaginationMeta;
use crate::state::ApiState;
use domain::error::{AppError, AppResult};
use domain::ids::{ChannelId, OrgId, UserId};
use domain::traits::*;
use domain::types::channel::{Channel, ChannelMember, ChannelType};

/// List channels in an organization.
#[instrument]
pub async fn list_channels(
    State(state): State<ApiState>,
    Path(org_id): Path<String>,
) -> AppResult<impl axum::response::IntoResponse> {
    let uuid = Uuid::parse_str(&org_id)
        .map_err(|e| AppError::Validation(format!("Invalid org ID: {}", e)))?;
    let channels = state
        .services
        .channel_repo
        .list_by_org(OrgId::from_uuid(uuid), 1, 50)
        .await?;

    let data: Vec<ChannelResponse> = channels
        .into_iter()
        .map(|c| ChannelResponse {
            id: c.id.to_string(),
            org_id: c.org_id.to_string(),
            name: c.name,
            description: c.description,
            channel_type: format!("{:?}", c.channel_type).to_lowercase(),
            is_private: c.is_private,
            created_by: c.created_by.to_string(),
            created_at: c.created_at.to_rfc3339(),
            updated_at: c.updated_at.to_rfc3339(),
        })
        .collect();

    let response = ChannelListResponse {
        data,
        meta: PaginationMeta {
            page: 1,
            per_page: 50,
            total: 0,
        },
    };

    Ok((StatusCode::OK, Json(response)))
}

/// Create a new channel.
#[instrument]
pub async fn create_channel(
    State(state): State<ApiState>,
    Path(org_id): Path<String>,
    Json(req): Json<CreateChannelRequest>,
) -> AppResult<impl axum::response::IntoResponse> {
    req.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    let uuid = Uuid::parse_str(&org_id)
        .map_err(|e| AppError::Validation(format!("Invalid org ID: {}", e)))?;

    let channel = Channel {
        id: ChannelId::new(),
        org_id: OrgId::from_uuid(uuid),
        name: req.name.clone(),
        description: req.description.clone(),
        channel_type: if req.is_private {
            ChannelType::Private
        } else {
            ChannelType::Public
        },
        is_private: req.is_private,
        created_by: UserId::new(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
        deleted_at: None,
    };

    state.services.channel_repo.create(&channel).await?;

    info!(channel_id = %channel.id, "Channel created successfully");

    let response = ChannelResponse {
        id: channel.id.to_string(),
        org_id: channel.org_id.to_string(),
        name: channel.name,
        description: channel.description,
        channel_type: format!("{:?}", channel.channel_type).to_lowercase(),
        is_private: channel.is_private,
        created_by: channel.created_by.to_string(),
        created_at: channel.created_at.to_rfc3339(),
        updated_at: channel.updated_at.to_rfc3339(),
    };

    Ok((StatusCode::CREATED, Json(response)))
}

/// Get channel by ID.
#[instrument]
pub async fn get_channel(
    State(state): State<ApiState>,
    Path((org_id, channel_id)): Path<(String, String)>,
) -> AppResult<impl axum::response::IntoResponse> {
    let org_uuid = Uuid::parse_str(&org_id)
        .map_err(|e| AppError::Validation(format!("Invalid org ID: {}", e)))?;
    let channel_uuid = Uuid::parse_str(&channel_id)
        .map_err(|e| AppError::Validation(format!("Invalid channel ID: {}", e)))?;

    let channel = state
        .services
        .channel_repo
        .find_by_id(
            ChannelId::from_uuid(channel_uuid),
            OrgId::from_uuid(org_uuid),
        )
        .await?
        .ok_or_else(|| AppError::NotFound {
            resource: "Channel".to_string(),
            id: channel_id,
        })?;

    let response = ChannelResponse {
        id: channel.id.to_string(),
        org_id: channel.org_id.to_string(),
        name: channel.name,
        description: channel.description,
        channel_type: format!("{:?}", channel.channel_type).to_lowercase(),
        is_private: channel.is_private,
        created_by: channel.created_by.to_string(),
        created_at: channel.created_at.to_rfc3339(),
        updated_at: channel.updated_at.to_rfc3339(),
    };

    Ok((StatusCode::OK, Json(response)))
}

/// Update channel.
#[instrument]
pub async fn update_channel(
    State(state): State<ApiState>,
    Path((org_id, channel_id)): Path<(String, String)>,
    Json(req): Json<UpdateChannelRequest>,
) -> AppResult<impl axum::response::IntoResponse> {
    req.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    let org_uuid = Uuid::parse_str(&org_id)
        .map_err(|e| AppError::Validation(format!("Invalid org ID: {}", e)))?;
    let channel_uuid = Uuid::parse_str(&channel_id)
        .map_err(|e| AppError::Validation(format!("Invalid channel ID: {}", e)))?;

    let channel = state
        .services
        .channel_repo
        .find_by_id(
            ChannelId::from_uuid(channel_uuid),
            OrgId::from_uuid(org_uuid),
        )
        .await?
        .ok_or_else(|| AppError::NotFound {
            resource: "Channel".to_string(),
            id: channel_id,
        })?;

    let mut updated_channel = channel.clone();
    if let Some(name) = &req.name {
        updated_channel.name = name.clone();
    }
    if let Some(desc) = &req.description {
        updated_channel.description = Some(desc.clone());
    }
    if let Some(is_private) = req.is_private {
        updated_channel.is_private = is_private;
    }

    let updated = state.services.channel_repo.update(&updated_channel).await?;

    let response = ChannelResponse {
        id: updated.id.to_string(),
        org_id: updated.org_id.to_string(),
        name: updated.name,
        description: updated.description,
        channel_type: format!("{:?}", updated.channel_type).to_lowercase(),
        is_private: updated.is_private,
        created_by: updated.created_by.to_string(),
        created_at: updated.created_at.to_rfc3339(),
        updated_at: updated.updated_at.to_rfc3339(),
    };

    Ok((StatusCode::OK, Json(response)))
}

/// Delete channel.
#[instrument]
pub async fn delete_channel(
    State(state): State<ApiState>,
    Path((org_id, channel_id)): Path<(String, String)>,
) -> AppResult<impl axum::response::IntoResponse> {
    let org_uuid = Uuid::parse_str(&org_id)
        .map_err(|e| AppError::Validation(format!("Invalid org ID: {}", e)))?;
    let channel_uuid = Uuid::parse_str(&channel_id)
        .map_err(|e| AppError::Validation(format!("Invalid channel ID: {}", e)))?;

    state
        .services
        .channel_repo
        .soft_delete(
            ChannelId::from_uuid(channel_uuid),
            OrgId::from_uuid(org_uuid),
        )
        .await?;

    Ok(StatusCode::NO_CONTENT)
}

/// Join a channel.
#[instrument]
pub async fn join_channel(
    State(state): State<ApiState>,
    Path((org_id, channel_id)): Path<(String, String)>,
) -> AppResult<impl axum::response::IntoResponse> {
    let org_uuid = Uuid::parse_str(&org_id)
        .map_err(|e| AppError::Validation(format!("Invalid org ID: {}", e)))?;
    let channel_uuid = Uuid::parse_str(&channel_id)
        .map_err(|e| AppError::Validation(format!("Invalid channel ID: {}", e)))?;

    // In production, user_id would come from the authenticated user context
    let user_id = UserId::new();

    let member = state
        .services
        .channel_repo
        .add_member(&ChannelMember {
            id: Uuid::now_v7(),
            channel_id: ChannelId::from_uuid(channel_uuid),
            user_id,
            org_id: OrgId::from_uuid(org_uuid),
            is_admin: false,
            joined_at: Utc::now(),
            last_read_at: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        })
        .await?;

    info!(channel_id = %channel_id, user_id = %user_id, "User joined channel");

    let response = ChannelMemberResponse {
        id: member.id.to_string(),
        user_id: member.user_id.to_string(),
        is_admin: member.is_admin,
        joined_at: member.joined_at.to_rfc3339(),
    };

    Ok((StatusCode::CREATED, Json(response)))
}

/// Leave a channel.
#[instrument]
pub async fn leave_channel(
    State(state): State<ApiState>,
    Path((org_id, channel_id)): Path<(String, String)>,
) -> AppResult<impl axum::response::IntoResponse> {
    let _org_uuid = Uuid::parse_str(&org_id)
        .map_err(|e| AppError::Validation(format!("Invalid org ID: {}", e)))?;
    let channel_uuid = Uuid::parse_str(&channel_id)
        .map_err(|e| AppError::Validation(format!("Invalid channel ID: {}", e)))?;

    // In production, user_id would come from the authenticated user context
    let user_id = UserId::new();

    state
        .services
        .channel_repo
        .remove_member(ChannelId::from_uuid(channel_uuid), user_id)
        .await?;

    Ok(StatusCode::NO_CONTENT)
}

/// List channel members.
#[instrument]
pub async fn list_channel_members(
    State(state): State<ApiState>,
    Path((org_id, channel_id)): Path<(String, String)>,
) -> AppResult<impl axum::response::IntoResponse> {
    let _org_uuid = Uuid::parse_str(&org_id)
        .map_err(|e| AppError::Validation(format!("Invalid org ID: {}", e)))?;
    let channel_uuid = Uuid::parse_str(&channel_id)
        .map_err(|e| AppError::Validation(format!("Invalid channel ID: {}", e)))?;

    let members = state
        .services
        .channel_repo
        .list_members(ChannelId::from_uuid(channel_uuid))
        .await?;

    let data: Vec<ChannelMemberResponse> = members
        .into_iter()
        .map(|m| ChannelMemberResponse {
            id: m.id.to_string(),
            user_id: m.user_id.to_string(),
            is_admin: m.is_admin,
            joined_at: m.joined_at.to_rfc3339(),
        })
        .collect();

    let total = data.len() as i64;
    let response = ChannelMemberListResponse {
        data,
        meta: PaginationMeta {
            page: 1,
            per_page: total,
            total,
        },
    };

    Ok((StatusCode::OK, Json(response)))
}

/// Query parameters for channel search.
#[derive(Debug, Deserialize)]
pub struct ChannelSearchQuery {
    pub q: Option<String>,
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}

/// Search channels by name.
#[instrument]
pub async fn search_channels(
    State(state): State<ApiState>,
    Path(org_id): Path<String>,
    Query(query): Query<ChannelSearchQuery>,
) -> AppResult<impl axum::response::IntoResponse> {
    let org_uuid = Uuid::parse_str(&org_id)
        .map_err(|e| AppError::Validation(format!("Invalid org ID: {}", e)))?;

    let page = query.page.unwrap_or(1);
    let per_page = query.per_page.unwrap_or(20);

    let channels = if let Some(q) = &query.q {
        state
            .services
            .channel_repo
            .search_by_name(OrgId::from_uuid(org_uuid), q, page, per_page)
            .await?
    } else {
        state
            .services
            .channel_repo
            .list_by_org(OrgId::from_uuid(org_uuid), page, per_page)
            .await?
    };

    let data: Vec<ChannelResponse> = channels
        .into_iter()
        .map(|c| ChannelResponse {
            id: c.id.to_string(),
            org_id: c.org_id.to_string(),
            name: c.name,
            description: c.description,
            channel_type: format!("{:?}", c.channel_type).to_lowercase(),
            is_private: c.is_private,
            created_by: c.created_by.to_string(),
            created_at: c.created_at.to_rfc3339(),
            updated_at: c.updated_at.to_rfc3339(),
        })
        .collect();

    let response = ChannelListResponse {
        data,
        meta: PaginationMeta {
            page,
            per_page,
            total: 0,
        },
    };

    Ok((StatusCode::OK, Json(response)))
}
