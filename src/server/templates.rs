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

#[derive(Template)]
#[template(path = "artifacts/create.html")]
pub struct ArtifactCreateTemplate {
  pub body: String,
  pub error: Option<String>,
  pub kind: String,
  pub tags: String,
  pub title: String,
}

impl IntoResponse for ArtifactCreateTemplate {
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
#[template(path = "artifacts/edit.html")]
pub struct ArtifactEditTemplate {
  pub artifact: Artifact,
  pub body: String,
  pub error: Option<String>,
  pub kind: String,
  pub tags: String,
  pub title: String,
}

impl IntoResponse for ArtifactEditTemplate {
  fn into_response(self) -> Response {
    render(&self)
  }
}

#[derive(Template)]
#[template(path = "artifacts/list.html")]
pub struct ArtifactListTemplate {
  pub archived_count: usize,
  pub artifacts: Vec<Artifact>,
  pub current_status: String,
  pub open_count: usize,
}

impl IntoResponse for ArtifactListTemplate {
  fn into_response(self) -> Response {
    render(&self)
  }
}

#[derive(Template)]
#[template(path = "dashboard.html")]
pub struct DashboardTemplate {
  pub artifact_count: usize,
  pub cancelled_count: usize,
  pub done_count: usize,
  pub in_progress_count: usize,
  pub iteration_count: usize,
  pub open_count: usize,
  pub task_count: usize,
}

impl IntoResponse for DashboardTemplate {
  fn into_response(self) -> Response {
    render(&self)
  }
}

/// Pre-rendered event for display in the task detail template.
pub struct DisplayEvent {
  pub author: String,
  pub avatar_url: String,
  pub created_at: String,
  pub description: String,
  pub is_agent: bool,
}

pub struct DisplayLink {
  pub display_text: String,
  pub href: Option<String>,
  pub rel: RelationshipType,
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
#[template(path = "iterations/board.html")]
pub struct IterationBoardTemplate {
  pub cancelled_tasks: Vec<Task>,
  pub done_tasks: Vec<Task>,
  pub in_progress_tasks: Vec<Task>,
  pub iteration: Iteration,
  pub open_tasks: Vec<Task>,
}

impl IntoResponse for IterationBoardTemplate {
  fn into_response(self) -> Response {
    render(&self)
  }
}

#[derive(Template)]
#[template(path = "iterations/detail.html")]
pub struct IterationDetailTemplate {
  pub iteration: Iteration,
  pub phases: Vec<PhaseGroup>,
  pub tasks: Vec<Task>,
}

impl IntoResponse for IterationDetailTemplate {
  fn into_response(self) -> Response {
    render(&self)
  }
}

#[derive(Template)]
#[template(path = "iterations/list.html")]
pub struct IterationListTemplate {
  pub active_count: usize,
  pub completed_count: usize,
  pub current_status: String,
  pub failed_count: usize,
  pub iterations: Vec<Iteration>,
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
#[template(path = "search.html")]
pub struct SearchTemplate {
  pub artifact_count: usize,
  pub artifacts: Vec<Artifact>,
  pub iteration_count: usize,
  pub iterations: Vec<Iteration>,
  pub query: String,
  pub task_count: usize,
  pub tasks: Vec<Task>,
}

impl IntoResponse for SearchTemplate {
  fn into_response(self) -> Response {
    render(&self)
  }
}

#[derive(Template)]
#[template(path = "tasks/create.html")]
pub struct TaskCreateTemplate {
  pub description: String,
  pub error: Option<String>,
  pub priority: String,
  pub tags: String,
  pub title: String,
}

impl IntoResponse for TaskCreateTemplate {
  fn into_response(self) -> Response {
    render(&self)
  }
}

#[derive(Template)]
#[template(path = "tasks/detail.html")]
pub struct TaskDetailTemplate {
  pub blocking: ResolvedBlocking,
  pub description_html: String,
  pub display_links: Vec<DisplayLink>,
  pub is_blocked: bool,
  pub task: Task,
  pub timeline: Vec<TimelineEntry>,
}

impl IntoResponse for TaskDetailTemplate {
  fn into_response(self) -> Response {
    render(&self)
  }
}

#[derive(Template)]
#[template(path = "tasks/edit.html")]
pub struct TaskEditTemplate {
  pub description: String,
  pub error: Option<String>,
  pub priority: String,
  pub tags: String,
  pub task: Task,
  pub title: String,
}

impl IntoResponse for TaskEditTemplate {
  fn into_response(self) -> Response {
    render(&self)
  }
}

#[derive(Template)]
#[template(path = "tasks/list.html")]
pub struct TaskListTemplate {
  pub cancelled_count: usize,
  pub current_status: Status,
  pub done_count: usize,
  pub in_progress_count: usize,
  pub open_count: usize,
  pub rows: Vec<TaskRow>,
  pub tasks: Vec<Task>,
}

impl IntoResponse for TaskListTemplate {
  fn into_response(self) -> Response {
    render(&self)
  }
}

pub struct TaskRow {
  pub blocked_by_display: String,
  pub blocking: ResolvedBlocking,
  pub is_blocked: bool,
  pub task: Task,
}

/// A timeline entry merging events and notes, sorted by time.
pub enum TimelineEntry {
  Event(DisplayEvent),
  Note(DisplayNote),
}

// ── Fragment templates (content only, no base layout) ────────────────────────

#[derive(Template)]
#[template(path = "artifacts/detail_content.html")]
pub struct ArtifactDetailFragmentTemplate {
  pub artifact: Artifact,
  pub body_html: String,
}

#[derive(Template)]
#[template(path = "artifacts/list_content.html")]
pub struct ArtifactListFragmentTemplate {
  pub archived_count: usize,
  pub artifacts: Vec<Artifact>,
  pub current_status: String,
  pub open_count: usize,
}

#[derive(Template)]
#[template(path = "dashboard_content.html")]
pub struct DashboardFragmentTemplate {
  pub artifact_count: usize,
  pub cancelled_count: usize,
  pub done_count: usize,
  pub in_progress_count: usize,
  pub iteration_count: usize,
  pub open_count: usize,
  pub task_count: usize,
}

#[derive(Template)]
#[template(path = "iterations/board_content.html")]
pub struct IterationBoardFragmentTemplate {
  pub cancelled_tasks: Vec<Task>,
  pub done_tasks: Vec<Task>,
  pub in_progress_tasks: Vec<Task>,
  pub iteration: Iteration,
  pub open_tasks: Vec<Task>,
}

#[derive(Template)]
#[template(path = "iterations/detail_content.html")]
pub struct IterationDetailFragmentTemplate {
  pub iteration: Iteration,
  pub phases: Vec<PhaseGroup>,
  pub tasks: Vec<Task>,
}

#[derive(Template)]
#[template(path = "iterations/list_content.html")]
pub struct IterationListFragmentTemplate {
  pub active_count: usize,
  pub completed_count: usize,
  pub current_status: String,
  pub failed_count: usize,
  pub iterations: Vec<Iteration>,
}

#[derive(Template)]
#[template(path = "tasks/detail_content.html")]
pub struct TaskDetailFragmentTemplate {
  pub blocking: ResolvedBlocking,
  pub description_html: String,
  pub display_links: Vec<DisplayLink>,
  pub is_blocked: bool,
  pub task: Task,
  pub timeline: Vec<TimelineEntry>,
}

#[derive(Template)]
#[template(path = "tasks/list_content.html")]
pub struct TaskListFragmentTemplate {
  pub cancelled_count: usize,
  pub current_status: Status,
  pub done_count: usize,
  pub in_progress_count: usize,
  pub open_count: usize,
  pub rows: Vec<TaskRow>,
  pub tasks: Vec<Task>,
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
