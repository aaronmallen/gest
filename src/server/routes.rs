//! Route definitions and router construction.

use axum::{Router, routing::get};

use super::{assets, handlers, state::ServerState};

/// Build the top-level Axum router with all routes mounted.
pub fn router(state: ServerState) -> Router {
  Router::new()
    .route("/", get(handlers::dashboard))
    .route("/tasks", get(handlers::task_list))
    .route("/tasks/{id}", get(handlers::task_detail))
    .route("/artifacts", get(handlers::artifact_list))
    .route("/artifacts/{id}", get(handlers::artifact_detail))
    .route("/iterations", get(handlers::iteration_list))
    .route("/iterations/{id}", get(handlers::iteration_detail))
    .route("/iterations/{id}/board", get(handlers::iteration_board))
    .route("/search", get(handlers::search))
    .route("/static/{*path}", get(assets::serve))
    .fallback(handlers::not_found)
    .with_state(state)
}
