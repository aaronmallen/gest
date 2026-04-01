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
    link::RelationshipType,
    task::{Status, Status as TaskStatus},
  },
  store::ResolvedBlocking,
};

pub struct DisplayLink {
  pub rel: RelationshipType,
  pub href: Option<String>,
  pub display_text: String,
}

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
  pub current_status: Status,
  pub open_count: usize,
  pub in_progress_count: usize,
  pub done_count: usize,
  pub cancelled_count: usize,
}

impl IntoResponse for TaskListTemplate {
  fn into_response(self) -> Response {
    render(&self)
  }
}

/// Pre-rendered note for display in the task detail template.
pub struct DisplayNote {
  pub author: String,
  pub avatar_url: String,
  pub body_html: String,
  pub created_at: String,
  pub id_short: String,
  pub is_agent: bool,
}

#[derive(Template)]
#[template(path = "tasks/detail.html")]
pub struct TaskDetailTemplate {
  pub task: Task,
  pub blocking: ResolvedBlocking,
  pub is_blocked: bool,
  pub description_html: String,
  pub display_links: Vec<DisplayLink>,
  pub display_notes: Vec<DisplayNote>,
}

impl IntoResponse for TaskDetailTemplate {
  fn into_response(self) -> Response {
    render(&self)
  }
}

// ── Task Create / Edit ───────────────────────────────────────────────────────

#[derive(Template)]
#[template(path = "tasks/create.html")]
pub struct TaskCreateTemplate {
  pub title: String,
  pub description: String,
  pub tags: String,
  pub priority: String,
  pub error: Option<String>,
}

impl IntoResponse for TaskCreateTemplate {
  fn into_response(self) -> Response {
    render(&self)
  }
}

#[derive(Template)]
#[template(path = "tasks/edit.html")]
pub struct TaskEditTemplate {
  pub task: Task,
  pub title: String,
  pub description: String,
  pub tags: String,
  pub priority: String,
  pub error: Option<String>,
}

impl IntoResponse for TaskEditTemplate {
  fn into_response(self) -> Response {
    render(&self)
  }
}

// ── Artifacts ────────────────────────────────────────────────────────────────

#[derive(Template)]
#[template(path = "artifacts/list.html")]
pub struct ArtifactListTemplate {
  pub artifacts: Vec<Artifact>,
  pub open_count: usize,
  pub archived_count: usize,
  pub current_status: String,
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

#[derive(Template)]
#[template(path = "artifacts/create.html")]
pub struct ArtifactCreateTemplate {
  pub title: String,
  pub kind: String,
  pub tags: String,
  pub body: String,
  pub error: Option<String>,
}

impl IntoResponse for ArtifactCreateTemplate {
  fn into_response(self) -> Response {
    render(&self)
  }
}

#[derive(Template)]
#[template(path = "artifacts/edit.html")]
pub struct ArtifactEditTemplate {
  pub artifact: Artifact,
  pub title: String,
  pub kind: String,
  pub tags: String,
  pub body: String,
  pub error: Option<String>,
}

impl IntoResponse for ArtifactEditTemplate {
  fn into_response(self) -> Response {
    render(&self)
  }
}

// ── Iterations ───────────────────────────────────────────────────────────────

#[derive(Template)]
#[template(path = "iterations/list.html")]
pub struct IterationListTemplate {
  pub iterations: Vec<Iteration>,
  pub current_status: String,
  pub active_count: usize,
  pub completed_count: usize,
  pub failed_count: usize,
}

impl IntoResponse for IterationListTemplate {
  fn into_response(self) -> Response {
    render(&self)
  }
}

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
  pub iterations: Vec<Iteration>,
  pub task_count: usize,
  pub artifact_count: usize,
  pub iteration_count: usize,
}

impl IntoResponse for SearchTemplate {
  fn into_response(self) -> Response {
    render(&self)
  }
}
