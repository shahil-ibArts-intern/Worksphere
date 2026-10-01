use serde::Serialize;

/// Pin response.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PinResponse {
    pub id: String,
    pub channel_id: String,
    pub message_id: String,
    pub pinned_by: String,
    pub created_at: String,
}

/// Pinned message list response.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PinListResponse {
    pub data: Vec<PinResponse>,
}
