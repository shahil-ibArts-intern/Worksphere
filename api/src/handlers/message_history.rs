use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use tracing::instrument;
use uuid::Uuid;

use crate::dto::message_history::*;
use crate::dto::user::PaginationMeta;
use crate::state::ApiState;
use domain::error::{AppError, AppResult};
use domain::ids::MessageId;

/// Get message edit/delete history.
#[instrument]
pub async fn get_message_history(
    State(state): State<ApiState>,
    Path(message_id): Path<String>,
    Query(params): Query<HistoryQueryParams>,
) -> AppResult<impl axum::response::IntoResponse> {
    let message_uuid = Uuid::parse_str(&message_id)
        .map_err(|e| AppError::Validation(format!("Invalid message ID: {}", e)))?;

    let page = params.page.unwrap_or(1).max(1);
    let per_page = params.per_page.unwrap_or(50).clamp(1, 100);

    let history = state
        .services
        .message_history_service
        .get_history(MessageId::from_uuid(message_uuid), page, per_page)
        .await?;

    let total_count = state
        .services
        .message_history_service
        .get_history_count(MessageId::from_uuid(message_uuid))
        .await?;

    let data: Vec<MessageHistoryResponse> = history
        .into_iter()
        .map(|h| MessageHistoryResponse {
            id: h.id.to_string(),
            message_id: h.message_id.to_string(),
            user_id: h.user_id.to_string(),
            history_type: format!("{:?}", h.history_type).to_lowercase(),
            previous_content: h.previous_content,
            new_content: h.new_content,
            created_at: h.created_at.to_rfc3339(),
        })
        .collect();

    let response = MessageHistoryListResponse {
        data,
        meta: PaginationMeta {
            page,
            per_page,
            total: total_count,
        },
        total_count,
    };

    Ok((StatusCode::OK, Json(response)))
}

/// Query parameters for history pagination.
#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoryQueryParams {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}
