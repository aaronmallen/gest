//! Route definitions and router construction.

use axum::{
  Router,
  routing::{get, post},
};

use super::{assets, handlers, state::ServerState};

/// Build the top-level Axum router with all routes mounted.
pub fn router(state: ServerState) -> Router {
  Router::new()
    .route("/", get(handlers::dashboard))
    .route("/tasks", get(handlers::task_list).post(handlers::task_create))
    .route("/tasks/new", get(handlers::task_create_form))
    .route("/tasks/{id}/edit", get(handlers::task_edit_form))
    .route("/tasks/{id}", get(handlers::task_detail).post(handlers::task_update))
    .route(
      "/artifacts",
      get(handlers::artifact_list).post(handlers::artifact_create),
    )
    .route("/artifacts/new", get(handlers::artifact_create_form))
    .route(
      "/artifacts/{id}",
      get(handlers::artifact_detail).post(handlers::artifact_update),
    )
    .route("/artifacts/{id}/edit", get(handlers::artifact_edit_form))
    .route("/artifacts/{id}/archive", post(handlers::artifact_archive))
    .route("/iterations", get(handlers::iteration_list))
    .route("/iterations/{id}", get(handlers::iteration_detail))
    .route("/iterations/{id}/board", get(handlers::iteration_board))
    .route("/search", get(handlers::search))
    .route("/api/search", get(handlers::api_search))
    .route("/api/render-markdown", post(handlers::api_render_markdown))
    .route("/static/{*path}", get(assets::serve))
    .fallback(handlers::not_found)
    .with_state(state)
}
