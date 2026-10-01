use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use tracing::{info, instrument};
use uuid::Uuid;
use validator::Validate;

use crate::dto::message::*;
use crate::dto::user::PaginationMeta;
use crate::state::ApiState;
use domain::error::{AppError, AppResult};
use domain::ids::{ChannelId, MessageId, UserId};
use domain::traits::*;

/// List messages in a channel.
#[instrument]
pub async fn list_messages(
    State(state): State<ApiState>,
    Path(channel_id): Path<String>,
) -> AppResult<impl axum::response::IntoResponse> {
    let uuid = Uuid::parse_str(&channel_id)
        .map_err(|e| AppError::Validation(format!("Invalid channel ID: {}", e)))?;
    let messages = state
        .services
        .message_repo
        .list_by_channel(ChannelId::from_uuid(uuid), None, 50)
        .await?;

    let data: Vec<MessageResponse> = messages
        .into_iter()
        .map(|m| MessageResponse {
            id: m.id.to_string(),
            channel_id: m.channel_id.to_string(),
            user_id: m.user_id.to_string(),
            content: m.content,
            message_type: format!("{:?}", m.message_type).to_lowercase(),
            parent_id: m.parent_id.map(|id| id.to_string()),
            edited_at: m.edited_at.map(|t| t.to_rfc3339()),
            created_at: m.created_at.to_rfc3339(),
        })
        .collect();

    let response = MessageListResponse {
        data,
        meta: PaginationMeta {
            page: 1,
            per_page: 50,
            total: 0,
        },
        next_cursor: None,
    };

    Ok((StatusCode::OK, Json(response)))
}

/// Send a message to a channel.
///
/// Supports idempotency via client-generated message IDs.
/// If `id` is provided in the request, it's used as the message ID.
/// If a message with that ID already exists, the existing message is returned.
#[instrument]
pub async fn send_message(
    State(state): State<ApiState>,
    Path(channel_id): Path<String>,
    Json(req): Json<SendMessageRequest>,
) -> AppResult<impl axum::response::IntoResponse> {
    req.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    let uuid = Uuid::parse_str(&channel_id)
        .map_err(|e| AppError::Validation(format!("Invalid channel ID: {}", e)))?;

    // In production, user_id would come from the authenticated user context
    let user_id = UserId::new();

    // Parse client-provided message ID for idempotency
    let client_message_id = req
        .id
        .as_ref()
        .and_then(|id| Uuid::parse_str(id).ok().map(MessageId::from_uuid));

    let message = state
        .services
        .message_repo
        .create_with_id(
            ChannelId::from_uuid(uuid),
            user_id,
            &req.content,
            req.parent_id
                .as_ref()
                .and_then(|id| Uuid::parse_str(id).ok())
                .map(MessageId::from_uuid),
            client_message_id,
        )
        .await?;

    info!(message_id = %message.id, "Message sent successfully");

    let response = MessageResponse {
        id: message.id.to_string(),
        channel_id: message.channel_id.to_string(),
        user_id: message.user_id.to_string(),
        content: message.content,
        message_type: format!("{:?}", message.message_type).to_lowercase(),
        parent_id: message.parent_id.map(|id| id.to_string()),
        edited_at: message.edited_at.map(|t| t.to_rfc3339()),
        created_at: message.created_at.to_rfc3339(),
    };

    Ok((StatusCode::CREATED, Json(response)))
}

/// Get message by ID.
#[instrument]
pub async fn get_message(
    State(state): State<ApiState>,
    Path(message_id): Path<String>,
) -> AppResult<impl axum::response::IntoResponse> {
    let uuid = Uuid::parse_str(&message_id)
        .map_err(|e| AppError::Validation(format!("Invalid message ID: {}", e)))?;
    let message = state
        .services
        .message_repo
        .find_by_id(MessageId::from_uuid(uuid))
        .await?
        .ok_or_else(|| AppError::NotFound {
            resource: "Message".to_string(),
            id: message_id,
        })?;

    let response = MessageResponse {
        id: message.id.to_string(),
        channel_id: message.channel_id.to_string(),
        user_id: message.user_id.to_string(),
        content: message.content,
        message_type: format!("{:?}", message.message_type).to_lowercase(),
        parent_id: message.parent_id.map(|id| id.to_string()),
        edited_at: message.edited_at.map(|t| t.to_rfc3339()),
        created_at: message.created_at.to_rfc3339(),
    };

    Ok((StatusCode::OK, Json(response)))
}

/// Edit a message.
#[instrument]
pub async fn edit_message(
    State(state): State<ApiState>,
    Path(message_id): Path<String>,
    Json(req): Json<EditMessageRequest>,
) -> AppResult<impl axum::response::IntoResponse> {
    req.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    let uuid = Uuid::parse_str(&message_id)
        .map_err(|e| AppError::Validation(format!("Invalid message ID: {}", e)))?;
    let message = state
        .services
        .message_repo
        .find_by_id(MessageId::from_uuid(uuid))
        .await?
        .ok_or_else(|| AppError::NotFound {
            resource: "Message".to_string(),
            id: message_id,
        })?;

    let mut updated_message = message.clone();
    updated_message.content = req.content.clone();

    let updated = state.services.message_repo.update(&updated_message).await?;

    let response = MessageResponse {
        id: updated.id.to_string(),
        channel_id: updated.channel_id.to_string(),
        user_id: updated.user_id.to_string(),
        content: updated.content,
        message_type: format!("{:?}", updated.message_type).to_lowercase(),
        parent_id: updated.parent_id.map(|id| id.to_string()),
        edited_at: updated.edited_at.map(|t| t.to_rfc3339()),
        created_at: updated.created_at.to_rfc3339(),
    };

    Ok((StatusCode::OK, Json(response)))
}

/// Delete a message.
#[instrument]
pub async fn delete_message(
    State(state): State<ApiState>,
    Path(message_id): Path<String>,
) -> AppResult<impl axum::response::IntoResponse> {
    let uuid = Uuid::parse_str(&message_id)
        .map_err(|e| AppError::Validation(format!("Invalid message ID: {}", e)))?;
    let user_id = UserId::new();

    state
        .services
        .message_repo
        .soft_delete(MessageId::from_uuid(uuid), user_id)
        .await?;

    Ok(StatusCode::NO_CONTENT)
}
