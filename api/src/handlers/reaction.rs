use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use tracing::{info, instrument};
use uuid::Uuid;
use validator::Validate;

use crate::dto::reaction::*;
use crate::state::ApiState;
use domain::error::{AppError, AppResult};
use domain::ids::{MessageId, OrgId, UserId};

/// Add a reaction to a message.
#[instrument]
pub async fn add_reaction(
    State(state): State<ApiState>,
    Path(message_id): Path<String>,
    Json(req): Json<AddReactionRequest>,
) -> AppResult<impl axum::response::IntoResponse> {
    req.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    let message_uuid = Uuid::parse_str(&message_id)
        .map_err(|e| AppError::Validation(format!("Invalid message ID: {}", e)))?;

    // In production, user_id and org_id would come from the authenticated user context
    let user_id = UserId::new();
    let org_id = OrgId::new();

    let reaction = state
        .services
        .reaction_service
        .add_reaction(
            MessageId::from_uuid(message_uuid),
            org_id,
            user_id,
            &req.emoji,
        )
        .await?;

    info!(message_id = %message_id, emoji = %req.emoji, "Reaction added");

    let response = ReactionResponse {
        id: reaction.id.to_string(),
        message_id: reaction.message_id.to_string(),
        user_id: reaction.user_id.to_string(),
        emoji: reaction.emoji,
        created_at: reaction.created_at.to_rfc3339(),
    };

    Ok((StatusCode::CREATED, Json(response)))
}

/// Remove a reaction from a message.
#[instrument]
pub async fn remove_reaction(
    State(state): State<ApiState>,
    Path((message_id, emoji)): Path<(String, String)>,
) -> AppResult<impl axum::response::IntoResponse> {
    let message_uuid = Uuid::parse_str(&message_id)
        .map_err(|e| AppError::Validation(format!("Invalid message ID: {}", e)))?;

    // In production, user_id and org_id would come from the authenticated user context
    let user_id = UserId::new();
    let org_id = OrgId::new();

    state
        .services
        .reaction_service
        .remove_reaction(MessageId::from_uuid(message_uuid), org_id, user_id, &emoji)
        .await?;

    info!(message_id = %message_id, emoji = %emoji, "Reaction removed");

    Ok(StatusCode::NO_CONTENT)
}

/// Get all reactions on a message.
#[instrument]
pub async fn get_reactions(
    State(state): State<ApiState>,
    Path(message_id): Path<String>,
) -> AppResult<impl axum::response::IntoResponse> {
    let message_uuid = Uuid::parse_str(&message_id)
        .map_err(|e| AppError::Validation(format!("Invalid message ID: {}", e)))?;

    let reactions = state
        .services
        .reaction_service
        .get_reactions(MessageId::from_uuid(message_uuid))
        .await?;

    let data: Vec<ReactionCountResponse> = reactions
        .into_iter()
        .map(|rc| ReactionCountResponse {
            emoji: rc.emoji,
            count: rc.count,
            users: rc.users.into_iter().map(|u| u.to_string()).collect(),
        })
        .collect();

    let response = ReactionListResponse { data };

    Ok((StatusCode::OK, Json(response)))
}
