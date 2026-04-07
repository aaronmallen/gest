//! Dashboard and fallback handlers.

use askama::Template;
use axum::{
  extract::State,
  http::StatusCode,
  response::{Html, IntoResponse, Response},
};

use crate::{
  store::{
    model::{
      artifact, iteration,
      primitives::{IterationStatus, TaskStatus},
      task,
    },
    repo,
  },
  web::AppState,
};

#[derive(Template)]
#[template(path = "dashboard_content.html")]
struct DashboardFragmentTemplate {
  active_iteration_count: usize,
  artifact_count: usize,
  cancelled_count: usize,
  cancelled_iteration_count: usize,
  completed_iteration_count: usize,
  done_count: usize,
  in_progress_count: usize,
  iteration_count: usize,
  open_count: usize,
  task_count: usize,
}

#[derive(Template)]
#[template(path = "dashboard.html")]
struct DashboardTemplate {
  active_iteration_count: usize,
  artifact_count: usize,
  cancelled_count: usize,
  cancelled_iteration_count: usize,
  completed_iteration_count: usize,
  done_count: usize,
  in_progress_count: usize,
  iteration_count: usize,
  open_count: usize,
  task_count: usize,
}

#[derive(Template)]
#[template(path = "not_found.html")]
struct NotFoundTemplate;

/// Dashboard page showing project summary.
pub async fn dashboard(State(state): State<AppState>) -> Result<Html<String>, String> {
  let conn = state.store().connect().await.map_err(|e| e.to_string())?;
  let pid = state.project_id();

  let tasks = repo::task::all(&conn, pid, &Default::default())
    .await
    .map_err(|e| e.to_string())?;
  let artifacts = repo::artifact::all(&conn, pid, &Default::default())
    .await
    .map_err(|e| e.to_string())?;
  let iterations = repo::iteration::all(&conn, pid, &Default::default())
    .await
    .map_err(|e| e.to_string())?;

  let (
    active_iteration_count,
    artifact_count,
    cancelled_count,
    cancelled_iteration_count,
    completed_iteration_count,
    done_count,
    in_progress_count,
    iteration_count,
    open_count,
    task_count,
  ) = dashboard_counts(&tasks, &artifacts, &iterations);

  let tmpl = DashboardTemplate {
    active_iteration_count,
    artifact_count,
    cancelled_count,
    cancelled_iteration_count,
    completed_iteration_count,
    done_count,
    in_progress_count,
    iteration_count,
    open_count,
    task_count,
  };
  Ok(Html(tmpl.render().map_err(|e| e.to_string())?))
}

/// Dashboard content fragment for live reload.
pub async fn dashboard_fragment(State(state): State<AppState>) -> Result<Html<String>, String> {
  let conn = state.store().connect().await.map_err(|e| e.to_string())?;
  let pid = state.project_id();

  let tasks = repo::task::all(&conn, pid, &Default::default())
    .await
    .map_err(|e| e.to_string())?;
  let artifacts = repo::artifact::all(&conn, pid, &Default::default())
    .await
    .map_err(|e| e.to_string())?;
  let iterations = repo::iteration::all(&conn, pid, &Default::default())
    .await
    .map_err(|e| e.to_string())?;

  let (
    active_iteration_count,
    artifact_count,
    cancelled_count,
    cancelled_iteration_count,
    completed_iteration_count,
    done_count,
    in_progress_count,
    iteration_count,
    open_count,
    task_count,
  ) = dashboard_counts(&tasks, &artifacts, &iterations);

  let tmpl = DashboardFragmentTemplate {
    active_iteration_count,
    artifact_count,
    cancelled_count,
    cancelled_iteration_count,
    completed_iteration_count,
    done_count,
    in_progress_count,
    iteration_count,
    open_count,
    task_count,
  };
  Ok(Html(tmpl.render().map_err(|e| e.to_string())?))
}

/// Fallback handler for unmatched routes.
pub async fn not_found() -> Response {
  let body = NotFoundTemplate
    .render()
    .unwrap_or_else(|_| "404 — not found".to_owned());
  (StatusCode::NOT_FOUND, Html(body)).into_response()
}

/// Compute dashboard status counts from task, artifact, and iteration lists.
fn dashboard_counts(
  tasks: &[task::Model],
  artifacts: &[artifact::Model],
  iterations: &[iteration::Model],
) -> (usize, usize, usize, usize, usize, usize, usize, usize, usize, usize) {
  let mut open_count = 0;
  let mut in_progress_count = 0;
  let mut done_count = 0;
  let mut cancelled_count = 0;
  for t in tasks {
    match t.status() {
      TaskStatus::Open => open_count += 1,
      TaskStatus::InProgress => in_progress_count += 1,
      TaskStatus::Done => done_count += 1,
      TaskStatus::Cancelled => cancelled_count += 1,
    }
  }

  let mut active_iteration_count = 0;
  let mut completed_iteration_count = 0;
  let mut cancelled_iteration_count = 0;
  for i in iterations {
    match i.status() {
      IterationStatus::Active => active_iteration_count += 1,
      IterationStatus::Completed => completed_iteration_count += 1,
      IterationStatus::Cancelled => cancelled_iteration_count += 1,
    }
  }

  (
    active_iteration_count,
    artifacts.len(),
    cancelled_count,
    cancelled_iteration_count,
    completed_iteration_count,
    done_count,
    in_progress_count,
    iterations.len(),
    open_count,
    tasks.len(),
  )
}
