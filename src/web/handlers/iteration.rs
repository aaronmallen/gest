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
  web::{
    AppState,
    handlers::log_err,
    timeline::{self, TimelineItem},
  },
};

/// Query parameters for the iteration list view (status tab selection).
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
  timeline_items: Vec<TimelineItem>,
}

#[derive(Template)]
#[template(path = "iterations/detail.html")]
struct IterationDetailTemplate {
  iteration: iteration::Model,
  phases: Vec<PhaseGroup>,
  status_counts: StatusCounts,
  tags: Vec<String>,
  task_count: i64,
  timeline_items: Vec<TimelineItem>,
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
  Ok(Html(tmpl.render().map_err(log_err("iteration_board"))?))
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
  Ok(Html(tmpl.render().map_err(log_err("iteration_board_fragment"))?))
}

/// Iteration detail page.
pub async fn iteration_detail(State(state): State<AppState>, Path(id): Path<String>) -> Result<Html<String>, String> {
  let (iteration, tags, phases, task_count, status_counts, timeline_items) =
    build_iteration_detail(&state, &id).await?;

  let tmpl = IterationDetailTemplate {
    iteration,
    tags,
    phases,
    task_count,
    status_counts,
    timeline_items,
  };
  Ok(Html(tmpl.render().map_err(log_err("iteration_detail"))?))
}

/// Iteration detail fragment (for SSE live reload).
pub async fn iteration_detail_fragment(
  State(state): State<AppState>,
  Path(id): Path<String>,
) -> Result<Html<String>, String> {
  let (iteration, tags, phases, task_count, status_counts, timeline_items) =
    build_iteration_detail(&state, &id).await?;

  let tmpl = IterationDetailFragmentTemplate {
    iteration,
    tags,
    phases,
    task_count,
    status_counts,
    timeline_items,
  };
  Ok(Html(tmpl.render().map_err(log_err("iteration_detail_fragment"))?))
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
  Ok(Html(tmpl.render().map_err(log_err("iteration_list"))?))
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
  Ok(Html(tmpl.render().map_err(log_err("iteration_list_fragment"))?))
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
  let conn = state
    .store()
    .connect()
    .await
    .map_err(log_err("build_iteration_board"))?;
  let iter_id = repo::resolve::resolve_id(&conn, "iterations", id)
    .await
    .map_err(log_err("build_iteration_board"))?;
  let iteration = repo::iteration::find_by_id(&conn, iter_id.clone())
    .await
    .map_err(log_err("build_iteration_board"))?
    .ok_or_else(|| {
      log::error!("build_iteration_board: iteration not found: {id}");
      format!("iteration not found: {id}")
    })?;

  let tasks = repo::iteration::tasks_with_phase(&conn, &iter_id)
    .await
    .map_err(log_err("build_iteration_board"))?;

  let mut open = Vec::new();
  let mut in_progress = Vec::new();
  let mut done = Vec::new();
  let mut cancelled = Vec::new();
  for t in tasks {
    match t.status.as_str() {
      "in-progress" => in_progress.push(t),
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
) -> Result<
  (
    iteration::Model,
    Vec<String>,
    Vec<PhaseGroup>,
    i64,
    StatusCounts,
    Vec<TimelineItem>,
  ),
  String,
> {
  let conn = state
    .store()
    .connect()
    .await
    .map_err(log_err("build_iteration_detail"))?;
  let iter_id = repo::resolve::resolve_id(&conn, "iterations", id)
    .await
    .map_err(log_err("build_iteration_detail"))?;
  let iteration = repo::iteration::find_by_id(&conn, iter_id.clone())
    .await
    .map_err(log_err("build_iteration_detail"))?
    .ok_or_else(|| {
      log::error!("build_iteration_detail: iteration not found: {id}");
      format!("iteration not found: {id}")
    })?;

  let tags = repo::tag::for_entity(&conn, EntityType::Iteration, &iter_id)
    .await
    .map_err(log_err("build_iteration_detail"))?;
  let tasks = repo::iteration::tasks_with_phase(&conn, &iter_id)
    .await
    .map_err(log_err("build_iteration_detail"))?;
  let status_counts = repo::iteration::task_status_counts(&conn, &iter_id)
    .await
    .map_err(log_err("build_iteration_detail"))?;

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
  let timeline_items = timeline::build_timeline(&conn, EntityType::Iteration, &iter_id).await?;
  Ok((iteration, tags, phases, task_count, status_counts, timeline_items))
}

/// Build enriched iteration list data.
async fn build_iteration_list(
  state: &AppState,
  status_param: &Option<String>,
) -> Result<(Vec<IterationRow>, usize, usize, usize, String), String> {
  let conn = state.store().connect().await.map_err(log_err("build_iteration_list"))?;

  // Fetch all iterations to compute counts
  let all_iterations = repo::iteration::all(&conn, state.project_id(), &iteration::Filter::all())
    .await
    .map_err(log_err("build_iteration_list"))?;

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

  let current_status = status_param.clone().unwrap_or_else(|| "all".to_owned());

  // Filter iterations based on status param. Default (no param) shows every iteration;
  // a concrete status narrows to that status.
  let filter = match current_status.as_str() {
    "all" => iteration::Filter::all(),
    s => iteration::Filter {
      status: s.parse::<IterationStatus>().ok(),
      ..iteration::Filter::all()
    },
  };

  let iterations = repo::iteration::all(&conn, state.project_id(), &filter)
    .await
    .map_err(log_err("build_iteration_list"))?;

  let mut rows = Vec::with_capacity(iterations.len());
  for it in iterations {
    let tags = repo::tag::for_entity(&conn, EntityType::Iteration, it.id())
      .await
      .map_err(log_err("build_iteration_list"))?;
    let counts = repo::iteration::task_status_counts(&conn, it.id())
      .await
      .map_err(log_err("build_iteration_list"))?;
    let max_phase = repo::iteration::max_phase(&conn, it.id())
      .await
      .map_err(log_err("build_iteration_list"))?;
    rows.push(IterationRow {
      task_count: counts.total,
      phase_count: max_phase.unwrap_or(0),
      tags,
      iteration: it,
    });
  }

  Ok((rows, active_count, completed_count, cancelled_count, current_status))
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::store::{self, model::Project};

  async fn setup() -> AppState {
    let (store, tmp) = store::open_temp().await.unwrap();
    let conn = store.connect().await.unwrap();
    let project = Project::new("/tmp/web-iter-test".into());
    conn
      .execute(
        "INSERT INTO projects (id, root, created_at, updated_at) VALUES (?1, ?2, ?3, ?4)",
        [
          project.id().to_string(),
          project.root().to_string_lossy().into_owned(),
          project.created_at().to_rfc3339(),
          project.updated_at().to_rfc3339(),
        ],
      )
      .await
      .unwrap();
    let project_id = project.id().clone();

    // Seed two active, one completed, one cancelled.
    for (title, status) in [
      ("Active A", IterationStatus::Active),
      ("Active B", IterationStatus::Active),
      ("Completed", IterationStatus::Completed),
      ("Cancelled", IterationStatus::Cancelled),
    ] {
      let new = iteration::New {
        title: title.into(),
        ..Default::default()
      };
      let it = repo::iteration::create(&conn, &project_id, &new).await.unwrap();
      if status != IterationStatus::Active {
        repo::iteration::update(
          &conn,
          it.id(),
          &iteration::Patch {
            status: Some(status),
            ..Default::default()
          },
        )
        .await
        .unwrap();
      }
    }

    std::mem::forget(tmp);
    AppState::new(store, project_id)
  }

  mod build_iteration_board {
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::store::{
      model::{Project, iteration, primitives::Id},
      repo,
    };

    #[tokio::test]
    async fn it_buckets_tasks_by_status_including_in_progress() {
      let (store_arc, tmp) = store::open_temp().await.unwrap();
      let conn = store_arc.connect().await.unwrap();
      let project = Project::new("/tmp/web-iter-board".into());
      conn
        .execute(
          "INSERT INTO projects (id, root, created_at, updated_at) VALUES (?1, ?2, ?3, ?4)",
          [
            project.id().to_string(),
            project.root().to_string_lossy().into_owned(),
            project.created_at().to_rfc3339(),
            project.updated_at().to_rfc3339(),
          ],
        )
        .await
        .unwrap();
      let project_id = project.id().clone();
      std::mem::forget(tmp);

      let iter = repo::iteration::create(
        &conn,
        &project_id,
        &iteration::New {
          title: "Sprint".into(),
          ..Default::default()
        },
      )
      .await
      .unwrap();

      for (title, status) in [
        ("Open task", "open"),
        ("In progress task", "in-progress"),
        ("Done task", "done"),
        ("Cancelled task", "cancelled"),
      ] {
        let task_id = Id::new();
        let params: [String; 4] = [
          task_id.to_string(),
          project_id.to_string(),
          title.to_string(),
          status.to_string(),
        ];
        conn
          .execute(
            "INSERT INTO tasks (id, project_id, title, status) VALUES (?1, ?2, ?3, ?4)",
            params,
          )
          .await
          .unwrap();
        repo::iteration::add_task(&conn, iter.id(), &task_id, 0).await.unwrap();
      }

      let state = AppState::new(store_arc, project_id);
      let (_iter, open, in_progress, done, cancelled) =
        build_iteration_board(&state, &iter.id().to_string()).await.unwrap();

      assert_eq!(open.len(), 1);
      assert_eq!(open[0].title, "Open task");

      assert_eq!(in_progress.len(), 1);
      assert_eq!(in_progress[0].title, "In progress task");

      assert_eq!(done.len(), 1);
      assert_eq!(done[0].title, "Done task");

      assert_eq!(cancelled.len(), 1);
      assert_eq!(cancelled[0].title, "Cancelled task");
    }
  }

  mod build_iteration_list {
    use pretty_assertions::assert_eq;

    use super::*;

    #[tokio::test]
    async fn it_defaults_to_all_when_no_status_given() {
      let state = setup().await;

      let (rows, active, completed, cancelled, current) = build_iteration_list(&state, &None).await.unwrap();

      assert_eq!(current, "all");
      assert_eq!(rows.len(), 4);
      assert_eq!((active, completed, cancelled), (2, 1, 1));
    }

    #[tokio::test]
    async fn it_filters_to_a_specific_status() {
      let state = setup().await;

      let (rows, _, _, _, current) = build_iteration_list(&state, &Some("completed".into())).await.unwrap();

      assert_eq!(current, "completed");
      assert_eq!(rows.len(), 1);
      assert_eq!(rows[0].iteration.status(), IterationStatus::Completed);
    }

    #[tokio::test]
    async fn it_reports_counts_across_every_status_regardless_of_filter() {
      let state = setup().await;

      let (_, active_a, completed_a, cancelled_a, _) = build_iteration_list(&state, &None).await.unwrap();
      let (_, active_b, completed_b, cancelled_b, _) =
        build_iteration_list(&state, &Some("completed".into())).await.unwrap();

      assert_eq!((active_a, completed_a, cancelled_a), (2, 1, 1));
      assert_eq!((active_b, completed_b, cancelled_b), (2, 1, 1));
    }

    #[tokio::test]
    async fn it_returns_every_iteration_when_status_is_all() {
      let state = setup().await;

      let (rows, active, completed, cancelled, current) =
        build_iteration_list(&state, &Some("all".into())).await.unwrap();

      assert_eq!(current, "all");
      assert_eq!(rows.len(), 4);
      assert_eq!((active, completed, cancelled), (2, 1, 1));
    }
  }

  mod iteration_detail_timeline {
    use pretty_assertions::assert_eq;

    use crate::{
      store::{
        self,
        model::{Project, iteration, note, primitives::EntityType},
        repo,
      },
      web::timeline,
    };

    #[tokio::test]
    async fn it_merges_notes_and_semantic_events_in_chronological_order() {
      let (store_arc, tmp) = store::open_temp().await.unwrap();
      let conn = store_arc.connect().await.unwrap();
      let project = Project::new("/tmp/web-iter-timeline".into());
      conn
        .execute(
          "INSERT INTO projects (id, root, created_at, updated_at) VALUES (?1, ?2, ?3, ?4)",
          [
            project.id().to_string(),
            project.root().to_string_lossy().into_owned(),
            project.created_at().to_rfc3339(),
            project.updated_at().to_rfc3339(),
          ],
        )
        .await
        .unwrap();
      let project_id = project.id().clone();
      std::mem::forget(tmp);

      let iter = repo::iteration::create(
        &conn,
        &project_id,
        &iteration::New {
          title: "Sprint".into(),
          ..Default::default()
        },
      )
      .await
      .unwrap();

      let tx = repo::transaction::begin(&conn, &project_id, "iteration create")
        .await
        .unwrap();
      repo::transaction::record_semantic_event(
        &conn,
        tx.id(),
        "iterations",
        &iter.id().to_string(),
        "created",
        None,
        Some("created"),
        None,
        None,
      )
      .await
      .unwrap();

      repo::note::create(
        &conn,
        EntityType::Iteration,
        iter.id(),
        &note::New {
          body: "iteration note".into(),
          author_id: None,
        },
      )
      .await
      .unwrap();

      // Keep the store_arc alive for the query.
      let _ = store_arc.clone();
      let items = timeline::build_timeline(&conn, EntityType::Iteration, iter.id())
        .await
        .unwrap();

      assert_eq!(items.len(), 2);
      assert!(items[0].as_event().is_some());
      assert!(items[1].as_note().is_some());
    }
  }
}
