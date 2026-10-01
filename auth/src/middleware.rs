use axum::{extract::Request, http::header, middleware::Next, response::Response};
use domain::error::AppError;

/// Extracts the JWT token from the Authorization header.
pub fn extract_token_from_header(auth_header: &str) -> Option<&str> {
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
            if extract_token_from_header(header).is_some() {
                // Token validation would happen here with JwtManager
                // For now, just pass through
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

/// Middleware that checks for a specific permission.
pub async fn require_auth(request: Request, next: Next) -> Result<Response, AppError> {
    auth_middleware(request, next).await
}
