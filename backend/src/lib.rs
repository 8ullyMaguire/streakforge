// Library facade so integration tests can reference the crate's modules.
pub mod account;
pub mod api;
pub mod audit;
pub mod auth;
#[cfg(test)]
pub mod auth_tests;
pub mod config;
pub mod cage;
pub mod consent;
pub mod db;
pub mod error;
pub mod fold;
pub mod guest;
pub mod manifesto;
pub mod metrics;
pub mod moderation;
pub mod router;

use config::Config;
use sqlx::PgPool;

/// Shared application state. Handlers use the tower-sessions `Session` extractor
/// directly, so the session layer itself does not need to live in state.
#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub cfg: Config,
}
