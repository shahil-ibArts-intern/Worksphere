use std::fmt;

use sqlx::PgPool;

use crate::handlers;

/// Shared application state passed to all handlers.
#[derive(Clone)]
pub struct ApiState {
    pub db: PgPool,
    pub jwt: auth::JwtManager,
    pub services: handlers::ServiceRegistry,
}

impl ApiState {
    /// Creates a new API state.
    pub fn new(db: PgPool, jwt: auth::JwtManager) -> Self {
        Self {
            db: db.clone(),
            jwt,
            services: handlers::ServiceRegistry::new(db),
        }
    }
}

impl fmt::Debug for ApiState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ApiState")
            .field("db", &"[PgPool]")
            .field("jwt", &"[JwtManager]")
            .field("services", &"[ServiceRegistry]")
            .finish()
    }
}
