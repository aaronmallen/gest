//! Iteration list/detail/board handlers.

use std::collections::BTreeMap;

use askama::Template;
use axum::{
  extract::{Path, Query, State},
  response::Html,
};
use serde::Deserialize;

use crate::{
  store::{
    model::{
      iteration,
      primitives::{EntityType, IterationStatus},
    },
    repo::{
      self,
      iteration::{IterationTaskRow, StatusCounts},
    },
  },
  web::AppState,
};

#[derive(Deserialize)]
pub struct IterationListParams {
  status: Option<String>,
}

#[derive(Template)]
#[template(path = "iterations/board_content.html")]
struct IterationBoardFragmentTemplate {
  cancelled_tasks: Vec<IterationTaskRow>,
  done_tasks: Vec<IterationTaskRow>,
  in_progress_tasks: Vec<IterationTaskRow>,
  iteration: iteration::Model,
  open_tasks: Vec<IterationTaskRow>,
}

#[derive(Template)]
#[template(path = "iterations/board.html")]
struct IterationBoardTemplate {
  cancelled_tasks: Vec<IterationTaskRow>,
  done_tasks: Vec<IterationTaskRow>,
  in_progress_tasks: Vec<IterationTaskRow>,
  iteration: iteration::Model,
  open_tasks: Vec<IterationTaskRow>,
}

#[derive(Template)]
#[template(path = "iterations/detail_content.html")]
struct IterationDetailFragmentTemplate {
  iteration: iteration::Model,
  phases: Vec<PhaseGroup>,
  status_counts: StatusCounts,
  tags: Vec<String>,
  task_count: i64,
}

#[derive(Template)]
#[template(path = "iterations/detail.html")]
struct IterationDetailTemplate {
  iteration: iteration::Model,
  phases: Vec<PhaseGroup>,
  status_counts: StatusCounts,
  tags: Vec<String>,
  task_count: i64,
}

#[derive(Template)]
#[template(path = "iterations/list_content.html")]
struct IterationListFragmentTemplate {
  active_count: usize,
  cancelled_count: usize,
  completed_count: usize,
  current_status: String,
  rows: Vec<IterationRow>,
}

#[derive(Template)]
#[template(path = "iterations/list.html")]
struct IterationListTemplate {
  active_count: usize,
  cancelled_count: usize,
  completed_count: usize,
  current_status: String,
  rows: Vec<IterationRow>,
}

struct IterationRow {
  iteration: iteration::Model,
  phase_count: u32,
  tags: Vec<String>,
  task_count: i64,
}

struct PhaseGroup {
  number: u32,
  tasks: Vec<IterationTaskRow>,
}

/// Iteration board page.
pub async fn iteration_board(State(state): State<AppState>, Path(id): Path<String>) -> Result<Html<String>, String> {
  let (iteration, open_tasks, in_progress_tasks, done_tasks, cancelled_tasks) =
    build_iteration_board(&state, &id).await?;

  let tmpl = IterationBoardTemplate {
    iteration,
    open_tasks,
    in_progress_tasks,
    done_tasks,
    cancelled_tasks,
  };
  Ok(Html(tmpl.render().map_err(|e| e.to_string())?))
}

/// Iteration board fragment (for SSE live reload).
pub async fn iteration_board_fragment(
  State(state): State<AppState>,
  Path(id): Path<String>,
) -> Result<Html<String>, String> {
  let (iteration, open_tasks, in_progress_tasks, done_tasks, cancelled_tasks) =
    build_iteration_board(&state, &id).await?;

  let tmpl = IterationBoardFragmentTemplate {
    iteration,
    open_tasks,
    in_progress_tasks,
    done_tasks,
    cancelled_tasks,
  };
  Ok(Html(tmpl.render().map_err(|e| e.to_string())?))
}

/// Iteration detail page.
pub async fn iteration_detail(State(state): State<AppState>, Path(id): Path<String>) -> Result<Html<String>, String> {
  let (iteration, tags, phases, task_count, status_counts) = build_iteration_detail(&state, &id).await?;

  let tmpl = IterationDetailTemplate {
    iteration,
    tags,
    phases,
    task_count,
    status_counts,
  };
  Ok(Html(tmpl.render().map_err(|e| e.to_string())?))
}

/// Iteration detail fragment (for SSE live reload).
pub async fn iteration_detail_fragment(
  State(state): State<AppState>,
  Path(id): Path<String>,
) -> Result<Html<String>, String> {
  let (iteration, tags, phases, task_count, status_counts) = build_iteration_detail(&state, &id).await?;

  let tmpl = IterationDetailFragmentTemplate {
    iteration,
    tags,
    phases,
    task_count,
    status_counts,
  };
  Ok(Html(tmpl.render().map_err(|e| e.to_string())?))
}

/// Iteration list page.
pub async fn iteration_list(
  State(state): State<AppState>,
  Query(params): Query<IterationListParams>,
) -> Result<Html<String>, String> {
  let (rows, active_count, completed_count, cancelled_count, current_status) =
    build_iteration_list(&state, &params.status).await?;

  let tmpl = IterationListTemplate {
    rows,
    active_count,
    completed_count,
    cancelled_count,
    current_status,
  };
  Ok(Html(tmpl.render().map_err(|e| e.to_string())?))
}

/// Iteration list fragment (for SSE live reload).
pub async fn iteration_list_fragment(
  State(state): State<AppState>,
  Query(params): Query<IterationListParams>,
) -> Result<Html<String>, String> {
  let (rows, active_count, completed_count, cancelled_count, current_status) =
    build_iteration_list(&state, &params.status).await?;

  let tmpl = IterationListFragmentTemplate {
    rows,
    active_count,
    completed_count,
    cancelled_count,
    current_status,
  };
  Ok(Html(tmpl.render().map_err(|e| e.to_string())?))
}

/// Build iteration board data from an iteration id.
async fn build_iteration_board(
  state: &AppState,
  id: &str,
) -> Result<
  (
    iteration::Model,
    Vec<IterationTaskRow>,
    Vec<IterationTaskRow>,
    Vec<IterationTaskRow>,
    Vec<IterationTaskRow>,
  ),
  String,
> {
  let conn = state.store().connect().await.map_err(|e| e.to_string())?;
  let iter_id = repo::resolve::resolve_id(&conn, "iterations", id)
    .await
    .map_err(|e| e.to_string())?;
  let iteration = repo::iteration::find_by_id(&conn, iter_id.clone())
    .await
    .map_err(|e| e.to_string())?
    .ok_or_else(|| format!("iteration not found: {id}"))?;

  let tasks = repo::iteration::tasks_with_phase(&conn, &iter_id)
    .await
    .map_err(|e| e.to_string())?;

  let mut open = Vec::new();
  let mut in_progress = Vec::new();
  let mut done = Vec::new();
  let mut cancelled = Vec::new();
  for t in tasks {
    match t.status.as_str() {
      "in_progress" => in_progress.push(t),
      "done" => done.push(t),
      "cancelled" => cancelled.push(t),
      _ => open.push(t),
    }
  }

  Ok((iteration, open, in_progress, done, cancelled))
}

/// Build enriched iteration detail data.
async fn build_iteration_detail(
  state: &AppState,
  id: &str,
) -> Result<(iteration::Model, Vec<String>, Vec<PhaseGroup>, i64, StatusCounts), String> {
  let conn = state.store().connect().await.map_err(|e| e.to_string())?;
  let iter_id = repo::resolve::resolve_id(&conn, "iterations", id)
    .await
    .map_err(|e| e.to_string())?;
  let iteration = repo::iteration::find_by_id(&conn, iter_id.clone())
    .await
    .map_err(|e| e.to_string())?
    .ok_or_else(|| format!("iteration not found: {id}"))?;

  let tags = repo::tag::for_entity(&conn, EntityType::Iteration, &iter_id)
    .await
    .map_err(|e| e.to_string())?;
  let tasks = repo::iteration::tasks_with_phase(&conn, &iter_id)
    .await
    .map_err(|e| e.to_string())?;
  let status_counts = repo::iteration::task_status_counts(&conn, &iter_id)
    .await
    .map_err(|e| e.to_string())?;

  // Group tasks by phase
  let mut phase_map: BTreeMap<u32, Vec<IterationTaskRow>> = BTreeMap::new();
  for t in tasks {
    phase_map.entry(t.phase).or_default().push(t);
  }
  let phases: Vec<PhaseGroup> = phase_map
    .into_iter()
    .map(|(number, tasks)| PhaseGroup {
      number,
      tasks,
    })
    .collect();

  let task_count = status_counts.total;
  Ok((iteration, tags, phases, task_count, status_counts))
}

/// Build enriched iteration list data.
async fn build_iteration_list(
  state: &AppState,
  status_param: &Option<String>,
) -> Result<(Vec<IterationRow>, usize, usize, usize, String), String> {
  let conn = state.store().connect().await.map_err(|e| e.to_string())?;

  // Fetch all iterations to compute counts
  let all_iterations = repo::iteration::all(
    &conn,
    state.project_id(),
    &iteration::Filter {
      all: true,
      ..Default::default()
    },
  )
  .await
  .map_err(|e| e.to_string())?;

  let active_count = all_iterations
    .iter()
    .filter(|i| i.status() == IterationStatus::Active)
    .count();
  let completed_count = all_iterations
    .iter()
    .filter(|i| i.status() == IterationStatus::Completed)
    .count();
  let cancelled_count = all_iterations
    .iter()
    .filter(|i| i.status() == IterationStatus::Cancelled)
    .count();

  let current_status = status_param.clone().unwrap_or_default();

  // Filter iterations based on status param
  let filter = iteration::Filter {
    all: current_status.is_empty() || current_status == "all",
    status: if current_status.is_empty() || current_status == "all" {
      None
    } else {
      current_status.parse::<IterationStatus>().ok()
    },
    ..Default::default()
  };

  let iterations = repo::iteration::all(&conn, state.project_id(), &filter)
    .await
    .map_err(|e| e.to_string())?;

  let mut rows = Vec::with_capacity(iterations.len());
  for it in iterations {
    let tags = repo::tag::for_entity(&conn, EntityType::Iteration, it.id())
      .await
      .map_err(|e| e.to_string())?;
    let counts = repo::iteration::task_status_counts(&conn, it.id())
      .await
      .map_err(|e| e.to_string())?;
    let max_phase = repo::iteration::max_phase(&conn, it.id())
      .await
      .map_err(|e| e.to_string())?;
    rows.push(IterationRow {
      task_count: counts.total,
      phase_count: max_phase.unwrap_or(0),
      tags,
      iteration: it,
    });
  }

  Ok((rows, active_count, completed_count, cancelled_count, current_status))
}
