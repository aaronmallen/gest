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
    .route("/api/render-markdown", post(handlers::api_render_markdown))
    .route("/api/search", get(handlers::api_search))
    .route(
      "/artifacts",
      get(handlers::artifact_list).post(handlers::artifact_create),
    )
    .route("/artifacts/new", get(handlers::artifact_create_form))
    .route(
      "/artifacts/{id}",
      get(handlers::artifact_detail).post(handlers::artifact_update),
    )
    .route("/artifacts/{id}/archive", post(handlers::artifact_archive))
    .route("/events", get(handlers::events))
    .route("/artifacts/{id}/edit", get(handlers::artifact_edit_form))
    .route("/iterations", get(handlers::iteration_list))
    .route("/iterations/{id}", get(handlers::iteration_detail))
    .route("/iterations/{id}/board", get(handlers::iteration_board))
    .route("/search", get(handlers::search))
    .route("/static/{*path}", get(assets::serve))
    .route("/tasks", get(handlers::task_list).post(handlers::task_create))
    .route("/tasks/new", get(handlers::task_create_form))
    .route("/tasks/{id}", get(handlers::task_detail).post(handlers::task_update))
    .route("/tasks/{id}/edit", get(handlers::task_edit_form))
    .route("/tasks/{id}/notes", post(handlers::note_add))
    // Fragment endpoints (content only, for live reload)
    .route("/_dashboard", get(handlers::dashboard_fragment))
    .route("/artifacts/_list", get(handlers::artifact_list_fragment))
    .route("/artifacts/{id}/_detail", get(handlers::artifact_detail_fragment))
    .route("/iterations/_list", get(handlers::iteration_list_fragment))
    .route("/iterations/{id}/_board", get(handlers::iteration_board_fragment))
    .route("/iterations/{id}/_detail", get(handlers::iteration_detail_fragment))
    .route("/tasks/_list", get(handlers::task_list_fragment))
    .route("/tasks/{id}/_detail", get(handlers::task_detail_fragment))
    .fallback(handlers::not_found)
    .with_state(state)
}
