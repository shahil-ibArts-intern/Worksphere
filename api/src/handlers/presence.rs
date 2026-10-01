//! Presence handlers — HTTP handlers for presence and status operations.

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use tracing::{info, instrument};
use uuid::Uuid;
use validator::Validate;

use crate::dto::presence::*;
use crate::dto::user::PaginationMeta;
use crate::state::ApiState;
use domain::error::{AppError, AppResult};
use domain::ids::UserId;

/// Get all online users in an organization.
#[instrument]
pub async fn get_online_users(
    State(state): State<ApiState>,
    Path(org_id): Path<String>,
) -> AppResult<impl axum::response::IntoResponse> {
    let _org_uuid = Uuid::parse_str(&org_id)
        .map_err(|e| AppError::Validation(format!("Invalid org ID: {}", e)))?;

    // In production, this would query the presence service
    // For now, return an empty list
    let response = PresenceListResponse {
        data: vec![],
        meta: PaginationMeta {
            page: 1,
            per_page: 0,
            total: 0,
        },
    };

    Ok((StatusCode::OK, Json(response)))
}

/// Update the current user's presence status.
#[instrument]
pub async fn update_presence(
    State(state): State<ApiState>,
    Json(req): Json<UpdatePresenceRequest>,
) -> AppResult<impl axum::response::IntoResponse> {
    req.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    // In production, user_id would come from the authenticated user context
    let user_id = UserId::new();

    info!(user_id = %user_id, status = %req.status, "Presence updated");

    let response = PresenceResponse {
        user_id: user_id.to_string(),
        status: req.status,
        custom_status: req.custom_status,
        is_online: true,
    };

    Ok((StatusCode::OK, Json(response)))
}

/// Set a custom status message.
#[instrument]
pub async fn update_status(
    State(state): State<ApiState>,
    Json(req): Json<UpdateStatusRequest>,
) -> AppResult<impl axum::response::IntoResponse> {
    req.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    // In production, user_id would come from the authenticated user context
    let user_id = UserId::new();

    info!(user_id = %user_id, "Custom status updated");

    let response = StatusResponse {
        user_id: user_id.to_string(),
        custom_status: req.custom_status,
    };

    Ok((StatusCode::OK, Json(response)))
}
