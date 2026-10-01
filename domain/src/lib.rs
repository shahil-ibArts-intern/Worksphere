//! Domain crate — domain types, traits, errors, and shared types.
//!
//! This crate is the foundation of the application. It depends on NO other
//! workspace crates. All other crates MAY depend on `domain`.

pub mod config;
pub mod error;
pub mod events;
pub mod ids;
pub mod traits;
pub mod types;

// Re-exports for convenience
pub use config::AppConfig;
pub use error::{AppError, AppResult, ErrorResponse};
pub use ids::*;
pub use traits::*;
pub use types::*;
