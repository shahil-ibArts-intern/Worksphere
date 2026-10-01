use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use tracing::{info, instrument};
use uuid::Uuid;
use validator::Validate;

use crate::dto::dm::*;
use crate::dto::message::MessageResponse;
use crate::dto::user::PaginationMeta;
use crate::state::ApiState;
use domain::error::{AppError, AppResult};
use domain::ids::{DmConversationId, MessageId, OrgId, UserId};
use serde::{Deserialize, Serialize};

/// List DM conversations for the current user.
#[instrument]
pub async fn list_dm_conversations(
    State(state): State<ApiState>,
    Query(params): Query<DmListQueryParams>,
) -> AppResult<impl axum::response::IntoResponse> {
    // In production, user_id would come from authenticated context
    let user_id = UserId::new();

    let page = params.page.unwrap_or(1).max(1);
    let per_page = params.per_page.unwrap_or(50).clamp(1, 100);

    let conversations = state
        .services
        .dm_service
        .list_conversations(user_id, page, per_page)
        .await?;

    // For each conversation, we'd fetch participants and last message
    // For now, return basic info
    let data: Vec<DmConversationResponse> = conversations
        .into_iter()
        .map(|c| DmConversationResponse {
            id: c.id.to_string(),
            org_id: c.org_id.to_string(),
            name: c.name,
            is_group: c.is_group,
            created_by: c.created_by.to_string(),
            created_at: c.created_at.to_rfc3339(),
            updated_at: c.updated_at.to_rfc3339(),
            participant_count: 0, // Would fetch from DB
            last_message: None,
        })
        .collect();

    let response = DmConversationListResponse {
        data,
        meta: PaginationMeta {
            page,
            per_page,
            total: 0,
        },
    };

    Ok((StatusCode::OK, Json(response)))
}

/// Start or get a 1:1 DM conversation.
#[instrument]
pub async fn get_or_create_dm(
    State(state): State<ApiState>,
    Path(other_user_id): Path<String>,
) -> AppResult<impl axum::response::IntoResponse> {
    let other_uuid = Uuid::parse_str(&other_user_id)
        .map_err(|e| AppError::Validation(format!("Invalid user ID: {}", e)))?;

    // In production, current_user_id and org_id would come from authenticated context
    let current_user_id = UserId::new();
    let org_id = OrgId::new();

    let conversation = state
        .services
        .dm_service
        .get_or_create_direct_conversation(org_id, current_user_id, UserId::from_uuid(other_uuid))
        .await?;

    info!(conversation_id = %conversation.id, "1:1 DM retrieved/created");

    let response = DmConversationResponse {
        id: conversation.id.to_string(),
        org_id: conversation.org_id.to_string(),
        name: conversation.name,
        is_group: conversation.is_group,
        created_by: conversation.created_by.to_string(),
        created_at: conversation.created_at.to_rfc3339(),
        updated_at: conversation.updated_at.to_rfc3339(),
        participant_count: 2,
        last_message: None,
    };

    Ok((StatusCode::OK, Json(response)))
}

/// Create a group DM conversation.
#[instrument]
pub async fn create_group_dm(
    State(state): State<ApiState>,
    Json(req): Json<CreateGroupDmRequest>,
) -> AppResult<impl axum::response::IntoResponse> {
    req.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    // In production, current_user_id and org_id would come from authenticated context
    let current_user_id = UserId::new();
    let org_id = OrgId::new();

    let participant_ids: Result<Vec<UserId>, _> = req
        .participant_ids
        .iter()
        .map(|id| Uuid::parse_str(id).map(UserId::from_uuid))
        .collect();

    let participant_ids = participant_ids
        .map_err(|e| AppError::Validation(format!("Invalid participant ID: {}", e)))?;

    let conversation = state
        .services
        .dm_service
        .create_group_conversation(org_id, req.name, current_user_id, participant_ids)
        .await?;

    info!(conversation_id = %conversation.id, "Group DM created");

    let response = DmConversationResponse {
        id: conversation.id.to_string(),
        org_id: conversation.org_id.to_string(),
        name: conversation.name,
        is_group: conversation.is_group,
        created_by: conversation.created_by.to_string(),
        created_at: conversation.created_at.to_rfc3339(),
        updated_at: conversation.updated_at.to_rfc3339(),
        participant_count: 0, // Would fetch from DB
        last_message: None,
    };

    Ok((StatusCode::CREATED, Json(response)))
}

/// Get DM conversation by ID.
#[instrument]
pub async fn get_dm_conversation(
    State(state): State<ApiState>,
    Path(conversation_id): Path<String>,
) -> AppResult<impl axum::response::IntoResponse> {
    let uuid = Uuid::parse_str(&conversation_id)
        .map_err(|e| AppError::Validation(format!("Invalid conversation ID: {}", e)))?;

    let conversation = state
        .services
        .dm_service
        .get_conversation(DmConversationId::from_uuid(uuid))
        .await?;

    let response = DmConversationResponse {
        id: conversation.id.to_string(),
        org_id: conversation.org_id.to_string(),
        name: conversation.name,
        is_group: conversation.is_group,
        created_by: conversation.created_by.to_string(),
        created_at: conversation.created_at.to_rfc3339(),
        updated_at: conversation.updated_at.to_rfc3339(),
        participant_count: 0, // Would fetch from DB
        last_message: None,
    };

    Ok((StatusCode::OK, Json(response)))
}

/// Get DM conversation participants.
#[instrument]
pub async fn get_dm_participants(
    State(state): State<ApiState>,
    Path(conversation_id): Path<String>,
) -> AppResult<impl axum::response::IntoResponse> {
    let uuid = Uuid::parse_str(&conversation_id)
        .map_err(|e| AppError::Validation(format!("Invalid conversation ID: {}", e)))?;

    let participants = state
        .services
        .dm_service
        .get_participants(DmConversationId::from_uuid(uuid))
        .await?;

    let data: Vec<DmParticipantResponse> = participants
        .into_iter()
        .map(|p| DmParticipantResponse {
            id: p.id.to_string(),
            user_id: p.user_id.to_string(),
            joined_at: p.joined_at.to_rfc3339(),
            last_read_at: p.last_read_at.map(|t| t.to_rfc3339()),
        })
        .collect();

    Ok((StatusCode::OK, Json(data)))
}

/// Send a message in a DM conversation.
#[instrument]
pub async fn send_dm_message(
    State(state): State<ApiState>,
    Path(conversation_id): Path<String>,
    Json(req): Json<SendDmMessageRequest>,
) -> AppResult<impl axum::response::IntoResponse> {
    req.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    let uuid = Uuid::parse_str(&conversation_id)
        .map_err(|e| AppError::Validation(format!("Invalid conversation ID: {}", e)))?;

    // In production, current_user_id and org_id would come from authenticated context
    let current_user_id = UserId::new();
    let org_id = OrgId::new();

    let client_message_id = req
        .id
        .as_ref()
        .and_then(|id| Uuid::parse_str(id).ok().map(MessageId::from_uuid));

    let message = state
        .services
        .dm_service
        .send_message(
            DmConversationId::from_uuid(uuid),
            org_id,
            current_user_id,
            &req.content,
            client_message_id,
        )
        .await?;

    info!(conversation_id = %conversation_id, "DM message sent");

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

/// List messages in a DM conversation.
#[instrument]
pub async fn list_dm_messages(
    State(state): State<ApiState>,
    Path(conversation_id): Path<String>,
    Query(params): Query<DmMessageQueryParams>,
) -> AppResult<impl axum::response::IntoResponse> {
    let uuid = Uuid::parse_str(&conversation_id)
        .map_err(|e| AppError::Validation(format!("Invalid conversation ID: {}", e)))?;

    let cursor = params
        .cursor
        .as_ref()
        .and_then(|id| Uuid::parse_str(id).ok().map(MessageId::from_uuid));
    let limit = params.limit.unwrap_or(50).clamp(1, 100);

    let messages = state
        .services
        .dm_service
        .list_messages(DmConversationId::from_uuid(uuid), cursor, limit)
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

    let response = DmMessageListResponse { data };

    Ok((StatusCode::OK, Json(response)))
}

/// Mark DM messages as read.
#[instrument]
pub async fn mark_dm_as_read(
    State(state): State<ApiState>,
    Path(conversation_id): Path<String>,
) -> AppResult<impl axum::response::IntoResponse> {
    let uuid = Uuid::parse_str(&conversation_id)
        .map_err(|e| AppError::Validation(format!("Invalid conversation ID: {}", e)))?;

    // In production, user_id would come from authenticated context
    let user_id = UserId::new();

    state
        .services
        .dm_service
        .mark_as_read(DmConversationId::from_uuid(uuid), user_id)
        .await?;

    Ok(StatusCode::NO_CONTENT)
}

/// Leave a DM conversation.
#[instrument]
pub async fn leave_dm_conversation(
    State(state): State<ApiState>,
    Path(conversation_id): Path<String>,
) -> AppResult<impl axum::response::IntoResponse> {
    let uuid = Uuid::parse_str(&conversation_id)
        .map_err(|e| AppError::Validation(format!("Invalid conversation ID: {}", e)))?;

    // In production, user_id would come from authenticated context
    let user_id = UserId::new();

    state
        .services
        .dm_service
        .leave_conversation(DmConversationId::from_uuid(uuid), user_id)
        .await?;

    Ok(StatusCode::NO_CONTENT)
}

/// Query parameters for DM conversation list.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DmListQueryParams {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}

/// Query parameters for DM message list.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DmMessageQueryParams {
    pub cursor: Option<String>,
    pub limit: Option<i64>,
}

/// DM message list response.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DmMessageListResponse {
    pub data: Vec<MessageResponse>,
}
