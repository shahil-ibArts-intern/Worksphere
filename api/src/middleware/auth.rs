use axum::{extract::Request, http::header, middleware::Next, response::Response};
use domain::error::AppError;

/// Extracts the JWT token from the Authorization header.
pub fn extract_bearer_token(auth_header: &str) -> Option<&str> {
    auth_header.strip_prefix("Bearer ")
}

/// Axum middleware that validates JWT tokens.
pub async fn auth_middleware(request: Request, next: Next) -> Result<Response, AppError> {
    let auth_header = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok());

    match auth_header {
        Some(header) => {
            if extract_bearer_token(header).is_some() {
                Ok(next.run(request).await)
            } else {
                Err(AppError::Authentication(
                    "Invalid authorization header format".to_string(),
                ))
            }
        }
        None => Err(AppError::Authentication(
            "Missing authorization header".to_string(),
        )),
    }
}
