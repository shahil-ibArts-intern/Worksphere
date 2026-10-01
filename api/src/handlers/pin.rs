use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use tracing::{info, instrument};
use uuid::Uuid;

use crate::dto::pin::*;
use crate::state::ApiState;
use domain::error::{AppError, AppResult};
use domain::ids::{ChannelId, MessageId, OrgId, UserId};

/// Pin a message in a channel.
#[instrument]
pub async fn pin_message(
    State(state): State<ApiState>,
    Path(channel_id): Path<String>,
    Json(req): Json<PinMessageRequest>,
) -> AppResult<impl axum::response::IntoResponse> {
    let channel_uuid = Uuid::parse_str(&channel_id)
        .map_err(|e| AppError::Validation(format!("Invalid channel ID: {}", e)))?;

    let message_uuid = Uuid::parse_str(&req.message_id)
        .map_err(|e| AppError::Validation(format!("Invalid message ID: {}", e)))?;

    // In production, user_id and org_id would come from authenticated context
    let user_id = UserId::new();
    let org_id = OrgId::new();

    let pin = state
        .services
        .pin_service
        .pin_message(
            ChannelId::from_uuid(channel_uuid),
            org_id,
            user_id,
            MessageId::from_uuid(message_uuid),
        )
        .await?;

    info!(channel_id = %channel_id, message_id = %req.message_id, "Message pinned");

    let response = PinResponse {
        id: pin.id.to_string(),
        channel_id: pin.channel_id.to_string(),
        message_id: pin.message_id.to_string(),
        pinned_by: pin.pinned_by.to_string(),
        created_at: pin.created_at.to_rfc3339(),
    };

    Ok((StatusCode::CREATED, Json(response)))
}

/// Unpin a message from a channel.
#[instrument]
pub async fn unpin_message(
    State(state): State<ApiState>,
    Path((channel_id, message_id)): Path<(String, String)>,
) -> AppResult<impl axum::response::IntoResponse> {
    let channel_uuid = Uuid::parse_str(&channel_id)
        .map_err(|e| AppError::Validation(format!("Invalid channel ID: {}", e)))?;

    let message_uuid = Uuid::parse_str(&message_id)
        .map_err(|e| AppError::Validation(format!("Invalid message ID: {}", e)))?;

    // In production, user_id and org_id would come from authenticated context
    let user_id = UserId::new();
    let org_id = OrgId::new();

    state
        .services
        .pin_service
        .unpin_message(
            ChannelId::from_uuid(channel_uuid),
            org_id,
            user_id,
            MessageId::from_uuid(message_uuid),
        )
        .await?;

    info!(channel_id = %channel_id, message_id = %message_id, "Message unpinned");

    Ok(StatusCode::NO_CONTENT)
}

/// List pinned messages in a channel.
#[instrument]
pub async fn list_pins(
    State(state): State<ApiState>,
    Path(channel_id): Path<String>,
) -> AppResult<impl axum::response::IntoResponse> {
    let channel_uuid = Uuid::parse_str(&channel_id)
        .map_err(|e| AppError::Validation(format!("Invalid channel ID: {}", e)))?;

    let pins = state
        .services
        .pin_service
        .list_pins(ChannelId::from_uuid(channel_uuid))
        .await?;

    let data: Vec<PinResponse> = pins
        .into_iter()
        .map(|p| PinResponse {
            id: p.id.to_string(),
            channel_id: p.channel_id.to_string(),
            message_id: p.message_id.to_string(),
            pinned_by: p.pinned_by.to_string(),
            created_at: p.created_at.to_rfc3339(),
        })
        .collect();

    let response = PinListResponse { data };

    Ok((StatusCode::OK, Json(response)))
}

/// Request to pin a message.
#[derive(Debug, serde::Deserialize, validator::Validate)]
#[serde(rename_all = "camelCase")]
pub struct PinMessageRequest {
    #[validate(length(min = 1))]
    pub message_id: String,
}
