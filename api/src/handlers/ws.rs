//! WebSocket upgrade handler.
//!
//! Provides the `WS /api/v1/ws?token={jwt}` endpoint for real-time communication.

use axum::extract::ws::WebSocketUpgrade;
use axum::extract::{ConnectInfo, Query, State};
use axum::response::IntoResponse;
use std::net::SocketAddr;
use tracing::instrument;

use realtime::server::{ws_handler, WsQueryParams};
use realtime::WsServerState;

use crate::state::ApiState;

/// WebSocket endpoint handler.
///
/// Delegates to the realtime crate's `ws_handler` with the API state's
/// JWT manager and connection registry.
#[instrument(skip(ws, state, params))]
pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<ApiState>,
    Query(params): Query<WsQueryParams>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> impl IntoResponse {
    let ws_state = WsServerState::new(state.jwt.clone());
    ws_handler(ws, State(ws_state), Query(params), ConnectInfo(addr)).await
}
