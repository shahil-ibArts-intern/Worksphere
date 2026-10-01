use axum::{
    routing::{delete, get, patch, post},
    Router,
};

use crate::handlers;
use crate::state::ApiState;

/// Creates the API router with all routes.
pub fn create_router(state: ApiState) -> Router {
    Router::new()
        // Health checks
        .route("/health", get(handlers::health::health_check))
        .route("/ready", get(handlers::health::readiness_check))
        // WebSocket endpoint
        .route("/api/v1/ws", get(handlers::ws::websocket_handler))
        // Auth routes
        .route("/api/v1/auth/register", post(handlers::auth::register))
        .route("/api/v1/auth/login", post(handlers::auth::login))
        .route("/api/v1/auth/refresh", post(handlers::auth::refresh_token))
        .route("/api/v1/auth/logout", post(handlers::auth::logout))
        .route("/api/v1/auth/forgot-password", post(handlers::auth::forgot_password))
        .route("/api/v1/auth/reset-password", post(handlers::auth::reset_password))
        // User routes
        .route("/api/v1/users/me", get(handlers::user::get_current_user))
        .route("/api/v1/users/me", patch(handlers::user::update_profile))
        .route("/api/v1/users/:user_id", get(handlers::user::get_user_by_id))
        .route("/api/v1/orgs/:org_id/users", get(handlers::user::list_org_users))
        // Organization routes
        .route("/api/v1/orgs", post(handlers::org::create_org))
        .route("/api/v1/orgs/:org_id", get(handlers::org::get_org))
        .route("/api/v1/orgs/:org_id", patch(handlers::org::update_org))
        .route("/api/v1/orgs/:org_id", delete(handlers::org::delete_org))
        .route("/api/v1/orgs/:org_id/members", get(handlers::org::list_members))
        .route("/api/v1/orgs/:org_id/invitations", post(handlers::org::invite_user))
        .route(
            "/api/v1/orgs/:org_id/members/:user_id/role",
            patch(handlers::org::change_member_role),
        )
        .route(
            "/api/v1/orgs/:org_id/members/:user_id",
            delete(handlers::org::remove_member),
        )
        // Channel routes
        .route("/api/v1/orgs/:org_id/channels", get(handlers::channel::list_channels))
        .route("/api/v1/orgs/:org_id/channels", post(handlers::channel::create_channel))
        .route(
            "/api/v1/orgs/:org_id/channels/search",
            get(handlers::channel::search_channels),
        )
        .route(
            "/api/v1/orgs/:org_id/channels/:channel_id",
            get(handlers::channel::get_channel),
        )
        .route(
            "/api/v1/orgs/:org_id/channels/:channel_id",
            patch(handlers::channel::update_channel),
        )
        .route(
            "/api/v1/orgs/:org_id/channels/:channel_id",
            delete(handlers::channel::delete_channel),
        )
        .route(
            "/api/v1/orgs/:org_id/channels/:channel_id/join",
            post(handlers::channel::join_channel),
        )
        .route(
            "/api/v1/orgs/:org_id/channels/:channel_id/leave",
            post(handlers::channel::leave_channel),
        )
        .route(
            "/api/v1/orgs/:org_id/channels/:channel_id/members",
            get(handlers::channel::list_channel_members),
        )
        // Message routes
        .route(
            "/api/v1/channels/:channel_id/messages",
            get(handlers::message::list_messages),
        )
        .route(
            "/api/v1/channels/:channel_id/messages",
            post(handlers::message::send_message),
        )
        .route("/api/v1/messages/:message_id", get(handlers::message::get_message))
        .route("/api/v1/messages/:message_id", patch(handlers::message::edit_message))
        .route("/api/v1/messages/:message_id", delete(handlers::message::delete_message))
        // Thread routes
        .route(
            "/api/v1/messages/:message_id/thread",
            get(handlers::thread::get_thread_replies),
        )
        .route(
            "/api/v1/messages/:message_id/thread",
            post(handlers::thread::reply_in_thread),
        )
        // DM routes
        .route("/api/v1/dm", get(handlers::dm::list_dm_conversations))
        .route("/api/v1/dm", post(handlers::dm::create_group_dm))
        .route("/api/v1/dm/:user_id", get(handlers::dm::get_or_create_dm))
        .route("/api/v1/dm/:conversation_id", get(handlers::dm::get_dm_conversation))
        .route("/api/v1/dm/:conversation_id/participants", get(handlers::dm::get_dm_participants))
        .route("/api/v1/dm/:conversation_id/messages", get(handlers::dm::list_dm_messages))
        .route("/api/v1/dm/:conversation_id/messages", post(handlers::dm::send_dm_message))
        .route("/api/v1/dm/:conversation_id/read", post(handlers::dm::mark_dm_as_read))
        .route("/api/v1/dm/:conversation_id/leave", post(handlers::dm::leave_dm_conversation))
        // Pin routes
        .route(
            "/api/v1/channels/:channel_id/pins",
            post(handlers::pin::pin_message),
        )
        .route(
            "/api/v1/channels/:channel_id/pins/:message_id",
            delete(handlers::pin::unpin_message),
        )
        .route(
            "/api/v1/channels/:channel_id/pins",
            get(handlers::pin::list_pins),
        )
        // Reaction routes
        .route(
            "/api/v1/messages/:message_id/reactions",
            post(handlers::reaction::add_reaction),
        )
        .route(
            "/api/v1/messages/:message_id/reactions/:emoji",
            delete(handlers::reaction::remove_reaction),
        )
        .route(
            "/api/v1/messages/:message_id/reactions",
            get(handlers::reaction::get_reactions),
        )
        // Message History routes
        .route(
            "/api/v1/messages/:message_id/history",
            get(handlers::message_history::get_message_history),
        )
        // Presence routes
        .route(
            "/api/v1/orgs/:org_id/presence",
            get(handlers::presence::get_online_users),
        )
        .route(
            "/api/v1/users/me/presence",
            patch(handlers::presence::update_presence),
        )
        .route(
            "/api/v1/users/me/status",
            patch(handlers::presence::update_status),
        )
        .with_state(state)
}
