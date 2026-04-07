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
      primitives::{EntityType, Id, RelationshipType, TaskStatus},
      relationship, task,
    },
    repo,
  },
  web::{
    AppState,
    forms::{self, ExistingLink, NoteFormData},
    markdown,
    note_display::{self, NoteDisplay},
  },
};

#[derive(Deserialize)]
pub struct TaskForm {
  description: Option<String>,
  priority: Option<u8>,
  title: String,
}

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
struct TaskCreateTemplate;

#[derive(Template)]
#[template(path = "tasks/detail_content.html")]
struct TaskDetailFragmentTemplate {
  blocking: bool,
  description_html: String,
  display_links: Vec<DisplayLink>,
  is_blocked: bool,
  notes: Vec<NoteDisplay>,
  tags: Vec<String>,
  task: task::Model,
}

#[derive(Template)]
#[template(path = "tasks/detail.html")]
struct TaskDetailTemplate {
  blocking: bool,
  description_html: String,
  display_links: Vec<DisplayLink>,
  is_blocked: bool,
  notes: Vec<NoteDisplay>,
  tags: Vec<String>,
  task: task::Model,
}

#[derive(Template)]
#[template(path = "tasks/edit.html")]
struct TaskEditTemplate {
  description: String,
  error: Option<String>,
  existing_links: Vec<ExistingLink>,
  priority: String,
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
) -> Result<Redirect, String> {
  let conn = state.store().connect().await.map_err(|e| e.to_string())?;
  let task_id = repo::resolve::resolve_id(&conn, "tasks", &id)
    .await
    .map_err(|e| e.to_string())?;

  let new = note::New {
    body: form.body,
    author_id: state.author_id().cloned(),
  };
  repo::note::create(&conn, EntityType::Task, &task_id, &new)
    .await
    .map_err(|e| e.to_string())?;

  let _ = state.reload_tx().send(());
  Ok(Redirect::to(&format!("/tasks/{}", task_id)))
}

/// Task create form.
pub async fn task_create_form() -> Result<Html<String>, String> {
  let tmpl = TaskCreateTemplate;
  Ok(Html(tmpl.render().map_err(|e| e.to_string())?))
}

/// Handle task creation from form.
pub async fn task_create_submit(State(state): State<AppState>, Form(form): Form<TaskForm>) -> Result<Redirect, String> {
  let conn = state.store().connect().await.map_err(|e| e.to_string())?;
  let new = task::New {
    description: form.description.unwrap_or_default(),
    priority: form.priority,
    title: form.title,
    ..Default::default()
  };
  let task = repo::task::create(&conn, state.project_id(), &new)
    .await
    .map_err(|e| e.to_string())?;
  let _ = state.reload_tx().send(());
  Ok(Redirect::to(&format!("/tasks/{}", task.id())))
}

/// Task detail page.
pub async fn task_detail(State(state): State<AppState>, Path(id): Path<String>) -> Result<Html<String>, String> {
  let conn = state.store().connect().await.map_err(|e| e.to_string())?;
  let task_id = repo::resolve::resolve_id(&conn, "tasks", &id)
    .await
    .map_err(|e| e.to_string())?;
  let task = repo::task::find_by_id(&conn, task_id.clone())
    .await
    .map_err(|e| e.to_string())?
    .ok_or_else(|| format!("task not found: {id}"))?;

  let (tags, notes, description_html, is_blocked, blocking, display_links) =
    load_task_detail_data(&conn, &task_id, &task).await?;

  let tmpl = TaskDetailTemplate {
    task,
    tags,
    notes,
    description_html,
    is_blocked,
    blocking,
    display_links,
  };
  Ok(Html(tmpl.render().map_err(|e| e.to_string())?))
}

/// Task detail fragment (for SSE live reload).
pub async fn task_detail_fragment(
  State(state): State<AppState>,
  Path(id): Path<String>,
) -> Result<Html<String>, String> {
  let conn = state.store().connect().await.map_err(|e| e.to_string())?;
  let task_id = repo::resolve::resolve_id(&conn, "tasks", &id)
    .await
    .map_err(|e| e.to_string())?;
  let task = repo::task::find_by_id(&conn, task_id.clone())
    .await
    .map_err(|e| e.to_string())?
    .ok_or_else(|| format!("task not found: {id}"))?;

  let (tags, notes, description_html, is_blocked, blocking, display_links) =
    load_task_detail_data(&conn, &task_id, &task).await?;

  let tmpl = TaskDetailFragmentTemplate {
    task,
    tags,
    notes,
    description_html,
    is_blocked,
    blocking,
    display_links,
  };
  Ok(Html(tmpl.render().map_err(|e| e.to_string())?))
}

/// Task edit form.
pub async fn task_edit_form(State(state): State<AppState>, Path(id): Path<String>) -> Result<Html<String>, String> {
  let conn = state.store().connect().await.map_err(|e| e.to_string())?;
  let task_id = repo::resolve::resolve_id(&conn, "tasks", &id)
    .await
    .map_err(|e| e.to_string())?;
  let task = repo::task::find_by_id(&conn, task_id.clone())
    .await
    .map_err(|e| e.to_string())?
    .ok_or_else(|| format!("task not found: {id}"))?;

  let tags = repo::tag::for_entity(&conn, EntityType::Task, &task_id)
    .await
    .map_err(|e| e.to_string())?;

  let rels = repo::relationship::for_entity(&conn, EntityType::Task, &task_id)
    .await
    .map_err(|e| e.to_string())?;
  let existing_links = forms::build_existing_links_for_entity(&task_id, EntityType::Task, &rels);

  let tmpl = TaskEditTemplate {
    title: task.title().to_owned(),
    description: task.description().to_owned(),
    priority: task.priority().map(|p| p.to_string()).unwrap_or_default(),
    tags: tags.join(", "),
    task,
    error: None,
    existing_links,
  };
  Ok(Html(tmpl.render().map_err(|e| e.to_string())?))
}

/// Task list page.
pub async fn task_list(
  State(state): State<AppState>,
  Query(params): Query<TaskListParams>,
) -> Result<Html<String>, String> {
  let conn = state.store().connect().await.map_err(|e| e.to_string())?;
  let tasks = repo::task::all(&conn, state.project_id(), &Default::default())
    .await
    .map_err(|e| e.to_string())?;

  let mut rows = build_task_rows(&conn, tasks).await?;
  let (open_count, in_progress_count, done_count, cancelled_count) = count_statuses(&rows);
  let current_status = params.status.clone().unwrap_or_default();

  if let Some(ref status) = params.status
    && !status.is_empty()
  {
    rows.retain(|r| r.task.status().to_string() == *status);
  }

  let tmpl = TaskListTemplate {
    rows,
    open_count,
    in_progress_count,
    done_count,
    cancelled_count,
    current_status,
  };
  Ok(Html(tmpl.render().map_err(|e| e.to_string())?))
}

/// Task list fragment (for SSE live reload).
pub async fn task_list_fragment(
  State(state): State<AppState>,
  Query(params): Query<TaskListParams>,
) -> Result<Html<String>, String> {
  let conn = state.store().connect().await.map_err(|e| e.to_string())?;
  let tasks = repo::task::all(&conn, state.project_id(), &Default::default())
    .await
    .map_err(|e| e.to_string())?;

  let mut rows = build_task_rows(&conn, tasks).await?;
  let (open_count, in_progress_count, done_count, cancelled_count) = count_statuses(&rows);
  let current_status = params.status.clone().unwrap_or_default();

  if let Some(ref status) = params.status
    && !status.is_empty()
  {
    rows.retain(|r| r.task.status().to_string() == *status);
  }

  let tmpl = TaskListFragmentTemplate {
    rows,
    open_count,
    in_progress_count,
    done_count,
    cancelled_count,
    current_status,
  };
  Ok(Html(tmpl.render().map_err(|e| e.to_string())?))
}

/// Handle task update from edit form.
pub async fn task_update(
  State(state): State<AppState>,
  Path(id): Path<String>,
  body: Bytes,
) -> Result<Redirect, String> {
  let conn = state.store().connect().await.map_err(|e| e.to_string())?;
  let task_id = repo::resolve::resolve_id(&conn, "tasks", &id)
    .await
    .map_err(|e| e.to_string())?;

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
    Some(status_str.parse().map_err(|e: String| e)?)
  };

  let priority: Option<Option<u8>> = if priority_str.is_empty() {
    Some(None)
  } else {
    let val: u8 = priority_str.parse().map_err(|_| "invalid priority".to_owned())?;
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
    .map_err(|e| e.to_string())?;

  // Update tags: detach all then re-attach
  repo::tag::detach_all(&conn, EntityType::Task, &task_id)
    .await
    .map_err(|e| e.to_string())?;

  for tag in tags_str.split(',').map(|t| t.trim()).filter(|t| !t.is_empty()) {
    repo::tag::attach(&conn, EntityType::Task, &task_id, tag)
      .await
      .map_err(|e| e.to_string())?;
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

/// Build enriched task rows from a list of tasks.
async fn build_task_rows(conn: &Connection, tasks: Vec<task::Model>) -> Result<Vec<TaskRow>, String> {
  let mut rows = Vec::with_capacity(tasks.len());
  for task in tasks {
    let task_id = task.id().clone();
    let tags = repo::tag::for_entity(conn, EntityType::Task, &task_id)
      .await
      .map_err(|e| e.to_string())?;
    let rels = repo::relationship::for_entity(conn, EntityType::Task, &task_id)
      .await
      .map_err(|e| e.to_string())?;

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

/// Count tasks by status from a slice of task rows.
fn count_statuses(rows: &[TaskRow]) -> (usize, usize, usize, usize) {
  let mut open = 0;
  let mut in_progress = 0;
  let mut done = 0;
  let mut cancelled = 0;
  for row in rows {
    match row.task.status() {
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
) -> Result<(Vec<String>, Vec<NoteDisplay>, String, bool, bool, Vec<DisplayLink>), String> {
  let tags = repo::tag::for_entity(conn, EntityType::Task, task_id)
    .await
    .map_err(|e| e.to_string())?;
  let raw_notes = repo::note::for_entity(conn, EntityType::Task, task_id)
    .await
    .map_err(|e| e.to_string())?;
  let rels = repo::relationship::for_entity(conn, EntityType::Task, task_id)
    .await
    .map_err(|e| e.to_string())?;

  let description_html = if task.description().is_empty() {
    String::new()
  } else {
    markdown::render_markdown_to_html(task.description())
  };

  let notes = note_display::build_note_displays(conn, raw_notes).await;

  let (is_blocked, blocking, _) = compute_blocking(task_id, &rels);
  let display_links = build_display_links(task_id, &rels);

  Ok((tags, notes, description_html, is_blocked, blocking, display_links))
}
