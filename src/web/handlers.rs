use askama::Template;
use axum::{
  body::Bytes,
  extract::{Form, Path, Query, State},
  http::StatusCode,
  response::{Html, IntoResponse, Redirect, Response},
};
use libsql::Connection;
use serde::{Deserialize, Serialize};

use super::AppState;
use crate::store::{
  model::{
    artifact, iteration, note,
    primitives::{EntityType, Id, IterationStatus, RelationshipType, TaskStatus},
    relationship, task,
  },
  repo,
  repo::iteration::{IterationTaskRow, StatusCounts},
};

// ── Helper types ──────────────────────────────────────────────────────────────────────────────────

/// Enriched row for the task list view.
struct TaskRow {
  task: task::Model,
  tags: Vec<String>,
  is_blocked: bool,
  blocking: bool,
  blocked_by_display: String,
}

/// A display-friendly representation of a relationship link.
struct DisplayLink {
  rel: String,
  display_text: String,
  href: Option<String>,
}

/// A pre-populated relationship for edit forms.
struct ExistingLink {
  rel: String,
  ref_: String,
}

/// JSON result returned from `/api/search`.
#[derive(Serialize)]
struct ApiSearchResult {
  id: String,
  #[serde(rename = "type")]
  kind: String,
  short_id: String,
  title: String,
}

// ── Templates ───────────────────────────────────────────────────────────────────────────────────

#[derive(Template)]
#[template(path = "artifacts/detail.html")]
struct ArtifactDetailTemplate {
  artifact: artifact::Model,
  body_html: String,
  tags: Vec<String>,
  notes: Vec<NoteDisplay>,
}

#[derive(Template)]
#[template(path = "artifacts/detail_content.html")]
struct ArtifactDetailContentTemplate {
  artifact: artifact::Model,
  body_html: String,
  tags: Vec<String>,
  notes: Vec<NoteDisplay>,
}

struct NoteDisplay {
  author_gravatar: Option<String>,
  author_is_agent: bool,
  author_name: Option<String>,
  body_html: String,
  created_at: String,
  id_short: String,
}

/// Build a Gravatar URL from an email address (or `None`).
fn gravatar_url(email: Option<&str>) -> Option<String> {
  let email = email?.trim().to_lowercase();
  let hash = format!("{:x}", md5::compute(email.as_bytes()));
  Some(format!("https://www.gravatar.com/avatar/{hash}?s=64&d=retro"))
}

/// Convert raw note models into display structs, resolving authors.
async fn build_note_displays(conn: &Connection, notes: Vec<note::Model>) -> Vec<NoteDisplay> {
  let mut displays = Vec::with_capacity(notes.len());
  for n in notes {
    let (author_name, author_gravatar, author_is_agent) = match n.author_id() {
      Some(aid) => match repo::author::find_by_id(conn, aid.clone()).await {
        Ok(Some(author)) => (
          Some(author.name().to_string()),
          gravatar_url(author.email()),
          author.author_type() == crate::store::model::primitives::AuthorType::Agent,
        ),
        _ => (None, None, false),
      },
      None => (None, None, false),
    };
    displays.push(NoteDisplay {
      author_gravatar,
      author_is_agent,
      author_name,
      body_html: super::markdown::render_markdown_to_html(n.body()),
      created_at: n.created_at().format("%Y-%m-%d %H:%M UTC").to_string(),
      id_short: n.id().short(),
    });
  }
  displays
}

#[derive(Template)]
#[template(path = "artifacts/list.html")]
struct ArtifactListTemplate {
  artifacts: Vec<ArtifactRow>,
  open_count: usize,
  archived_count: usize,
  current_status: String,
}

#[derive(Template)]
#[template(path = "artifacts/list_content.html")]
struct ArtifactListContentTemplate {
  artifacts: Vec<ArtifactRow>,
  open_count: usize,
  archived_count: usize,
  current_status: String,
}

struct ArtifactRow {
  artifact: artifact::Model,
  tags: Vec<String>,
}

#[derive(Template)]
#[template(path = "artifacts/create.html")]
struct ArtifactCreateTemplate {
  title: String,
  body: String,
  tags: String,
  error: Option<String>,
}

#[derive(Template)]
#[template(path = "artifacts/edit.html")]
struct ArtifactEditTemplate {
  artifact: artifact::Model,
  title: String,
  body: String,
  tags: String,
  error: Option<String>,
  existing_links: Vec<ExistingLink>,
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
#[template(path = "not_found.html")]
struct NotFoundTemplate;

struct IterationRow {
  iteration: iteration::Model,
  tags: Vec<String>,
  task_count: i64,
  phase_count: u32,
}

struct PhaseGroup {
  number: u32,
  tasks: Vec<IterationTaskRow>,
}

#[derive(Template)]
#[template(path = "iterations/board.html")]
struct IterationBoardTemplate {
  iteration: iteration::Model,
  open_tasks: Vec<IterationTaskRow>,
  in_progress_tasks: Vec<IterationTaskRow>,
  done_tasks: Vec<IterationTaskRow>,
  cancelled_tasks: Vec<IterationTaskRow>,
}

#[derive(Template)]
#[template(path = "iterations/board_content.html")]
struct IterationBoardFragmentTemplate {
  iteration: iteration::Model,
  open_tasks: Vec<IterationTaskRow>,
  in_progress_tasks: Vec<IterationTaskRow>,
  done_tasks: Vec<IterationTaskRow>,
  cancelled_tasks: Vec<IterationTaskRow>,
}

#[derive(Template)]
#[template(path = "iterations/detail.html")]
struct IterationDetailTemplate {
  iteration: iteration::Model,
  tags: Vec<String>,
  phases: Vec<PhaseGroup>,
  task_count: i64,
  status_counts: StatusCounts,
}

#[derive(Template)]
#[template(path = "iterations/detail_content.html")]
struct IterationDetailFragmentTemplate {
  iteration: iteration::Model,
  tags: Vec<String>,
  phases: Vec<PhaseGroup>,
  task_count: i64,
  status_counts: StatusCounts,
}

#[derive(Template)]
#[template(path = "iterations/list.html")]
struct IterationListTemplate {
  rows: Vec<IterationRow>,
  active_count: usize,
  completed_count: usize,
  cancelled_count: usize,
  current_status: String,
}

#[derive(Template)]
#[template(path = "iterations/list_content.html")]
struct IterationListFragmentTemplate {
  rows: Vec<IterationRow>,
  active_count: usize,
  completed_count: usize,
  cancelled_count: usize,
  current_status: String,
}

#[derive(Deserialize)]
pub struct IterationListParams {
  status: Option<String>,
}

#[derive(Template)]
#[template(path = "tasks/detail.html")]
struct TaskDetailTemplate {
  task: task::Model,
  tags: Vec<String>,
  notes: Vec<NoteDisplay>,
  description_html: String,
  is_blocked: bool,
  blocking: bool,
  display_links: Vec<DisplayLink>,
}

#[derive(Template)]
#[template(path = "tasks/detail_content.html")]
struct TaskDetailFragmentTemplate {
  task: task::Model,
  tags: Vec<String>,
  notes: Vec<NoteDisplay>,
  description_html: String,
  is_blocked: bool,
  blocking: bool,
  display_links: Vec<DisplayLink>,
}

#[derive(Template)]
#[template(path = "tasks/edit.html")]
struct TaskEditTemplate {
  task: task::Model,
  title: String,
  description: String,
  priority: String,
  tags: String,
  error: Option<String>,
  existing_links: Vec<ExistingLink>,
}

#[derive(Template)]
#[template(path = "search.html")]
struct SearchTemplate {
  artifacts: Vec<artifact::Model>,
  iterations: Vec<iteration::Model>,
  query: String,
  tasks: Vec<task::Model>,
}

#[derive(Template)]
#[template(path = "tasks/create.html")]
struct TaskCreateTemplate;

#[derive(Template)]
#[template(path = "tasks/list.html")]
struct TaskListTemplate {
  rows: Vec<TaskRow>,
  open_count: usize,
  in_progress_count: usize,
  done_count: usize,
  cancelled_count: usize,
  current_status: String,
}

#[derive(Template)]
#[template(path = "tasks/list_content.html")]
struct TaskListFragmentTemplate {
  rows: Vec<TaskRow>,
  open_count: usize,
  in_progress_count: usize,
  done_count: usize,
  cancelled_count: usize,
  current_status: String,
}

// ── Query / Form types ──────────────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct ArtifactListParams {
  status: Option<String>,
}

#[derive(Deserialize)]
pub struct ArtifactForm {
  title: String,
  body: Option<String>,
  tags: Option<String>,
}

#[derive(Deserialize)]
pub struct NoteFormData {
  body: String,
}

#[derive(Deserialize)]
pub struct ApiSearchParams {
  q: Option<String>,
}

#[derive(Deserialize)]
pub struct SearchQuery {
  q: Option<String>,
}

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

// ── Shared helpers ────────────────────────────────────────────────────────────────────────────────

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
    super::markdown::render_markdown_to_html(task.description())
  };

  let notes = build_note_displays(conn, raw_notes).await;

  let (is_blocked, blocking, _) = compute_blocking(task_id, &rels);
  let display_links = build_display_links(task_id, &rels);

  Ok((tags, notes, description_html, is_blocked, blocking, display_links))
}

// ── Helpers ─────────────────────────────────────────────────────────────────────────────────────

/// Build the enriched artifact list data (rows with tags, counts, filtered).
async fn build_artifact_list_data(
  state: &AppState,
  status: Option<String>,
) -> Result<(Vec<ArtifactRow>, usize, usize, String), String> {
  let conn = state.store().connect().await.map_err(|e| e.to_string())?;

  // Fetch all artifacts to compute counts
  let all_artifacts = repo::artifact::all(
    &conn,
    state.project_id(),
    &artifact::Filter {
      all: true,
      ..Default::default()
    },
  )
  .await
  .map_err(|e| e.to_string())?;

  let open_count = all_artifacts.iter().filter(|a| !a.is_archived()).count();
  let archived_count = all_artifacts.iter().filter(|a| a.is_archived()).count();

  let current_status = status.unwrap_or_else(|| "open".to_owned());

  // Build filter based on status param
  let filter = match current_status.as_str() {
    "all" => artifact::Filter {
      all: true,
      ..Default::default()
    },
    "archived" => artifact::Filter {
      only_archived: true,
      ..Default::default()
    },
    _ => artifact::Filter::default(), // default shows open only
  };

  let artifacts = repo::artifact::all(&conn, state.project_id(), &filter)
    .await
    .map_err(|e| e.to_string())?;

  let mut rows = Vec::with_capacity(artifacts.len());
  for a in artifacts {
    let tags = repo::tag::for_entity(&conn, EntityType::Artifact, a.id())
      .await
      .map_err(|e| e.to_string())?;
    rows.push(ArtifactRow {
      artifact: a,
      tags,
    });
  }

  Ok((rows, open_count, archived_count, current_status))
}

/// Build enriched artifact detail data.
async fn build_artifact_detail_data(
  state: &AppState,
  id: &str,
) -> Result<(artifact::Model, String, Vec<String>, Vec<NoteDisplay>), String> {
  let conn = state.store().connect().await.map_err(|e| e.to_string())?;
  let artifact_id = repo::resolve::resolve_id(&conn, "artifacts", id)
    .await
    .map_err(|e| e.to_string())?;
  let artifact = repo::artifact::find_by_id(&conn, artifact_id.clone())
    .await
    .map_err(|e| e.to_string())?
    .ok_or_else(|| format!("artifact not found: {id}"))?;

  let tags = repo::tag::for_entity(&conn, EntityType::Artifact, &artifact_id)
    .await
    .map_err(|e| e.to_string())?;
  let raw_notes = repo::note::for_entity(&conn, EntityType::Artifact, &artifact_id)
    .await
    .map_err(|e| e.to_string())?;

  let body_html = super::markdown::render_markdown_to_html(artifact.body());
  let notes = build_note_displays(&conn, raw_notes).await;

  Ok((artifact, body_html, tags, notes))
}

/// Build existing link items for edit form pre-population.
fn build_existing_links_for_entity(
  entity_id: &Id,
  entity_type: EntityType,
  rels: &[relationship::Model],
) -> Vec<ExistingLink> {
  let mut links = Vec::new();
  for rel in rels {
    let (rel_label, other_id, other_type) = if rel.source_id() == entity_id && rel.source_type() == entity_type {
      (rel.rel_type().to_string(), rel.target_id().clone(), rel.target_type())
    } else {
      (
        rel.rel_type().inverse().to_string(),
        rel.source_id().clone(),
        rel.source_type(),
      )
    };

    let type_prefix = match other_type {
      EntityType::Task => "tasks",
      EntityType::Artifact => "artifacts",
      EntityType::Iteration => "iterations",
    };
    let ref_ = format!("{type_prefix}/{other_id}");
    links.push(ExistingLink {
      rel: rel_label,
      ref_,
    });
  }
  links
}

/// Parse parallel `link_rel[]` and `link_ref[]` form fields and create relationships.
async fn sync_form_links(
  conn: &Connection,
  entity_type: EntityType,
  entity_id: &Id,
  link_rels: &[String],
  link_refs: &[String],
) -> Result<(), String> {
  // Delete all existing relationships where this entity is the source
  let existing = repo::relationship::for_entity(conn, entity_type, entity_id)
    .await
    .map_err(|e| e.to_string())?;
  for rel in &existing {
    if rel.source_id() == entity_id && rel.source_type() == entity_type {
      repo::relationship::delete(conn, rel.id())
        .await
        .map_err(|e| e.to_string())?;
    }
  }

  // Create new relationships from form data
  for (rel_str, ref_str) in link_rels.iter().zip(link_refs.iter()) {
    let rel_type: RelationshipType = match rel_str.parse() {
      Ok(r) => r,
      Err(_) => continue,
    };

    // ref format: "tasks/{id}" or "artifacts/{id}"
    let parts: Vec<&str> = ref_str.splitn(2, '/').collect();
    if parts.len() != 2 {
      continue;
    }
    let (target_type, target_id_str) = match parts[0] {
      "tasks" => (EntityType::Task, parts[1]),
      "artifacts" => (EntityType::Artifact, parts[1]),
      "iterations" => (EntityType::Iteration, parts[1]),
      _ => continue,
    };
    let target_id: Id = match target_id_str.parse() {
      Ok(id) => id,
      Err(_) => continue,
    };

    repo::relationship::create(conn, rel_type, entity_type, entity_id, target_type, &target_id)
      .await
      .map_err(|e| e.to_string())?;
  }

  Ok(())
}

/// Extract repeated `link_rel[]` and `link_ref[]` fields from url-encoded form body.
fn extract_link_fields(bytes: &[u8]) -> (Vec<String>, Vec<String>) {
  let mut rels = Vec::new();
  let mut refs = Vec::new();
  for (key, value) in form_urlencoded::parse(bytes) {
    match key.as_ref() {
      "link_rel[]" => rels.push(value.into_owned()),
      "link_ref[]" => refs.push(value.into_owned()),
      _ => {}
    }
  }
  (rels, refs)
}

/// Parse comma-separated tags string into trimmed, non-empty tag labels.
fn parse_tags(tags_str: &str) -> Vec<String> {
  tags_str
    .split(',')
    .map(|t| t.trim().to_owned())
    .filter(|t| !t.is_empty())
    .collect()
}

// ── Handlers ────────────────────────────────────────────────────────────────────────────────────

/// Artifact detail page.
pub async fn artifact_detail(State(state): State<AppState>, Path(id): Path<String>) -> Result<Html<String>, String> {
  let (artifact, body_html, tags, notes) = build_artifact_detail_data(&state, &id).await?;
  let tmpl = ArtifactDetailTemplate {
    artifact,
    body_html,
    tags,
    notes,
  };
  Ok(Html(tmpl.render().map_err(|e| e.to_string())?))
}

/// Artifact detail fragment (SSE live reload).
pub async fn artifact_detail_fragment(
  State(state): State<AppState>,
  Path(id): Path<String>,
) -> Result<Html<String>, String> {
  let (artifact, body_html, tags, notes) = build_artifact_detail_data(&state, &id).await?;
  let tmpl = ArtifactDetailContentTemplate {
    artifact,
    body_html,
    tags,
    notes,
  };
  Ok(Html(tmpl.render().map_err(|e| e.to_string())?))
}

/// Artifact list page.
pub async fn artifact_list(
  State(state): State<AppState>,
  Query(params): Query<ArtifactListParams>,
) -> Result<Html<String>, String> {
  let (artifacts, open_count, archived_count, current_status) = build_artifact_list_data(&state, params.status).await?;
  let tmpl = ArtifactListTemplate {
    artifacts,
    open_count,
    archived_count,
    current_status,
  };
  Ok(Html(tmpl.render().map_err(|e| e.to_string())?))
}

/// Artifact list fragment (SSE live reload).
pub async fn artifact_list_fragment(
  State(state): State<AppState>,
  Query(params): Query<ArtifactListParams>,
) -> Result<Html<String>, String> {
  let (artifacts, open_count, archived_count, current_status) = build_artifact_list_data(&state, params.status).await?;
  let tmpl = ArtifactListContentTemplate {
    artifacts,
    open_count,
    archived_count,
    current_status,
  };
  Ok(Html(tmpl.render().map_err(|e| e.to_string())?))
}

/// Artifact create form.
pub async fn artifact_create_form() -> Result<Html<String>, String> {
  let tmpl = ArtifactCreateTemplate {
    title: String::new(),
    body: String::new(),
    tags: String::new(),
    error: None,
  };
  Ok(Html(tmpl.render().map_err(|e| e.to_string())?))
}

/// Handle artifact creation from form.
pub async fn artifact_create_submit(
  State(state): State<AppState>,
  Form(form): Form<ArtifactForm>,
) -> Result<Redirect, String> {
  let conn = state.store().connect().await.map_err(|e| e.to_string())?;

  let new = artifact::New {
    title: form.title,
    body: form.body.unwrap_or_default(),
    ..Default::default()
  };
  let artifact = repo::artifact::create(&conn, state.project_id(), &new)
    .await
    .map_err(|e| e.to_string())?;

  // Attach tags
  if let Some(tags_str) = &form.tags {
    for label in parse_tags(tags_str) {
      repo::tag::attach(&conn, EntityType::Artifact, artifact.id(), &label)
        .await
        .map_err(|e| e.to_string())?;
    }
  }

  let _ = state.reload_tx().send(());
  Ok(Redirect::to(&format!("/artifacts/{}", artifact.id())))
}

/// Artifact edit form.
pub async fn artifact_edit_form(State(state): State<AppState>, Path(id): Path<String>) -> Result<Html<String>, String> {
  let conn = state.store().connect().await.map_err(|e| e.to_string())?;
  let artifact_id = repo::resolve::resolve_id(&conn, "artifacts", &id)
    .await
    .map_err(|e| e.to_string())?;
  let artifact = repo::artifact::find_by_id(&conn, artifact_id.clone())
    .await
    .map_err(|e| e.to_string())?
    .ok_or_else(|| format!("artifact not found: {id}"))?;

  let tags = repo::tag::for_entity(&conn, EntityType::Artifact, &artifact_id)
    .await
    .map_err(|e| e.to_string())?;

  let rels = repo::relationship::for_entity(&conn, EntityType::Artifact, &artifact_id)
    .await
    .map_err(|e| e.to_string())?;
  let existing_links = build_existing_links_for_entity(&artifact_id, EntityType::Artifact, &rels);

  let tmpl = ArtifactEditTemplate {
    title: artifact.title().to_owned(),
    body: artifact.body().to_owned(),
    tags: tags.join(", "),
    artifact,
    error: None,
    existing_links,
  };
  Ok(Html(tmpl.render().map_err(|e| e.to_string())?))
}

/// Handle artifact update from form.
pub async fn artifact_update(
  State(state): State<AppState>,
  Path(id): Path<String>,
  body: Bytes,
) -> Result<Redirect, String> {
  let conn = state.store().connect().await.map_err(|e| e.to_string())?;
  let artifact_id = repo::resolve::resolve_id(&conn, "artifacts", &id)
    .await
    .map_err(|e| e.to_string())?;

  // Parse form fields from raw body
  let mut title = String::new();
  let mut body_field = String::new();
  let mut tags_str = String::new();
  let (link_rels, link_refs) = extract_link_fields(&body);
  for (key, value) in form_urlencoded::parse(&body) {
    match key.as_ref() {
      "title" => title = value.into_owned(),
      "body" => body_field = value.into_owned(),
      "tags" => tags_str = value.into_owned(),
      _ => {}
    }
  }

  let patch = artifact::Patch {
    title: Some(title),
    body: Some(body_field),
    ..Default::default()
  };
  repo::artifact::update(&conn, &artifact_id, &patch)
    .await
    .map_err(|e| e.to_string())?;

  // Re-sync tags: detach all, then re-attach
  repo::tag::detach_all(&conn, EntityType::Artifact, &artifact_id)
    .await
    .map_err(|e| e.to_string())?;
  for label in parse_tags(&tags_str) {
    repo::tag::attach(&conn, EntityType::Artifact, &artifact_id, &label)
      .await
      .map_err(|e| e.to_string())?;
  }

  // Sync relationships
  sync_form_links(&conn, EntityType::Artifact, &artifact_id, &link_rels, &link_refs).await?;

  let _ = state.reload_tx().send(());
  Ok(Redirect::to(&format!("/artifacts/{artifact_id}")))
}

/// Archive an artifact.
pub async fn artifact_archive(State(state): State<AppState>, Path(id): Path<String>) -> Result<Redirect, String> {
  let conn = state.store().connect().await.map_err(|e| e.to_string())?;
  let artifact_id = repo::resolve::resolve_id(&conn, "artifacts", &id)
    .await
    .map_err(|e| e.to_string())?;
  repo::artifact::archive(&conn, &artifact_id)
    .await
    .map_err(|e| e.to_string())?;

  let _ = state.reload_tx().send(());
  Ok(Redirect::to("/artifacts"))
}

/// Add a note to an artifact.
pub async fn artifact_note_add(
  State(state): State<AppState>,
  Path(id): Path<String>,
  Form(form): Form<NoteFormData>,
) -> Result<Redirect, String> {
  let conn = state.store().connect().await.map_err(|e| e.to_string())?;
  let artifact_id = repo::resolve::resolve_id(&conn, "artifacts", &id)
    .await
    .map_err(|e| e.to_string())?;

  let new = note::New {
    body: form.body,
    author_id: state.author_id().cloned(),
  };
  repo::note::create(&conn, EntityType::Artifact, &artifact_id, &new)
    .await
    .map_err(|e| e.to_string())?;

  let _ = state.reload_tx().send(());
  Ok(Redirect::to(&format!("/artifacts/{artifact_id}")))
}

/// Compute dashboard status counts from task and iteration lists.
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
  let mut phase_map: std::collections::BTreeMap<u32, Vec<IterationTaskRow>> = std::collections::BTreeMap::new();
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

/// Search page.
pub async fn search(State(state): State<AppState>, Query(params): Query<SearchQuery>) -> Result<Html<String>, String> {
  let conn = state.store().connect().await.map_err(|e| e.to_string())?;
  let query = params.q.unwrap_or_default();

  let (tasks, artifacts, iterations) = if query.is_empty() {
    (Vec::new(), Vec::new(), Vec::new())
  } else {
    let parsed = crate::store::search_query::parse(&query);
    let results = repo::search::query(&conn, state.project_id(), &parsed, false)
      .await
      .map_err(|e| e.to_string())?;
    (results.tasks, results.artifacts, results.iterations)
  };

  let tmpl = SearchTemplate {
    artifacts,
    iterations,
    query,
    tasks,
  };
  Ok(Html(tmpl.render().map_err(|e| e.to_string())?))
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
  let existing_links = build_existing_links_for_entity(&task_id, EntityType::Task, &rels);

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
  let (link_rels, link_refs) = extract_link_fields(&body);
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
  sync_form_links(&conn, EntityType::Task, &task_id, &link_rels, &link_refs).await?;

  let _ = state.reload_tx().send(());
  Ok(Redirect::to(&format!("/tasks/{}", task_id)))
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

/// Fallback handler for unmatched routes.
pub async fn not_found() -> Response {
  let body = NotFoundTemplate
    .render()
    .unwrap_or_else(|_| "404 — not found".to_owned());
  (StatusCode::NOT_FOUND, Html(body)).into_response()
}

// ── API ──────────────────────────────────────────────────────────────────────────────────────────

/// Request body for the render-markdown endpoint.
#[derive(Deserialize)]
pub struct RenderMarkdownBody {
  pub body: String,
}

/// POST /api/render-markdown — render Markdown to HTML.
pub async fn api_render_markdown(axum::Json(payload): axum::Json<RenderMarkdownBody>) -> Response {
  let html_output = super::markdown::render_markdown_to_html(&payload.body);
  (
    [(axum::http::header::CONTENT_TYPE, "text/html; charset=utf-8")],
    html_output,
  )
    .into_response()
}

/// GET /api/search?q=... — JSON search results for the relationship picker.
pub async fn api_search(State(state): State<AppState>, Query(params): Query<ApiSearchParams>) -> Response {
  let query = params.q.unwrap_or_default();
  if query.is_empty() {
    return axum::Json(Vec::<ApiSearchResult>::new()).into_response();
  }

  let conn = match state.store().connect().await {
    Ok(c) => c,
    Err(e) => {
      log::error!("api search connect failed: {e}");
      return (
        StatusCode::INTERNAL_SERVER_ERROR,
        axum::Json(Vec::<ApiSearchResult>::new()),
      )
        .into_response();
    }
  };

  let parsed = crate::store::search_query::parse(&query);
  let results = match repo::search::query(&conn, state.project_id(), &parsed, true).await {
    Ok(r) => r,
    Err(e) => {
      log::error!("api search failed: {e}");
      return (
        StatusCode::INTERNAL_SERVER_ERROR,
        axum::Json(Vec::<ApiSearchResult>::new()),
      )
        .into_response();
    }
  };

  let mut items: Vec<ApiSearchResult> = Vec::new();
  for task in results.tasks {
    items.push(ApiSearchResult {
      id: task.id().to_string(),
      kind: "task".to_string(),
      short_id: task.id().short(),
      title: task.title().to_owned(),
    });
  }
  for artifact in results.artifacts {
    items.push(ApiSearchResult {
      id: artifact.id().to_string(),
      kind: "artifact".to_string(),
      short_id: artifact.id().short(),
      title: artifact.title().to_owned(),
    });
  }
  for iteration in results.iterations {
    items.push(ApiSearchResult {
      id: iteration.id().to_string(),
      kind: "iteration".to_string(),
      short_id: iteration.id().short(),
      title: iteration.title().to_owned(),
    });
  }

  axum::Json(items).into_response()
}
