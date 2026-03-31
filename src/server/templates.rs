//! Askama HTML templates for the web UI.

use askama::Template;
use axum::{
  http::StatusCode,
  response::{Html, IntoResponse, Response},
};

use crate::{
  model::{
    Artifact, Iteration, Task,
    iteration::Status as IterationStatus,
    task::{Status, Status as TaskStatus},
  },
  store::ResolvedBlocking,
};

/// Render an Askama template into an HTML response, returning 500 on error.
pub fn render(tmpl: &impl Template) -> Response {
  match tmpl.render() {
    Ok(html) => Html(html).into_response(),
    Err(e) => {
      log::error!("template render error: {e}");
      (StatusCode::INTERNAL_SERVER_ERROR, Html("<p>template error</p>")).into_response()
    }
  }
}

// ── Dashboard ────────────────────────────────────────────────────────────────

#[derive(Template)]
#[template(path = "dashboard.html")]
pub struct DashboardTemplate {
  pub task_count: usize,
  pub artifact_count: usize,
  pub iteration_count: usize,
  pub open_count: usize,
  pub in_progress_count: usize,
  pub done_count: usize,
  pub cancelled_count: usize,
}

impl IntoResponse for DashboardTemplate {
  fn into_response(self) -> Response {
    render(&self)
  }
}

// ── Tasks ────────────────────────────────────────────────────────────────────

/// A row in the task list view, pairing a task with its resolved blocking state.
pub struct TaskRow {
  pub task: Task,
  pub blocking: ResolvedBlocking,
  pub is_blocked: bool,
}

#[derive(Template)]
#[template(path = "tasks/list.html")]
pub struct TaskListTemplate {
  pub tasks: Vec<Task>,
  pub rows: Vec<TaskRow>,
}

impl IntoResponse for TaskListTemplate {
  fn into_response(self) -> Response {
    render(&self)
  }
}

#[derive(Template)]
#[template(path = "tasks/detail.html")]
pub struct TaskDetailTemplate {
  pub task: Task,
  pub blocking: ResolvedBlocking,
  pub is_blocked: bool,
  pub description_html: String,
}

impl IntoResponse for TaskDetailTemplate {
  fn into_response(self) -> Response {
    render(&self)
  }
}

// ── Artifacts ────────────────────────────────────────────────────────────────

#[derive(Template)]
#[template(path = "artifacts/list.html")]
pub struct ArtifactListTemplate {
  pub artifacts: Vec<Artifact>,
}

impl IntoResponse for ArtifactListTemplate {
  fn into_response(self) -> Response {
    render(&self)
  }
}

#[derive(Template)]
#[template(path = "artifacts/detail.html")]
pub struct ArtifactDetailTemplate {
  pub artifact: Artifact,
  pub body_html: String,
}

impl IntoResponse for ArtifactDetailTemplate {
  fn into_response(self) -> Response {
    render(&self)
  }
}

// ── Iterations ───────────────────────────────────────────────────────────────

#[derive(Template)]
#[template(path = "iterations/list.html")]
pub struct IterationListTemplate {
  pub iterations: Vec<Iteration>,
}

impl IntoResponse for IterationListTemplate {
  fn into_response(self) -> Response {
    render(&self)
  }
}

/// A group of tasks in a single phase for the iteration detail view.
pub struct PhaseGroup {
  pub number: u16,
  pub tasks: Vec<Task>,
}

#[derive(Template)]
#[template(path = "iterations/detail.html")]
pub struct IterationDetailTemplate {
  pub iteration: Iteration,
  pub tasks: Vec<Task>,
  pub phases: Vec<PhaseGroup>,
}

impl IntoResponse for IterationDetailTemplate {
  fn into_response(self) -> Response {
    render(&self)
  }
}

#[derive(Template)]
#[template(path = "iterations/board.html")]
pub struct IterationBoardTemplate {
  pub iteration: Iteration,
  pub open_tasks: Vec<Task>,
  pub in_progress_tasks: Vec<Task>,
  pub done_tasks: Vec<Task>,
  pub cancelled_tasks: Vec<Task>,
}

impl IntoResponse for IterationBoardTemplate {
  fn into_response(self) -> Response {
    render(&self)
  }
}

// ── Search ───────────────────────────────────────────────────────────────────

#[derive(Template)]
#[template(path = "search.html")]
pub struct SearchTemplate {
  pub query: String,
  pub tasks: Vec<Task>,
  pub artifacts: Vec<Artifact>,
  pub task_count: usize,
  pub artifact_count: usize,
}

impl IntoResponse for SearchTemplate {
  fn into_response(self) -> Response {
    render(&self)
  }
}
