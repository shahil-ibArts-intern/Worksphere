use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use tracing::{info, instrument};
use uuid::Uuid;
use validator::Validate;

use crate::dto::message::MessageResponse;
use crate::dto::thread::*;
use crate::dto::user::PaginationMeta;
use crate::state::ApiState;
use domain::error::{AppError, AppResult};
use domain::ids::{ChannelId, MessageId, OrgId, UserId};
use serde::Deserialize;

/// Get thread replies for a message.
#[instrument]
pub async fn get_thread_replies(
    State(state): State<ApiState>,
    Path(message_id): Path<String>,
    Query(params): Query<ThreadQueryParams>,
) -> AppResult<impl axum::response::IntoResponse> {
    let message_uuid = Uuid::parse_str(&message_id)
        .map_err(|e| AppError::Validation(format!("Invalid message ID: {}", e)))?;

    let page = params.page.unwrap_or(1).max(1);
    let per_page = params.per_page.unwrap_or(50).clamp(1, 100);

    let replies = state
        .services
        .thread_service
        .get_thread_replies(MessageId::from_uuid(message_uuid), page, per_page)
        .await?;

    let reply_count = state
        .services
        .thread_service
        .get_thread_reply_count(MessageId::from_uuid(message_uuid))
        .await?;

    let data: Vec<MessageResponse> = replies
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

    let response = ThreadReplyListResponse {
        data,
        meta: PaginationMeta {
            page,
            per_page,
            total: reply_count,
        },
        reply_count,
    };

    Ok((StatusCode::OK, Json(response)))
}

/// Reply in a thread.
#[instrument]
pub async fn reply_in_thread(
    State(state): State<ApiState>,
    Path(message_id): Path<String>,
    Json(req): Json<ThreadReplyRequest>,
) -> AppResult<impl axum::response::IntoResponse> {
    req.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    let message_uuid = Uuid::parse_str(&message_id)
        .map_err(|e| AppError::Validation(format!("Invalid message ID: {}", e)))?;

    // In production, user_id, org_id, channel_id would come from authenticated context
    let user_id = UserId::new();
    let org_id = OrgId::new();
    let channel_id = ChannelId::new(); // Would need to fetch from parent message

    let client_message_id = req
        .id
        .as_ref()
        .and_then(|id| Uuid::parse_str(id).ok().map(MessageId::from_uuid));

    let message = state
        .services
        .thread_service
        .reply_in_thread(
            channel_id,
            org_id,
            user_id,
            MessageId::from_uuid(message_uuid),
            &req.content,
            client_message_id,
        )
        .await?;

    info!(parent_id = %message_id, "Thread reply created");

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

/// Query parameters for thread pagination.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThreadQueryParams {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}
