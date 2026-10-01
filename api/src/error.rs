use axum::{
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;

/// API error response body.
#[derive(Debug, Serialize)]
pub struct ApiErrorResponse {
    pub error: String,
    pub message: String,
    pub status: u16,
}

/// Converts domain::AppError into an Axum response.
pub fn app_error_to_response(err: domain::AppError) -> Response {
    let status = err.status_code();
    let body = ApiErrorResponse {
        error: status.canonical_reason().unwrap_or("Error").to_string(),
        message: err.client_message(),
        status: status.as_u16(),
    };
    (status, Json(body)).into_response()
}
