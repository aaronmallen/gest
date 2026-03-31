//! Request handlers for each web view.

use axum::{
  extract::{Path, Query, State},
  http::StatusCode,
  response::{Html, IntoResponse, Response},
};
use pulldown_cmark::{Options, Parser, html};

use super::{
  state::ServerState,
  templates::{
    ArtifactDetailTemplate, ArtifactListTemplate, DashboardTemplate, TaskDetailTemplate, TaskListTemplate, TaskRow,
  },
};
use crate::{
  model::{ArtifactFilter, IterationFilter, TaskFilter, task::Status},
  store,
};

/// Render a Markdown string to HTML using GFM extensions.
fn render_markdown(input: &str) -> String {
  let mut opts = Options::empty();
  opts.insert(Options::ENABLE_TABLES);
  opts.insert(Options::ENABLE_STRIKETHROUGH);
  opts.insert(Options::ENABLE_TASKLISTS);
  let parser = Parser::new_ext(input, opts);
  let mut html_output = String::new();
  html::push_html(&mut html_output, parser);
  html_output
}

/// GET / — dashboard with entity counts and navigation.
pub async fn dashboard(State(state): State<ServerState>) -> Response {
  let tasks = store::list_tasks(
    &state.settings,
    &TaskFilter {
      all: true,
      ..Default::default()
    },
  )
  .unwrap_or_default();

  let artifact_count = store::list_artifacts(&state.settings, &ArtifactFilter::default())
    .unwrap_or_default()
    .len();

  let iteration_count = store::list_iterations(&state.settings, &IterationFilter::default())
    .unwrap_or_default()
    .len();

  let open_count = tasks.iter().filter(|t| t.status == Status::Open).count();
  let in_progress_count = tasks.iter().filter(|t| t.status == Status::InProgress).count();
  let done_count = tasks.iter().filter(|t| t.status == Status::Done).count();
  let cancelled_count = tasks.iter().filter(|t| t.status == Status::Cancelled).count();

  DashboardTemplate {
    task_count: tasks.len(),
    artifact_count,
    iteration_count,
    open_count,
    in_progress_count,
    done_count,
    cancelled_count,
  }
  .into_response()
}

/// GET /tasks — task list.
pub async fn task_list(State(state): State<ServerState>) -> Response {
  let filter = TaskFilter {
    all: true,
    ..Default::default()
  };
  let tasks = match store::list_tasks(&state.settings, &filter) {
    Ok(t) => t,
    Err(e) => {
      log::error!("failed to list tasks: {e}");
      return (StatusCode::INTERNAL_SERVER_ERROR, Html(format!("<p>error: {e}</p>"))).into_response();
    }
  };

  let blockings = store::resolve_blocking_batch(&state.settings, &tasks);

  let rows: Vec<TaskRow> = tasks
    .iter()
    .zip(blockings)
    .map(|(task, blocking)| {
      let full_id = task.id.to_string();
      let id_rest = full_id[task.id.short().len()..].to_owned();
      let is_blocked = !blocking.blocked_by_ids.is_empty();
      TaskRow {
        task: task.clone(),
        blocking,
        id_rest,
        is_blocked,
      }
    })
    .collect();

  TaskListTemplate {
    tasks,
    rows,
  }
  .into_response()
}

/// GET /tasks/:id — task detail.
pub async fn task_detail(State(state): State<ServerState>, Path(id_str): Path<String>) -> Response {
  let id = match store::resolve_task_id(&state.settings, &id_str, true) {
    Ok(id) => id,
    Err(_) => return (StatusCode::NOT_FOUND, Html("<p>404 — task not found</p>")).into_response(),
  };

  let task = match store::read_task(&state.settings, &id) {
    Ok(t) => t,
    Err(e) => {
      log::error!("failed to read task {id}: {e}");
      return (StatusCode::INTERNAL_SERVER_ERROR, Html(format!("<p>error: {e}</p>"))).into_response();
    }
  };

  let blocking = store::resolve_blocking(&state.settings, &task);
  let full_id = task.id.to_string();
  let id_rest = full_id[task.id.short().len()..].to_owned();
  let is_blocked = !blocking.blocked_by_ids.is_empty();

  TaskDetailTemplate {
    task,
    blocking,
    id_rest,
    is_blocked,
  }
  .into_response()
}

/// GET /artifacts — artifact list.
pub async fn artifact_list(State(state): State<ServerState>) -> Response {
  let filter = ArtifactFilter {
    show_all: true,
    ..Default::default()
  };

  match store::list_artifacts(&state.settings, &filter) {
    Ok(artifacts) => ArtifactListTemplate {
      artifacts,
    }
    .into_response(),
    Err(e) => {
      log::error!("failed to list artifacts: {e}");
      (
        StatusCode::INTERNAL_SERVER_ERROR,
        Html("<p>failed to load artifacts</p>"),
      )
        .into_response()
    }
  }
}

/// GET /artifacts/:id — artifact detail with rendered Markdown.
pub async fn artifact_detail(State(state): State<ServerState>, Path(id): Path<String>) -> Response {
  let resolved = match store::resolve_artifact_id(&state.settings, &id, true) {
    Ok(id) => id,
    Err(_) => return (StatusCode::NOT_FOUND, Html("<p>artifact not found</p>")).into_response(),
  };

  match store::read_artifact(&state.settings, &resolved) {
    Ok(artifact) => {
      let body_html = render_markdown(&artifact.body);
      ArtifactDetailTemplate {
        artifact,
        body_html,
      }
      .into_response()
    }
    Err(e) => {
      log::error!("failed to read artifact {resolved}: {e}");
      (
        StatusCode::INTERNAL_SERVER_ERROR,
        Html("<p>failed to load artifact</p>"),
      )
        .into_response()
    }
  }
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
