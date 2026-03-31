//! Askama HTML templates for the web UI.

use askama::Template;
use axum::{
  http::StatusCode,
  response::{Html, IntoResponse, Response},
};

use crate::{
  model::{Artifact, Task, task::Status},
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
  pub id_rest: String,
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
  pub id_rest: String,
  pub is_blocked: bool,
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
