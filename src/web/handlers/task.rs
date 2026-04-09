//! Task list/detail/create/edit/notes handlers.

use askama::Template;
use axum::{
  body::Bytes,
  extract::{Form, Path, Query, State},
  response::{Html, Redirect},
};
use libsql::Connection;
use serde::Deserialize;

use crate::{
  store::{
    model::{
      note,
      primitives::{EntityType, Id, Priority, RelationshipType, TaskStatus},
      relationship, task,
    },
    repo,
  },
  web::{
    AppState,
    forms::{self, ExistingLink, NoteFormData},
    handlers::{self, AppError, log_err},
    markdown,
    timeline::{self, TimelineItem},
  },
};

/// Query parameters for the task list view (status tab selection).
#[derive(Deserialize)]
pub struct TaskListParams {
  status: Option<String>,
}

/// A display-friendly representation of a relationship link (task detail view).
struct DisplayLink {
  display_text: String,
  href: Option<String>,
  rel: String,
}

#[derive(Template)]
#[template(path = "tasks/create.html")]
struct TaskCreateTemplate {
  priority_options: Vec<PriorityOption>,
}

#[derive(Template)]
#[template(path = "tasks/detail_content.html")]
struct TaskDetailFragmentTemplate {
  blocking: bool,
  description_html: String,
  display_links: Vec<DisplayLink>,
  is_blocked: bool,
  tags: Vec<String>,
  task: task::Model,
  timeline_items: Vec<TimelineItem>,
}

#[derive(Template)]
#[template(path = "tasks/detail.html")]
struct TaskDetailTemplate {
  blocking: bool,
  description_html: String,
  display_links: Vec<DisplayLink>,
  is_blocked: bool,
  tags: Vec<String>,
  task: task::Model,
  timeline_items: Vec<TimelineItem>,
}

#[derive(Template)]
#[template(path = "tasks/edit.html")]
struct TaskEditTemplate {
  description: String,
  error: Option<String>,
  existing_links: Vec<ExistingLink>,
  priority_options: Vec<PriorityOption>,
  tags: String,
  task: task::Model,
  title: String,
}

#[derive(Template)]
#[template(path = "tasks/list_content.html")]
struct TaskListFragmentTemplate {
  cancelled_count: usize,
  current_status: String,
  done_count: usize,
  in_progress_count: usize,
  open_count: usize,
  rows: Vec<TaskRow>,
}

#[derive(Template)]
#[template(path = "tasks/list.html")]
struct TaskListTemplate {
  cancelled_count: usize,
  current_status: String,
  done_count: usize,
  in_progress_count: usize,
  open_count: usize,
  rows: Vec<TaskRow>,
}

/// Dropdown option for the task priority `<select>`.
struct PriorityOption {
  label: String,
  selected: bool,
  value: String,
}

/// Enriched row for the task list view.
struct TaskRow {
  blocked_by_display: String,
  blocking: bool,
  is_blocked: bool,
  tags: Vec<String>,
  task: task::Model,
}

/// Add a note to a task.
pub async fn note_add(
  State(state): State<AppState>,
  Path(id): Path<String>,
  Form(form): Form<NoteFormData>,
) -> handlers::Result<Redirect> {
  log::debug!("note_add: task={id}");
  let conn = state.store().connect().await.map_err(log_err("note_add"))?;
  let task_id = repo::resolve::resolve_id(&conn, "tasks", &id)
    .await
    .map_err(log_err("note_add"))?;

  let new = note::New {
    body: form.body,
    author_id: state.author_id().clone(),
  };
  repo::note::create(&conn, EntityType::Task, &task_id, &new)
    .await
    .map_err(log_err("note_add"))?;

  let _ = state.reload_tx().send(());
  Ok(Redirect::to(&format!("/tasks/{}", task_id)))
}

/// Task create form.
pub async fn task_create_form() -> handlers::Result<Html<String>> {
  let tmpl = TaskCreateTemplate {
    priority_options: build_priority_options(None),
  };
  Ok(Html(tmpl.render().map_err(log_err("task_create_form"))?))
}

/// Handle task creation from form.
pub async fn task_create_submit(State(state): State<AppState>, body: Bytes) -> handlers::Result<Redirect> {
  let mut title = String::new();
  let mut description = String::new();
  let mut priority_str = String::new();
  for (key, value) in form_urlencoded::parse(&body) {
    match key.as_ref() {
      "title" => title = value.into_owned(),
      "description" => description = value.into_owned(),
      "priority" => priority_str = value.into_owned(),
      _ => {}
    }
  }
  log::debug!("task_create_submit: title={title}");

  let priority: Option<u8> = if priority_str.is_empty() {
    None
  } else {
    let byte: u8 = priority_str.parse().map_err(|_| {
      log::error!("task_create_submit: invalid priority: {priority_str}");
      AppError::BadRequest("invalid priority".to_owned())
    })?;
    Priority::try_from(byte).map_err(|_| {
      log::error!("task_create_submit: priority out of range: {byte}");
      AppError::BadRequest("invalid priority".to_owned())
    })?;
    Some(byte)
  };

  let conn = state.store().connect().await.map_err(log_err("task_create_submit"))?;
  let new = task::New {
    description,
    priority,
    title,
    ..Default::default()
  };
  let task = repo::task::create(&conn, state.project_id(), &new)
    .await
    .map_err(log_err("task_create_submit"))?;
  let _ = state.reload_tx().send(());
  Ok(Redirect::to(&format!("/tasks/{}", task.id())))
}

/// Task detail page.
pub async fn task_detail(State(state): State<AppState>, Path(id): Path<String>) -> handlers::Result<Html<String>> {
  let conn = state.store().connect().await.map_err(log_err("task_detail"))?;
  let task_id = repo::resolve::resolve_id(&conn, "tasks", &id)
    .await
    .map_err(log_err("task_detail"))?;
  let task = repo::task::find_by_id(&conn, task_id.clone())
    .await
    .map_err(log_err("task_detail"))?
    .ok_or_else(|| {
      log::error!("task_detail: task not found: {id}");
      AppError::NotFound
    })?;

  let (tags, description_html, is_blocked, blocking, display_links) =
    load_task_detail_data(&conn, &task_id, &task).await?;
  let timeline_items = timeline::build_timeline(&conn, EntityType::Task, &task_id).await?;

  let tmpl = TaskDetailTemplate {
    task,
    tags,
    description_html,
    is_blocked,
    blocking,
    display_links,
    timeline_items,
  };
  Ok(Html(tmpl.render().map_err(log_err("task_detail"))?))
}

/// Task detail fragment (for SSE live reload).
pub async fn task_detail_fragment(
  State(state): State<AppState>,
  Path(id): Path<String>,
) -> handlers::Result<Html<String>> {
  let conn = state.store().connect().await.map_err(log_err("task_detail_fragment"))?;
  let task_id = repo::resolve::resolve_id(&conn, "tasks", &id)
    .await
    .map_err(log_err("task_detail_fragment"))?;
  let task = repo::task::find_by_id(&conn, task_id.clone())
    .await
    .map_err(log_err("task_detail_fragment"))?
    .ok_or_else(|| {
      log::error!("task_detail_fragment: task not found: {id}");
      AppError::NotFound
    })?;

  let (tags, description_html, is_blocked, blocking, display_links) =
    load_task_detail_data(&conn, &task_id, &task).await?;
  let timeline_items = timeline::build_timeline(&conn, EntityType::Task, &task_id).await?;

  let tmpl = TaskDetailFragmentTemplate {
    task,
    tags,
    description_html,
    is_blocked,
    blocking,
    display_links,
    timeline_items,
  };
  Ok(Html(tmpl.render().map_err(log_err("task_detail_fragment"))?))
}

/// Task edit form.
pub async fn task_edit_form(State(state): State<AppState>, Path(id): Path<String>) -> handlers::Result<Html<String>> {
  let conn = state.store().connect().await.map_err(log_err("task_edit_form"))?;
  let task_id = repo::resolve::resolve_id(&conn, "tasks", &id)
    .await
    .map_err(log_err("task_edit_form"))?;
  let task = repo::task::find_by_id(&conn, task_id.clone())
    .await
    .map_err(log_err("task_edit_form"))?
    .ok_or_else(|| {
      log::error!("task_edit_form: task not found: {id}");
      AppError::NotFound
    })?;

  let tags = repo::tag::for_entity(&conn, EntityType::Task, &task_id)
    .await
    .map_err(log_err("task_edit_form"))?;

  let rels = repo::relationship::for_entity(&conn, EntityType::Task, &task_id)
    .await
    .map_err(log_err("task_edit_form"))?;
  let existing_links = forms::build_existing_links_for_entity(&task_id, EntityType::Task, &rels);

  let priority_options = build_priority_options(task.priority());
  let tmpl = TaskEditTemplate {
    title: task.title().to_owned(),
    description: task.description().to_owned(),
    priority_options,
    tags: tags.join(", "),
    task,
    error: None,
    existing_links,
  };
  Ok(Html(tmpl.render().map_err(log_err("task_edit_form"))?))
}

/// Task list page.
pub async fn task_list(
  State(state): State<AppState>,
  Query(params): Query<TaskListParams>,
) -> handlers::Result<Html<String>> {
  let (rows, open_count, in_progress_count, done_count, cancelled_count, current_status) =
    build_task_list_data(&state, params.status).await?;

  let tmpl = TaskListTemplate {
    rows,
    open_count,
    in_progress_count,
    done_count,
    cancelled_count,
    current_status,
  };
  Ok(Html(tmpl.render().map_err(log_err("task_list"))?))
}

/// Task list fragment (for SSE live reload).
pub async fn task_list_fragment(
  State(state): State<AppState>,
  Query(params): Query<TaskListParams>,
) -> handlers::Result<Html<String>> {
  let (rows, open_count, in_progress_count, done_count, cancelled_count, current_status) =
    build_task_list_data(&state, params.status).await?;

  let tmpl = TaskListFragmentTemplate {
    rows,
    open_count,
    in_progress_count,
    done_count,
    cancelled_count,
    current_status,
  };
  Ok(Html(tmpl.render().map_err(log_err("task_list_fragment"))?))
}

/// Handle task update from edit form.
pub async fn task_update(
  State(state): State<AppState>,
  Path(id): Path<String>,
  body: Bytes,
) -> handlers::Result<Redirect> {
  log::debug!("task_update: task={id}");
  let conn = state.store().connect().await.map_err(log_err("task_update"))?;
  let task_id = repo::resolve::resolve_id(&conn, "tasks", &id)
    .await
    .map_err(log_err("task_update"))?;

  // Parse form fields from raw body
  let mut title = String::new();
  let mut description = String::new();
  let mut status_str = String::new();
  let mut priority_str = String::new();
  let mut tags_str = String::new();
  let (link_rels, link_refs) = forms::extract_link_fields(&body);
  for (key, value) in form_urlencoded::parse(&body) {
    match key.as_ref() {
      "title" => title = value.into_owned(),
      "description" => description = value.into_owned(),
      "status" => status_str = value.into_owned(),
      "priority" => priority_str = value.into_owned(),
      "tags" => tags_str = value.into_owned(),
      _ => {}
    }
  }

  let status: Option<TaskStatus> = if status_str.is_empty() {
    None
  } else {
    Some(status_str.parse().map_err(|e: String| {
      log::error!("task_update: invalid status: {e}");
      AppError::BadRequest(format!("invalid status: {e}"))
    })?)
  };

  let priority: Option<Option<u8>> = if priority_str.is_empty() {
    Some(None)
  } else {
    let val: u8 = priority_str.parse().map_err(|_| {
      log::error!("task_update: invalid priority: {priority_str}");
      AppError::BadRequest("invalid priority".to_owned())
    })?;
    Priority::try_from(val).map_err(|_| {
      log::error!("task_update: priority out of range: {val}");
      AppError::BadRequest("invalid priority".to_owned())
    })?;
    Some(Some(val))
  };

  let patch = task::Patch {
    title: Some(title),
    description: Some(description),
    status,
    priority,
    ..Default::default()
  };

  repo::task::update(&conn, &task_id, &patch)
    .await
    .map_err(log_err("task_update"))?;

  // Update tags: detach all then re-attach
  repo::tag::detach_all(&conn, EntityType::Task, &task_id)
    .await
    .map_err(log_err("task_update"))?;

  for tag in tags_str.split(',').map(|t| t.trim()).filter(|t| !t.is_empty()) {
    repo::tag::attach(&conn, EntityType::Task, &task_id, tag)
      .await
      .map_err(log_err("task_update"))?;
  }

  // Sync relationships
  forms::sync_form_links(&conn, EntityType::Task, &task_id, &link_rels, &link_refs).await?;

  let _ = state.reload_tx().send(());
  Ok(Redirect::to(&format!("/tasks/{}", task_id)))
}

/// Build display links from relationships for detail view.
fn build_display_links(task_id: &Id, rels: &[relationship::Model]) -> Vec<DisplayLink> {
  let mut links = Vec::new();
  for rel in rels {
    let (rel_label, other_id, other_type) = if rel.source_id() == task_id {
      (rel.rel_type().to_string(), rel.target_id().clone(), rel.target_type())
    } else {
      (
        rel.rel_type().inverse().to_string(),
        rel.source_id().clone(),
        rel.source_type(),
      )
    };

    let href = match other_type {
      EntityType::Task => Some(format!("/tasks/{}", other_id)),
      EntityType::Artifact => Some(format!("/artifacts/{}", other_id)),
      _ => None,
    };

    links.push(DisplayLink {
      rel: rel_label,
      display_text: other_id.short(),
      href,
    });
  }
  links
}

/// Build the list of priority options rendered in the task create/edit forms.
///
/// `current` is the task's current priority (or `None` for the create form).
/// The resulting list always contains a leading "no priority" option followed
/// by one option per [`Priority::ALL`] variant.
fn build_priority_options(current: Option<u8>) -> Vec<PriorityOption> {
  let mut options = Vec::with_capacity(Priority::ALL.len() + 1);
  options.push(PriorityOption {
    label: "— none —".to_owned(),
    selected: current.is_none(),
    value: String::new(),
  });
  for priority in Priority::ALL {
    let byte: u8 = (*priority).into();
    options.push(PriorityOption {
      label: priority.to_string(),
      selected: current == Some(byte),
      value: byte.to_string(),
    });
  }
  options
}

/// Build task list data: rows filtered by status plus unfiltered per-status counts.
///
/// When `status_param` is `None`, defaults to `open`. The special value `all` bypasses
/// status filtering. Count values are always computed across every task in the project
/// so status-tab badges remain stable regardless of the current filter.
async fn build_task_list_data(
  state: &AppState,
  status_param: Option<String>,
) -> handlers::Result<(Vec<TaskRow>, usize, usize, usize, usize, String)> {
  let conn = state.store().connect().await.map_err(log_err("build_task_list_data"))?;

  let all_tasks = repo::task::all(&conn, state.project_id(), &task::Filter::all())
    .await
    .map_err(log_err("build_task_list_data"))?;

  let (open_count, in_progress_count, done_count, cancelled_count) = count_tasks_by_status(&all_tasks);

  let current_status = status_param.unwrap_or_else(|| "all".to_owned());

  let filter = match current_status.as_str() {
    "all" => task::Filter::all(),
    s => task::Filter {
      status: s.parse::<TaskStatus>().ok(),
      ..task::Filter::all()
    },
  };

  let tasks = repo::task::all(&conn, state.project_id(), &filter)
    .await
    .map_err(log_err("build_task_list_data"))?;
  let rows = build_task_rows(&conn, tasks).await?;

  Ok((
    rows,
    open_count,
    in_progress_count,
    done_count,
    cancelled_count,
    current_status,
  ))
}

/// Build enriched task rows from a list of tasks.
async fn build_task_rows(conn: &Connection, tasks: Vec<task::Model>) -> handlers::Result<Vec<TaskRow>> {
  let mut rows = Vec::with_capacity(tasks.len());
  for task in tasks {
    let task_id = task.id().clone();
    let tags = repo::tag::for_entity(conn, EntityType::Task, &task_id)
      .await
      .map_err(log_err("build_task_rows"))?;
    let rels = repo::relationship::for_entity(conn, EntityType::Task, &task_id)
      .await
      .map_err(log_err("build_task_rows"))?;

    let (is_blocked, blocking, blocked_by_display) = compute_blocking(&task_id, &rels);

    rows.push(TaskRow {
      task,
      tags,
      is_blocked,
      blocking,
      blocked_by_display,
    });
  }
  Ok(rows)
}

/// Determine blocked/blocking status and build a display string for "blocked by" tasks.
fn compute_blocking(task_id: &Id, rels: &[relationship::Model]) -> (bool, bool, String) {
  let mut is_blocked = false;
  let mut blocking = false;
  let mut blocked_by_ids = Vec::new();

  for rel in rels {
    match rel.rel_type() {
      RelationshipType::BlockedBy if rel.source_id() == task_id => {
        // This task is blocked by the target
        is_blocked = true;
        blocked_by_ids.push(rel.target_id().short());
      }
      RelationshipType::Blocks if rel.source_id() == task_id => {
        // This task blocks the target
        blocking = true;
      }
      _ => {}
    }
  }

  let blocked_by_display = if blocked_by_ids.is_empty() {
    String::new()
  } else {
    format!("blocked by {}", blocked_by_ids.join(", "))
  };

  (is_blocked, blocking, blocked_by_display)
}

/// Count tasks grouped by status, returned as `(open, in_progress, done, cancelled)`.
fn count_tasks_by_status(tasks: &[task::Model]) -> (usize, usize, usize, usize) {
  let mut open = 0;
  let mut in_progress = 0;
  let mut done = 0;
  let mut cancelled = 0;
  for task in tasks {
    match task.status() {
      TaskStatus::Open => open += 1,
      TaskStatus::InProgress => in_progress += 1,
      TaskStatus::Done => done += 1,
      TaskStatus::Cancelled => cancelled += 1,
    }
  }
  (open, in_progress, done, cancelled)
}

/// Load and build common task detail data.
async fn load_task_detail_data(
  conn: &Connection,
  task_id: &Id,
  task: &task::Model,
) -> handlers::Result<(Vec<String>, String, bool, bool, Vec<DisplayLink>)> {
  let tags = repo::tag::for_entity(conn, EntityType::Task, task_id)
    .await
    .map_err(log_err("load_task_detail_data"))?;
  let rels = repo::relationship::for_entity(conn, EntityType::Task, task_id)
    .await
    .map_err(log_err("load_task_detail_data"))?;

  let description_html = if task.description().is_empty() {
    String::new()
  } else {
    markdown::render_markdown_to_html(task.description())
  };

  let (is_blocked, blocking, _) = compute_blocking(task_id, &rels);
  let display_links = build_display_links(task_id, &rels);

  Ok((tags, description_html, is_blocked, blocking, display_links))
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::store::{self, model::Project};

  async fn setup() -> AppState {
    let (store, tmp) = store::open_temp().await.unwrap();
    let conn = store.connect().await.unwrap();
    let project = Project::new("/tmp/web-task-test".into());
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

    // Seed: one task per status
    for (title, status) in [
      ("Open A", TaskStatus::Open),
      ("Open B", TaskStatus::Open),
      ("In progress", TaskStatus::InProgress),
      ("Done", TaskStatus::Done),
      ("Cancelled", TaskStatus::Cancelled),
    ] {
      let new = task::New {
        title: title.into(),
        status: Some(status),
        ..Default::default()
      };
      repo::task::create(&conn, &project_id, &new).await.unwrap();
    }
    // Leak the tempdir for the duration of the test process
    std::mem::forget(tmp);
    AppState::new(store, project_id)
  }

  mod build_priority_options {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_emits_a_leading_none_option_followed_by_every_priority_variant() {
      let options = build_priority_options(None);

      assert_eq!(options.len(), Priority::ALL.len() + 1);
      assert_eq!(options[0].value, "");
      assert_eq!(options[0].label, "— none —");
      assert!(options[0].selected);
      for (i, priority) in Priority::ALL.iter().enumerate() {
        let option = &options[i + 1];
        let byte: u8 = (*priority).into();
        assert_eq!(option.value, byte.to_string());
        assert_eq!(option.label, priority.to_string());
        assert!(!option.selected);
      }
    }

    #[test]
    fn it_selects_the_matching_option_when_a_current_priority_is_given() {
      let options = build_priority_options(Some(2));

      let selected: Vec<_> = options.iter().filter(|o| o.selected).collect();
      assert_eq!(selected.len(), 1);
      assert_eq!(selected[0].value, "2");
      assert_eq!(selected[0].label, "medium");
    }
  }

  mod task_create_submit {
    use axum::{
      body::{Bytes, to_bytes},
      extract::State,
      http::StatusCode,
      response::IntoResponse,
    };
    use pretty_assertions::assert_eq;

    use super::*;

    #[tokio::test]
    async fn it_rejects_an_out_of_range_priority_with_a_bad_request_html_response() {
      let state = setup().await;

      let body = Bytes::from("title=bad&description=&priority=9");
      let err = super::super::task_create_submit(State(state), body)
        .await
        .expect_err("priority=9 is out of Priority range and must fail");

      let response = err.into_response();
      let status = response.status();
      let body_bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
      let html = String::from_utf8(body_bytes.to_vec()).unwrap();

      assert_eq!(status, StatusCode::BAD_REQUEST);
      assert!(html.contains("<html"));
      assert!(html.contains("invalid priority"));
    }

    #[tokio::test]
    async fn it_treats_an_empty_priority_as_no_priority_and_redirects() {
      let state = setup().await;

      let body = Bytes::from("title=no-priority&description=&priority=");
      let redirect = super::super::task_create_submit(State(state), body)
        .await
        .expect("empty priority should succeed");
      let response = redirect.into_response();

      assert_eq!(response.status(), StatusCode::SEE_OTHER);
    }
  }

  mod task_detail_error_response {
    use axum::{
      body::to_bytes,
      extract::{Path, State},
      http::StatusCode,
      response::IntoResponse,
    };
    use pretty_assertions::assert_eq;

    use super::*;

    #[tokio::test]
    async fn it_renders_html_500_when_the_handler_propagates_an_internal_error() {
      let state = setup().await;

      let err = task_detail(State(state), Path("!!!-not-a-valid-prefix".into()))
        .await
        .expect_err("invalid id prefix should propagate as an internal error");
      let response = err.into_response();
      let status = response.status();
      let body_bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
      let body = String::from_utf8(body_bytes.to_vec()).unwrap();

      assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
      assert!(body.contains("<html"));
      assert!(body.contains("500"));
    }
  }

  mod build_task_list_data {
    use pretty_assertions::assert_eq;

    use super::*;

    #[tokio::test]
    async fn it_defaults_to_all_when_no_status_given() {
      let state = setup().await;

      let (rows, open, in_prog, done, cancelled, current) = build_task_list_data(&state, None).await.unwrap();

      assert_eq!(current, "all");
      assert_eq!(rows.len(), 5);
      assert_eq!((open, in_prog, done, cancelled), (2, 1, 1, 1));
    }

    #[tokio::test]
    async fn it_filters_to_a_specific_status_at_the_db_layer() {
      let state = setup().await;

      let (rows, _, _, _, _, current) = build_task_list_data(&state, Some("done".into())).await.unwrap();

      assert_eq!(current, "done");
      assert_eq!(rows.len(), 1);
      assert_eq!(rows[0].task.status(), TaskStatus::Done);
    }

    #[tokio::test]
    async fn it_reports_counts_across_every_status_regardless_of_filter() {
      let state = setup().await;

      let (_, open_a, in_prog_a, done_a, cancelled_a, _) = build_task_list_data(&state, None).await.unwrap();
      let (_, open_b, in_prog_b, done_b, cancelled_b, _) =
        build_task_list_data(&state, Some("done".into())).await.unwrap();

      assert_eq!((open_a, in_prog_a, done_a, cancelled_a), (2, 1, 1, 1));
      assert_eq!((open_b, in_prog_b, done_b, cancelled_b), (2, 1, 1, 1));
    }

    #[tokio::test]
    async fn it_returns_every_task_when_status_is_all() {
      let state = setup().await;

      let (rows, open, in_prog, done, cancelled, current) =
        build_task_list_data(&state, Some("all".into())).await.unwrap();

      assert_eq!(current, "all");
      assert_eq!(rows.len(), 5);
      assert_eq!((open, in_prog, done, cancelled), (2, 1, 1, 1));
    }
  }

  mod task_detail_timeline {
    use pretty_assertions::assert_eq;

    use crate::{
      store::{
        self,
        model::{
          Project, note,
          primitives::{EntityType, TaskStatus},
          task,
        },
        repo,
      },
      web::{AppState, timeline},
    };

    #[tokio::test]
    async fn it_filters_out_events_with_null_semantic_type() {
      let (store_arc, tmp) = store::open_temp().await.unwrap();
      let conn = store_arc.connect().await.unwrap();
      let project = Project::new("/tmp/web-task-timeline-nullfilter".into());
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

      let task = repo::task::create(
        &conn,
        &project_id,
        &task::New {
          title: "Task".into(),
          ..Default::default()
        },
      )
      .await
      .unwrap();

      // A plain modified event (no semantic_type) should NOT appear.
      let tx = repo::transaction::begin(&conn, &project_id, "task update")
        .await
        .unwrap();
      repo::transaction::record_event(&conn, tx.id(), "tasks", &task.id().to_string(), "modified", None)
        .await
        .unwrap();

      let items = timeline::build_timeline(&conn, EntityType::Task, task.id())
        .await
        .unwrap();

      assert!(items.is_empty());
    }

    #[tokio::test]
    async fn it_merges_notes_and_semantic_events_for_a_task_in_chronological_order() {
      let (store_arc, tmp) = store::open_temp().await.unwrap();
      let conn = store_arc.connect().await.unwrap();
      let project = Project::new("/tmp/web-task-timeline".into());
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

      let task = repo::task::create(
        &conn,
        &project_id,
        &task::New {
          title: "Task".into(),
          ..Default::default()
        },
      )
      .await
      .unwrap();

      // Record a semantic create event.
      let tx = repo::transaction::begin(&conn, &project_id, "task create")
        .await
        .unwrap();
      repo::transaction::record_semantic_event(
        &conn,
        tx.id(),
        "tasks",
        &task.id().to_string(),
        "created",
        None,
        Some("created"),
        None,
        None,
      )
      .await
      .unwrap();

      // Add a note via the repo (simulating a web note_add).
      repo::note::create(
        &conn,
        EntityType::Task,
        task.id(),
        &note::New {
          body: "first note".into(),
          author_id: None,
        },
      )
      .await
      .unwrap();

      // A second semantic event: status change.
      let tx2 = repo::transaction::begin(&conn, &project_id, "task claim")
        .await
        .unwrap();
      let _ = repo::task::update(
        &conn,
        task.id(),
        &task::Patch {
          status: Some(TaskStatus::InProgress),
          ..Default::default()
        },
      )
      .await
      .unwrap();
      repo::transaction::record_semantic_event(
        &conn,
        tx2.id(),
        "tasks",
        &task.id().to_string(),
        "modified",
        None,
        Some("status-change"),
        Some("open"),
        Some("in-progress"),
      )
      .await
      .unwrap();

      // Prevent the temp store from dropping.
      let _state = AppState::new(store_arc.clone(), project_id.clone());
      let items = timeline::build_timeline(&conn, EntityType::Task, task.id())
        .await
        .unwrap();

      assert_eq!(items.len(), 3);
      // First item should be the created event.
      assert!(items[0].as_event().is_some());
      // Second should be the note.
      assert!(items[1].as_note().is_some());
      // Third should be the status-change event.
      assert_eq!(
        items[2].as_event().unwrap().display_text,
        "someone changed status from open to in-progress"
      );
    }
  }
}
