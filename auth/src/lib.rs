//! Auth crate — JWT authentication, password hashing, sessions, and permissions.
//!
//! This crate depends on `core` and `db`.

pub mod jwt;
pub mod middleware;
pub mod oauth;
pub mod password;
pub mod permissions;
pub mod session;

pub use jwt::{AccessTokenClaims, JwtManager, RefreshTokenClaims, TokenPair};
pub use oauth::{OAuthProvider, OAuthRegistry, OAuthTokenResponse, OAuthUserProfile};
pub use password::{hash_password, verify_password};
pub use permissions::*;
pub use session::{Session, SessionManager};
