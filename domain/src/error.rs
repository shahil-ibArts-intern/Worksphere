use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;

/// The single error type for the entire application.
/// Each variant wraps a domain-specific error.
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("authentication failed: {0}")]
    Authentication(String),

    #[error("authorization failed: {0}")]
    Authorization(String),

    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("validation error: {0}")]
    Validation(String),

    #[error("not found: {resource} with id {id}")]
    NotFound { resource: String, id: String },

    #[error("conflict: {0}")]
    Conflict(String),

    #[error("rate limit exceeded")]
    RateLimited,

    #[error("internal error: {0}")]
    Internal(String),

    #[error("service unavailable: {0}")]
    ServiceUnavailable(String),
}

/// Error response body sent to clients.
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
}

impl AppError {
    /// Returns the HTTP status code for this error.
    pub fn status_code(&self) -> StatusCode {
        match self {
            AppError::Authentication(_) => StatusCode::UNAUTHORIZED,
            AppError::Authorization(_) => StatusCode::FORBIDDEN,
            AppError::NotFound { .. } => StatusCode::NOT_FOUND,
            AppError::Conflict(_) => StatusCode::CONFLICT,
            AppError::Validation(_) => StatusCode::UNPROCESSABLE_ENTITY,
            AppError::RateLimited => StatusCode::TOO_MANY_REQUESTS,
            AppError::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::ServiceUnavailable(_) => StatusCode::SERVICE_UNAVAILABLE,
        }
    }

    /// Returns a client-safe error message.
    pub fn client_message(&self) -> String {
        match self {
            AppError::Authentication(msg) => msg.clone(),
            AppError::Authorization(msg) => msg.clone(),
            AppError::NotFound { resource, id } => {
                format!("{} with id {} not found", resource, id)
            }
            AppError::Conflict(msg) => msg.clone(),
            AppError::Validation(msg) => msg.clone(),
            AppError::RateLimited => "Too many requests. Please try again later.".to_string(),
            AppError::Database(_) => "Internal server error".to_string(),
            AppError::Internal(_) => "Internal server error".to_string(),
            AppError::ServiceUnavailable(msg) => msg.clone(),
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = self.status_code();
        let error_response = ErrorResponse {
            error: status.canonical_reason().unwrap_or("Error").to_string(),
            message: self.client_message(),
        };
        (status, Json(error_response)).into_response()
    }
}

/// Convenience type alias for Results with AppError.
pub type AppResult<T> = Result<T, AppError>;
