//! API crate — HTTP handlers, middleware, DTOs, and routes.
//!
//! This crate depends on `core`, `services`, `auth`, and `storage`.

pub mod dto;
pub mod error;
pub mod handlers;
pub mod middleware;
pub mod routes;
pub mod state;

pub use error::app_error_to_response;
pub use routes::create_router;
pub use state::ApiState;
