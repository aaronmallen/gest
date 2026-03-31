//! Request handlers for each web view.

use axum::{
  extract::{Path, Query, State},
  http::StatusCode,
  response::{Html, IntoResponse, Response},
};

use super::state::ServerState;

/// GET / — dashboard with entity counts and navigation.
pub async fn dashboard(State(_state): State<ServerState>) -> Response {
  Html("<p>dashboard — coming soon</p>").into_response()
}

/// GET /tasks — task list.
pub async fn task_list(State(_state): State<ServerState>) -> Response {
  Html("<p>task list — coming soon</p>").into_response()
}

/// GET /tasks/:id — task detail.
pub async fn task_detail(State(_state): State<ServerState>, Path(_id): Path<String>) -> Response {
  Html("<p>task detail — coming soon</p>").into_response()
}

/// GET /artifacts — artifact list.
pub async fn artifact_list(State(_state): State<ServerState>) -> Response {
  Html("<p>artifact list — coming soon</p>").into_response()
}

/// GET /artifacts/:id — artifact detail with rendered Markdown.
pub async fn artifact_detail(State(_state): State<ServerState>, Path(_id): Path<String>) -> Response {
  Html("<p>artifact detail — coming soon</p>").into_response()
}

/// GET /iterations — iteration list.
pub async fn iteration_list(State(_state): State<ServerState>) -> Response {
  Html("<p>iteration list — coming soon</p>").into_response()
}

/// GET /iterations/:id — iteration detail with phase graph.
pub async fn iteration_detail(State(_state): State<ServerState>, Path(_id): Path<String>) -> Response {
  Html("<p>iteration detail — coming soon</p>").into_response()
}

/// GET /iterations/:id/board — iteration kanban board.
pub async fn iteration_board(State(_state): State<ServerState>, Path(_id): Path<String>) -> Response {
  Html("<p>iteration board — coming soon</p>").into_response()
}

/// Query parameters for the search endpoint.
#[derive(serde::Deserialize)]
pub struct SearchParams {
  #[serde(default)]
  pub q: String,
}

/// GET /search — search across all entity types.
pub async fn search(State(_state): State<ServerState>, Query(_params): Query<SearchParams>) -> Response {
  Html("<p>search — coming soon</p>").into_response()
}

/// Fallback handler for unmatched routes.
pub async fn not_found() -> Response {
  (StatusCode::NOT_FOUND, Html("<p>404 — not found</p>")).into_response()
}
