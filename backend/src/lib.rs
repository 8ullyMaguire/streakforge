// Library facade so integration tests can reference the crate's modules.
pub mod api;
pub mod auth;
pub mod config;
pub mod db;
pub mod error;

use config::Config;
use sqlx::PgPool;

/// Shared application state. Handlers use the tower-sessions `Session` extractor
/// directly, so the session layer itself does not need to live in state.
#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub cfg: Config,
    pub x_oauth: auth::XOAuth,
}
