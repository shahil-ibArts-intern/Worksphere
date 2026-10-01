//! Database crate — SQLx repositories, migrations, and database infrastructure.
//!
//! This crate contains all SQL queries, migrations, and repository implementations.
//! It depends on `core` for types and errors.

pub mod pool;
pub mod repositories;
pub mod transaction;

pub use pool::{create_pool, create_pool_from_url, DatabaseConfig};
pub use repositories::*;
