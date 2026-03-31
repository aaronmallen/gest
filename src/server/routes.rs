//! Route definitions and router construction.

use axum::Router;

use super::state::ServerState;

/// Build the top-level Axum router with all routes mounted.
pub fn router(_state: ServerState) -> Router {
  Router::new()
}
