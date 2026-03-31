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
    ArtifactDetailTemplate, ArtifactListTemplate, DashboardTemplate, IterationBoardTemplate, IterationDetailTemplate,
    IterationListTemplate, PhaseGroup, SearchTemplate, TaskDetailTemplate, TaskListTemplate, TaskRow,
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
      let is_blocked = !blocking.blocked_by_ids.is_empty();
      TaskRow {
        task: task.clone(),
        blocking,
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
    Err(_) => {
      return (StatusCode::NOT_FOUND, Html("<p>404 — task not found</p>")).into_response();
    }
  };

  let task = match store::read_task(&state.settings, &id) {
    Ok(t) => t,
    Err(e) => {
      log::error!("failed to read task {id}: {e}");
      return (StatusCode::INTERNAL_SERVER_ERROR, Html(format!("<p>error: {e}</p>"))).into_response();
    }
  };

  let blocking = store::resolve_blocking(&state.settings, &task);
  let is_blocked = !blocking.blocked_by_ids.is_empty();

  let description_html = render_markdown(&task.description);

  TaskDetailTemplate {
    task,
    blocking,
    is_blocked,
    description_html,
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
    Err(_) => {
      return (StatusCode::NOT_FOUND, Html("<p>artifact not found</p>")).into_response();
    }
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
pub async fn iteration_list(State(state): State<ServerState>) -> Response {
  let filter = IterationFilter {
    all: true,
    ..Default::default()
  };

  match store::list_iterations(&state.settings, &filter) {
    Ok(iterations) => IterationListTemplate {
      iterations,
    }
    .into_response(),
    Err(e) => {
      log::error!("failed to list iterations: {e}");
      (
        StatusCode::INTERNAL_SERVER_ERROR,
        Html("<p>failed to load iterations</p>"),
      )
        .into_response()
    }
  }
}

/// GET /iterations/:id — iteration detail with phase graph.
pub async fn iteration_detail(State(state): State<ServerState>, Path(id_str): Path<String>) -> Response {
  let id = match store::resolve_iteration_id(&state.settings, &id_str, true) {
    Ok(id) => id,
    Err(_) => {
      return (StatusCode::NOT_FOUND, Html("<p>iteration not found</p>")).into_response();
    }
  };

  let iteration = match store::read_iteration(&state.settings, &id) {
    Ok(it) => it,
    Err(e) => {
      log::error!("failed to read iteration {id}: {e}");
      return (StatusCode::INTERNAL_SERVER_ERROR, Html(format!("<p>error: {e}</p>"))).into_response();
    }
  };

  let tasks = store::read_iteration_tasks(&state.settings, &iteration);

  // Group tasks by phase
  let mut phase_map: std::collections::BTreeMap<u16, Vec<crate::model::Task>> = std::collections::BTreeMap::new();
  for task in &tasks {
    let phase = task.phase.unwrap_or(0);
    phase_map.entry(phase).or_default().push(task.clone());
  }
  let phases: Vec<PhaseGroup> = phase_map
    .into_iter()
    .map(|(number, tasks)| PhaseGroup {
      number,
      tasks,
    })
    .collect();

  IterationDetailTemplate {
    iteration,
    tasks,
    phases,
  }
  .into_response()
}

/// GET /iterations/:id/board — iteration kanban board.
pub async fn iteration_board(State(state): State<ServerState>, Path(id_str): Path<String>) -> Response {
  let id = match store::resolve_iteration_id(&state.settings, &id_str, true) {
    Ok(id) => id,
    Err(_) => {
      return (StatusCode::NOT_FOUND, Html("<p>iteration not found</p>")).into_response();
    }
  };

  let iteration = match store::read_iteration(&state.settings, &id) {
    Ok(it) => it,
    Err(e) => {
      log::error!("failed to read iteration {id}: {e}");
      return (StatusCode::INTERNAL_SERVER_ERROR, Html(format!("<p>error: {e}</p>"))).into_response();
    }
  };

  let tasks = store::read_iteration_tasks(&state.settings, &iteration);

  let open_tasks: Vec<_> = tasks.iter().filter(|t| t.status == Status::Open).cloned().collect();
  let in_progress_tasks: Vec<_> = tasks
    .iter()
    .filter(|t| t.status == Status::InProgress)
    .cloned()
    .collect();
  let done_tasks: Vec<_> = tasks.iter().filter(|t| t.status == Status::Done).cloned().collect();
  let cancelled_tasks: Vec<_> = tasks
    .iter()
    .filter(|t| t.status == Status::Cancelled)
    .cloned()
    .collect();

  IterationBoardTemplate {
    iteration,
    open_tasks,
    in_progress_tasks,
    done_tasks,
    cancelled_tasks,
  }
  .into_response()
}

/// Query parameters for the search endpoint.
#[derive(serde::Deserialize)]
pub struct SearchParams {
  #[serde(default)]
  pub q: String,
}

/// GET /search — search across all entity types.
pub async fn search(State(state): State<ServerState>, Query(params): Query<SearchParams>) -> Response {
  if params.q.is_empty() {
    return SearchTemplate {
      query: String::new(),
      tasks: Vec::new(),
      artifacts: Vec::new(),
      task_count: 0,
      artifact_count: 0,
    }
    .into_response();
  }

  let results = match store::search(&state.settings, &params.q, true) {
    Ok(r) => r,
    Err(e) => {
      log::error!("search failed: {e}");
      return (StatusCode::INTERNAL_SERVER_ERROR, Html("<p>search failed</p>")).into_response();
    }
  };
  let task_count = results.tasks.len();
  let artifact_count = results.artifacts.len();

  SearchTemplate {
    query: params.q,
    tasks: results.tasks,
    artifacts: results.artifacts,
    task_count,
    artifact_count,
  }
  .into_response()
}

/// Fallback handler for unmatched routes.
pub async fn not_found() -> Response {
  (StatusCode::NOT_FOUND, Html("<p>404 — not found</p>")).into_response()
}
