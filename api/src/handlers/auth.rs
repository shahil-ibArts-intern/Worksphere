use axum::{extract::State, http::StatusCode, Json};
use chrono::Utc;
use tracing::{info, instrument};
use uuid::Uuid;
use validator::Validate;

use crate::dto::auth::*;
use crate::state::ApiState;
use domain::error::{AppError, AppResult};
use domain::ids::{OrgId, UserId};
use domain::traits::*;
use domain::types::user::{User, UserRole, UserStatus};

/// Register a new user.
#[instrument]
pub async fn register(
    State(state): State<ApiState>,
    Json(req): Json<RegisterRequest>,
) -> AppResult<impl axum::response::IntoResponse> {
    req.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    // Check if user already exists
    if state
        .services
        .user_repo
        .find_by_email(&req.email)
        .await?
        .is_some()
    {
        return Err(AppError::Conflict(
            "User with this email already exists".to_string(),
        ));
    }

    // Hash password
    let password_hash = auth::hash_password(&req.password)?;

    // Create user
    let user = User {
        id: UserId::new(),
        email: req.email.clone(),
        username: req.username.clone(),
        display_name: req.display_name.clone(),
        avatar_url: None,
        status: UserStatus::Offline,
        role: UserRole::Member,
        last_active_at: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        deleted_at: None,
    };

    state.services.user_repo.create(&user).await?;

    // Store password hash separately
    sqlx::query("UPDATE users SET password_hash = $1 WHERE id = $2")
        .bind(&password_hash)
        .bind(user.id.as_uuid())
        .execute(&state.services.db)
        .await?;

    info!(user_id = %user.id, "User registered successfully");

    let response = AuthResponse {
        access_token: String::new(),
        refresh_token: String::new(),
        expires_in: 0,
        token_type: "Bearer".to_string(),
        user: UserResponse {
            id: user.id.to_string(),
            email: user.email,
            username: user.username,
            display_name: user.display_name,
            avatar_url: user.avatar_url,
            status: format!("{:?}", user.status).to_lowercase(),
            role: format!("{:?}", user.role).to_lowercase(),
        },
    };

    Ok((StatusCode::CREATED, Json(response)))
}

/// Login with email and password.
#[instrument]
pub async fn login(
    State(state): State<ApiState>,
    Json(req): Json<LoginRequest>,
) -> AppResult<impl axum::response::IntoResponse> {
    req.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    let user = state
        .services
        .user_repo
        .find_by_email(&req.email)
        .await?
        .ok_or_else(|| AppError::Authentication("Invalid email or password".to_string()))?;

    let password_hash: String = sqlx::query_scalar("SELECT password_hash FROM users WHERE id = $1")
        .bind(user.id.as_uuid())
        .fetch_one(&state.services.db)
        .await?;

    let valid = auth::verify_password(&req.password, &password_hash)?;

    if !valid {
        return Err(AppError::Authentication(
            "Invalid email or password".to_string(),
        ));
    }

    // Create tokens
    let org_id = OrgId::new();
    let role = format!("{:?}", user.role);
    let tokens = state.jwt.create_token_pair(user.id, org_id, &role)?;

    info!(user_id = %user.id, "User logged in successfully");

    let response = AuthResponse {
        access_token: tokens.access_token,
        refresh_token: tokens.refresh_token,
        expires_in: tokens.expires_in,
        token_type: tokens.token_type,
        user: UserResponse {
            id: user.id.to_string(),
            email: user.email,
            username: user.username,
            display_name: user.display_name,
            avatar_url: user.avatar_url,
            status: format!("{:?}", user.status).to_lowercase(),
            role: format!("{:?}", user.role).to_lowercase(),
        },
    };

    Ok((StatusCode::OK, Json(response)))
}

/// Refresh access token.
#[instrument]
pub async fn refresh_token(
    State(state): State<ApiState>,
    Json(req): Json<RefreshTokenRequest>,
) -> AppResult<impl axum::response::IntoResponse> {
    let claims = state.jwt.validate_refresh_token(&req.refresh_token)?;
    let user_id = UserId::from_uuid(
        Uuid::parse_str(&claims.sub)
            .map_err(|e| AppError::Authentication(format!("Invalid user ID: {}", e)))?,
    );

    let user = state
        .services
        .user_repo
        .find_by_id(user_id)
        .await?
        .ok_or_else(|| AppError::Authentication("User not found".to_string()))?;

    let org_id = OrgId::new();
    let role = format!("{:?}", user.role);
    let tokens = state.jwt.create_token_pair(user.id, org_id, &role)?;

    let response = AuthResponse {
        access_token: tokens.access_token,
        refresh_token: tokens.refresh_token,
        expires_in: tokens.expires_in,
        token_type: tokens.token_type,
        user: UserResponse {
            id: user.id.to_string(),
            email: user.email,
            username: user.username,
            display_name: user.display_name,
            avatar_url: user.avatar_url,
            status: format!("{:?}", user.status).to_lowercase(),
            role: format!("{:?}", user.role).to_lowercase(),
        },
    };

    Ok((StatusCode::OK, Json(response)))
}

/// Logout user.
#[instrument]
pub async fn logout(
    State(_state): State<ApiState>,
) -> AppResult<impl axum::response::IntoResponse> {
    // In a full implementation, this would revoke the session
    Ok(StatusCode::NO_CONTENT)
}

/// Forgot password.
#[instrument]
pub async fn forgot_password(
    State(_state): State<ApiState>,
    Json(req): Json<ForgotPasswordRequest>,
) -> AppResult<impl axum::response::IntoResponse> {
    req.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    // In a full implementation, this would send a password reset email
    Ok(StatusCode::NO_CONTENT)
}

/// Reset password.
#[instrument]
pub async fn reset_password(
    State(_state): State<ApiState>,
    Json(req): Json<ResetPasswordRequest>,
) -> AppResult<impl axum::response::IntoResponse> {
    req.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    // In a full implementation, this would validate the token and reset the password
    Ok(StatusCode::NO_CONTENT)
}
