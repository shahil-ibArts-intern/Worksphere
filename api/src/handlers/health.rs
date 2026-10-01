use axum::Json;
use serde::Serialize;
use std::collections::HashMap;

/// Health check response.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
    pub timestamp: String,
}

/// Health check endpoint.
pub async fn health_check() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
    })
}

/// Readiness check response.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReadinessResponse {
    pub status: String,
    pub checks: HashMap<String, String>,
}

/// Readiness check endpoint.
pub async fn readiness_check() -> Json<ReadinessResponse> {
    let mut checks = HashMap::new();
    checks.insert("database".to_string(), "ok".to_string());

    Json(ReadinessResponse {
        status: "ok".to_string(),
        checks,
    })
}
