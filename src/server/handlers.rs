//! Request handlers for each web view.

use std::collections::BTreeMap;

use axum::{
  Json,
  body::Bytes,
  extract::{Form, Path, Query, State},
  http::StatusCode,
  response::{Html, IntoResponse, Redirect, Response},
};
use pulldown_cmark::{Options, Parser, html};

use super::{
  state::ServerState,
  templates::{
    ArtifactCreateTemplate, ArtifactDetailTemplate, ArtifactEditTemplate, ArtifactListTemplate, DashboardTemplate,
    DisplayEvent, DisplayLink, DisplayNote, IterationBoardTemplate, IterationDetailTemplate, IterationListTemplate,
    PhaseGroup, SearchTemplate, TaskCreateTemplate, TaskDetailTemplate, TaskEditTemplate, TaskListTemplate, TaskRow,
    TimelineEntry,
  },
};
use crate::{
  model::{
    ArtifactFilter, ArtifactPatch, IterationFilter, NewArtifact, NewTask, TaskFilter, TaskPatch,
    event::EventKind,
    iteration::Status as IterationStatus,
    link::{Link, RelationshipType},
    note::AuthorType,
    task::Status,
  },
  store,
};

// ── Type declarations ────────────────────────────────────────────────────────────────────────────

/// A single search result for the JSON API.
#[derive(serde::Serialize)]
pub struct ApiSearchResult {
  pub id: String,
  #[serde(rename = "type")]
  pub kind: String,
  pub short_id: String,
  pub title: String,
}

/// Form data for creating/updating an artifact.
#[derive(serde::Deserialize)]
pub struct ArtifactFormData {
  #[serde(default)]
  pub body: String,
  #[serde(default)]
  pub kind: String,
  #[serde(default)]
  pub tags: String,
  pub title: String,
}

/// Query parameters for the artifact list endpoint.
#[derive(serde::Deserialize)]
pub struct ArtifactListParams {
  #[serde(default)]
  pub status: Option<String>,
}

/// Query parameters for the iteration list endpoint.
#[derive(serde::Deserialize)]
pub struct IterationListParams {
  #[serde(default)]
  pub status: Option<String>,
}

/// Form data for adding a note to a task.
#[derive(serde::Deserialize)]
pub struct NoteFormData {
  #[serde(default)]
  pub body: String,
}

/// Request body for the render-markdown endpoint.
#[derive(serde::Deserialize)]
pub struct RenderMarkdownBody {
  pub body: String,
}

/// Query parameters for the search endpoint.
#[derive(serde::Deserialize)]
pub struct SearchParams {
  #[serde(default)]
  pub q: String,
}

/// Form data for task creation.
pub struct TaskFormData {
  pub description: String,
  pub link_refs: Vec<String>,
  pub link_rels: Vec<String>,
  pub priority: String,
  pub tags: String,
  pub title: String,
}

impl TaskFormData {
  /// Parse from raw URL-encoded form bytes, correctly handling repeated keys.
  fn from_bytes(bytes: &[u8]) -> Self {
    let mut title = String::new();
    let mut description = String::new();
    let mut tags = String::new();
    let mut priority = String::new();
    let mut link_rels = Vec::new();
    let mut link_refs = Vec::new();

    for (key, value) in form_urlencoded::parse(bytes) {
      match key.as_ref() {
        "title" => title = value.into_owned(),
        "description" => description = value.into_owned(),
        "tags" => tags = value.into_owned(),
        "priority" => priority = value.into_owned(),
        "link_rel[]" => link_rels.push(value.into_owned()),
        "link_ref[]" => link_refs.push(value.into_owned()),
        _ => {}
      }
    }

    Self {
      description,
      link_refs,
      link_rels,
      priority,
      tags,
      title,
    }
  }
}

/// Query parameters for the task list endpoint.
#[derive(serde::Deserialize)]
pub struct TaskListParams {
  #[serde(default)]
  pub status: Option<String>,
}

// ── Free functions ────────────────────────────────────────────────────────────

/// Generate a Gravatar URL from an email address using SHA-256.
fn gravatar_url(email: Option<&str>) -> String {
  use sha2::{Digest, Sha256};
  let email = email.unwrap_or("");
  let trimmed = email.trim().to_lowercase();
  let digest = Sha256::digest(trimmed.as_bytes());
  let hash: String = digest.iter().map(|b| format!("{b:02x}")).collect();
  format!("https://gravatar.com/avatar/{hash}?s=32&d=identicon")
}

/// Parse parallel `link_rel[]` and `link_ref[]` form fields into `Vec<Link>`.
fn parse_form_links(rels: &[String], refs: &[String]) -> Vec<Link> {
  rels
    .iter()
    .zip(refs.iter())
    .filter_map(|(rel, ref_)| {
      let rel: RelationshipType = rel.parse().ok()?;
      Some(Link {
        ref_: ref_.clone(),
        rel,
      })
    })
    .collect()
}

/// Render a Markdown string to HTML using GFM extensions.
fn render_markdown(input: &str) -> String {
  let mut opts = Options::empty();
  opts.insert(Options::ENABLE_TABLES);
  opts.insert(Options::ENABLE_STRIKETHROUGH);
  opts.insert(Options::ENABLE_TASKLISTS);
  let parser = Parser::new_ext(input, opts);
  let mut html_output = String::new();
  html::push_html(&mut html_output, parser);
  html_output
}

// ── Handlers ──────────────────────────────────────────────────────────────────

/// GET / — dashboard with entity counts and navigation.
pub async fn dashboard(State(state): State<ServerState>) -> Response {
  let tasks = store::list_tasks(
    &state.settings,
    &TaskFilter {
      all: true,
      ..Default::default()
    },
  )
  .unwrap_or_default();

  let artifact_count = store::list_artifacts(&state.settings, &ArtifactFilter::default())
    .unwrap_or_default()
    .len();

  let iteration_count = store::list_iterations(&state.settings, &IterationFilter::default())
    .unwrap_or_default()
    .len();

  let open_count = tasks.iter().filter(|t| t.status == Status::Open).count();
  let in_progress_count = tasks.iter().filter(|t| t.status == Status::InProgress).count();
  let done_count = tasks.iter().filter(|t| t.status == Status::Done).count();
  let cancelled_count = tasks.iter().filter(|t| t.status == Status::Cancelled).count();

  DashboardTemplate {
    task_count: tasks.len(),
    artifact_count,
    iteration_count,
    open_count,
    in_progress_count,
    done_count,
    cancelled_count,
  }
  .into_response()
}

/// GET /tasks — task list filtered by status.
pub async fn task_list(State(state): State<ServerState>, Query(params): Query<TaskListParams>) -> Response {
  let all_tasks = match store::list_tasks(
    &state.settings,
    &TaskFilter {
      all: true,
      ..Default::default()
    },
  ) {
    Ok(t) => t,
    Err(e) => {
      log::error!("failed to list tasks: {e}");
      return (StatusCode::INTERNAL_SERVER_ERROR, Html(format!("<p>error: {e}</p>"))).into_response();
    }
  };

  let open_count = all_tasks.iter().filter(|t| t.status == Status::Open).count();
  let in_progress_count = all_tasks.iter().filter(|t| t.status == Status::InProgress).count();
  let done_count = all_tasks.iter().filter(|t| t.status == Status::Done).count();
  let cancelled_count = all_tasks.iter().filter(|t| t.status == Status::Cancelled).count();

  let current_status = match params.status.as_deref() {
    Some("in_progress") => Status::InProgress,
    Some("done") => Status::Done,
    Some("cancelled") => Status::Cancelled,
    _ => Status::Open,
  };

  let tasks: Vec<_> = all_tasks.into_iter().filter(|t| t.status == current_status).collect();

  let blockings = store::resolve_blocking_batch(&state.settings, &tasks);

  let rows: Vec<TaskRow> = tasks
    .iter()
    .zip(blockings)
    .map(|(task, blocking)| {
      let is_blocked = !blocking.blocked_by_ids.is_empty();
      TaskRow {
        task: task.clone(),
        blocking,
        is_blocked,
      }
    })
    .collect();

  TaskListTemplate {
    tasks,
    rows,
    current_status,
    open_count,
    in_progress_count,
    done_count,
    cancelled_count,
  }
  .into_response()
}

/// GET /tasks/:id — task detail.
pub async fn task_detail(State(state): State<ServerState>, Path(id_str): Path<String>) -> Response {
  let id = match store::resolve_task_id(&state.settings, &id_str, true) {
    Ok(id) => id,
    Err(_) => return (StatusCode::NOT_FOUND, Html("<p>404 — task not found</p>")).into_response(),
  };

  let task = match store::read_task(&state.settings, &id) {
    Ok(t) => t,
    Err(e) => {
      log::error!("failed to read task {id}: {e}");
      return (StatusCode::INTERNAL_SERVER_ERROR, Html(format!("<p>error: {e}</p>"))).into_response();
    }
  };

  let blocking = store::resolve_blocking(&state.settings, &task);
  let is_blocked = !blocking.blocked_by_ids.is_empty();
  let description_html = render_markdown(&task.description);

  let display_links: Vec<DisplayLink> = task
    .links
    .iter()
    .map(|link| {
      let ref_ = &link.ref_;
      let internal_prefixes = ["tasks/", "artifacts/", "iterations/"];
      if let Some(prefix) = internal_prefixes.iter().find(|p| ref_.starts_with(**p)) {
        let id_part = &ref_[prefix.len()..];
        let short = if id_part.len() > 8 { &id_part[..8] } else { id_part };
        DisplayLink {
          rel: link.rel.clone(),
          href: Some(format!("/{ref_}")),
          display_text: short.to_owned(),
        }
      } else if ref_.starts_with("http") {
        DisplayLink {
          rel: link.rel.clone(),
          href: Some(ref_.clone()),
          display_text: ref_.clone(),
        }
      } else {
        DisplayLink {
          rel: link.rel.clone(),
          href: None,
          display_text: ref_.clone(),
        }
      }
    })
    .collect();

  let mut timeline: Vec<(chrono::DateTime<chrono::Utc>, TimelineEntry)> = Vec::new();

  for note in &task.notes {
    let author = note.author.clone();
    let avatar_url = gravatar_url(note.author_email.as_deref());
    timeline.push((
      note.created_at,
      TimelineEntry::Note(DisplayNote {
        author,
        avatar_url,
        body_html: render_markdown(&note.body),
        created_at: note.created_at.format("%Y-%m-%d %H:%M UTC").to_string(),
        id_short: note.id.short(),
        is_agent: matches!(note.author_type, AuthorType::Agent),
      }),
    ));
  }

  for event in &task.events {
    let author = event.author.clone();
    let avatar_url = gravatar_url(event.author_email.as_deref());
    let description = match &event.kind {
      EventKind::PhaseChange {
        from,
        to,
      } => {
        let f = from.map_or("none".to_string(), |v| v.to_string());
        let t = to.map_or("none".to_string(), |v| v.to_string());
        format!("phase changed from {f} to {t}")
      }
      EventKind::PriorityChange {
        from,
        to,
      } => {
        let f = from.map_or("none".to_string(), |v| format!("P{v}"));
        let t = to.map_or("none".to_string(), |v| format!("P{v}"));
        format!("priority changed from {f} to {t}")
      }
      EventKind::StatusChange {
        from,
        to,
      } => {
        format!("status changed from {from} to {to}")
      }
    };
    timeline.push((
      event.created_at,
      TimelineEntry::Event(DisplayEvent {
        author,
        avatar_url,
        created_at: event.created_at.format("%Y-%m-%d %H:%M UTC").to_string(),
        description,
        is_agent: matches!(event.author_type, AuthorType::Agent),
      }),
    ));
  }

  timeline.sort_by_key(|(ts, _)| *ts);
  let timeline: Vec<TimelineEntry> = timeline.into_iter().map(|(_, entry)| entry).collect();

  TaskDetailTemplate {
    task,
    blocking,
    is_blocked,
    description_html,
    display_links,
    timeline,
  }
  .into_response()
}

/// GET /tasks/new — task create form.
pub async fn task_create_form() -> Response {
  TaskCreateTemplate {
    title: String::new(),
    description: String::new(),
    tags: String::new(),
    priority: String::new(),
    error: None,
  }
  .into_response()
}

/// POST /tasks — create a task.
pub async fn task_create(State(state): State<ServerState>, body: Bytes) -> Response {
  let form = TaskFormData::from_bytes(&body);
  let title = form.title.trim().to_string();
  if title.is_empty() {
    return TaskCreateTemplate {
      title: form.title,
      description: form.description,
      tags: form.tags,
      priority: form.priority,
      error: Some("Title is required".to_string()),
    }
    .into_response();
  }

  let tags: Vec<String> = form
    .tags
    .split(',')
    .map(|s| s.trim().to_string())
    .filter(|s| !s.is_empty())
    .collect();

  let priority: Option<u8> = if form.priority.trim().is_empty() {
    None
  } else {
    form.priority.trim().parse().ok()
  };

  let links = parse_form_links(&form.link_rels, &form.link_refs);

  let new_task = NewTask {
    title,
    description: form.description,
    tags,
    priority,
    links,
    ..Default::default()
  };

  match store::create_task(&state.settings, new_task) {
    Ok(task) => Redirect::to(&format!("/tasks/{}", task.id)).into_response(),
    Err(e) => {
      log::error!("failed to create task: {e}");
      (StatusCode::INTERNAL_SERVER_ERROR, Html(format!("<p>error: {e}</p>"))).into_response()
    }
  }
}

/// GET /tasks/:id/edit — task edit form.
pub async fn task_edit_form(State(state): State<ServerState>, Path(id_str): Path<String>) -> Response {
  let id = match store::resolve_task_id(&state.settings, &id_str, true) {
    Ok(id) => id,
    Err(_) => return (StatusCode::NOT_FOUND, Html("<p>404 — task not found</p>")).into_response(),
  };

  let task = match store::read_task(&state.settings, &id) {
    Ok(t) => t,
    Err(e) => {
      log::error!("failed to read task {id}: {e}");
      return (StatusCode::INTERNAL_SERVER_ERROR, Html(format!("<p>error: {e}</p>"))).into_response();
    }
  };

  let tags = task.tags.join(", ");
  let priority = task.priority.map(|p| p.to_string()).unwrap_or_default();

  TaskEditTemplate {
    title: task.title.clone(),
    description: task.description.clone(),
    tags,
    priority,
    error: None,
    task,
  }
  .into_response()
}

/// POST /tasks/:id — update a task.
pub async fn task_update(State(state): State<ServerState>, Path(id_str): Path<String>, body: Bytes) -> Response {
  let form = TaskFormData::from_bytes(&body);
  let id = match store::resolve_task_id(&state.settings, &id_str, true) {
    Ok(id) => id,
    Err(_) => return (StatusCode::NOT_FOUND, Html("<p>404 — task not found</p>")).into_response(),
  };

  let title = form.title.trim().to_string();
  if title.is_empty() {
    let task = match store::read_task(&state.settings, &id) {
      Ok(t) => t,
      Err(e) => {
        log::error!("failed to read task {id}: {e}");
        return (StatusCode::INTERNAL_SERVER_ERROR, Html(format!("<p>error: {e}</p>"))).into_response();
      }
    };

    return TaskEditTemplate {
      title: form.title,
      description: form.description,
      tags: form.tags,
      priority: form.priority,
      error: Some("Title is required".to_string()),
      task,
    }
    .into_response();
  }

  let tags: Vec<String> = form
    .tags
    .split(',')
    .map(|s| s.trim().to_string())
    .filter(|s| !s.is_empty())
    .collect();

  let priority: Option<u8> = if form.priority.trim().is_empty() {
    None
  } else {
    form.priority.trim().parse().ok()
  };

  let patch = TaskPatch {
    title: Some(title),
    description: Some(form.description),
    tags: Some(tags),
    priority: Some(priority),
    ..Default::default()
  };

  let links = parse_form_links(&form.link_rels, &form.link_refs);

  match store::update_task(&state.settings, &id, patch, None) {
    Ok(mut task) => {
      task.links = links;
      if let Err(e) = store::write_task(&state.settings, &task) {
        log::error!("failed to write task links {id}: {e}");
      }
      Redirect::to(&format!("/tasks/{}", task.id)).into_response()
    }
    Err(e) => {
      log::error!("failed to update task {id}: {e}");
      (StatusCode::INTERNAL_SERVER_ERROR, Html(format!("<p>error: {e}</p>"))).into_response()
    }
  }
}

/// GET /artifacts — artifact list with status filtering.
pub async fn artifact_list(State(state): State<ServerState>, Query(params): Query<ArtifactListParams>) -> Response {
  let filter = ArtifactFilter {
    all: true,
    ..Default::default()
  };

  let all_artifacts = match store::list_artifacts(&state.settings, &filter) {
    Ok(a) => a,
    Err(e) => {
      log::error!("failed to list artifacts: {e}");
      return (
        StatusCode::INTERNAL_SERVER_ERROR,
        Html("<p>failed to load artifacts</p>"),
      )
        .into_response();
    }
  };

  let open_count = all_artifacts.iter().filter(|a| a.archived_at.is_none()).count();
  let archived_count = all_artifacts.iter().filter(|a| a.archived_at.is_some()).count();
  let current_status = params.status.unwrap_or_else(|| "open".to_string());

  let artifacts: Vec<_> = all_artifacts
    .into_iter()
    .filter(|a| match current_status.as_str() {
      "archived" => a.archived_at.is_some(),
      _ => a.archived_at.is_none(),
    })
    .collect();

  ArtifactListTemplate {
    artifacts,
    open_count,
    archived_count,
    current_status,
  }
  .into_response()
}

/// GET /artifacts/:id — artifact detail with rendered Markdown.
pub async fn artifact_detail(State(state): State<ServerState>, Path(id): Path<String>) -> Response {
  let resolved = match store::resolve_artifact_id(&state.settings, &id, true) {
    Ok(id) => id,
    Err(_) => return (StatusCode::NOT_FOUND, Html("<p>artifact not found</p>")).into_response(),
  };

  match store::read_artifact(&state.settings, &resolved) {
    Ok(artifact) => {
      let body_html = render_markdown(&artifact.body);
      ArtifactDetailTemplate {
        artifact,
        body_html,
      }
      .into_response()
    }
    Err(e) => {
      log::error!("failed to read artifact {resolved}: {e}");
      (
        StatusCode::INTERNAL_SERVER_ERROR,
        Html("<p>failed to load artifact</p>"),
      )
        .into_response()
    }
  }
}

/// GET /artifacts/new — render the create artifact form.
pub async fn artifact_create_form() -> Response {
  ArtifactCreateTemplate {
    title: String::new(),
    kind: String::new(),
    tags: String::new(),
    body: String::new(),
    error: None,
  }
  .into_response()
}

/// POST /artifacts — create a new artifact.
pub async fn artifact_create(State(state): State<ServerState>, Form(form): Form<ArtifactFormData>) -> Response {
  let title = form.title.trim().to_string();
  if title.is_empty() {
    return ArtifactCreateTemplate {
      title: form.title,
      kind: form.kind,
      tags: form.tags,
      body: form.body,
      error: Some("Title is required.".to_string()),
    }
    .into_response();
  }

  let kind = {
    let k = form.kind.trim().to_string();
    if k.is_empty() { None } else { Some(k) }
  };

  let tags: Vec<String> = form
    .tags
    .split(',')
    .map(|t| t.trim().to_string())
    .filter(|t| !t.is_empty())
    .collect();

  let new = NewArtifact {
    body: form.body,
    kind,
    metadata: yaml_serde::Mapping::new(),
    tags,
    title,
  };

  match store::create_artifact(&state.settings, new) {
    Ok(artifact) => Redirect::to(&format!("/artifacts/{}", artifact.id)).into_response(),
    Err(e) => {
      log::error!("failed to create artifact: {e}");
      (StatusCode::INTERNAL_SERVER_ERROR, Html(format!("<p>error: {e}</p>"))).into_response()
    }
  }
}

/// GET /artifacts/:id/edit — render the edit artifact form.
pub async fn artifact_edit_form(State(state): State<ServerState>, Path(id_str): Path<String>) -> Response {
  let resolved = match store::resolve_artifact_id(&state.settings, &id_str, true) {
    Ok(id) => id,
    Err(_) => return (StatusCode::NOT_FOUND, Html("<p>artifact not found</p>")).into_response(),
  };

  let artifact = match store::read_artifact(&state.settings, &resolved) {
    Ok(a) => a,
    Err(e) => {
      log::error!("failed to read artifact {resolved}: {e}");
      return (StatusCode::INTERNAL_SERVER_ERROR, Html(format!("<p>error: {e}</p>"))).into_response();
    }
  };

  let tags = artifact.tags.join(", ");
  let kind = artifact.kind.clone().unwrap_or_default();
  ArtifactEditTemplate {
    title: artifact.title.clone(),
    kind,
    tags,
    body: artifact.body.clone(),
    error: None,
    artifact,
  }
  .into_response()
}

/// POST /artifacts/:id — update an existing artifact.
pub async fn artifact_update(
  State(state): State<ServerState>,
  Path(id_str): Path<String>,
  Form(form): Form<ArtifactFormData>,
) -> Response {
  let resolved = match store::resolve_artifact_id(&state.settings, &id_str, true) {
    Ok(id) => id,
    Err(_) => return (StatusCode::NOT_FOUND, Html("<p>artifact not found</p>")).into_response(),
  };

  let title = form.title.trim().to_string();
  if title.is_empty() {
    let artifact = match store::read_artifact(&state.settings, &resolved) {
      Ok(a) => a,
      Err(e) => {
        log::error!("failed to read artifact {resolved}: {e}");
        return (StatusCode::INTERNAL_SERVER_ERROR, Html(format!("<p>error: {e}</p>"))).into_response();
      }
    };

    return ArtifactEditTemplate {
      title: form.title,
      kind: form.kind,
      tags: form.tags,
      body: form.body,
      error: Some("Title is required.".to_string()),
      artifact,
    }
    .into_response();
  }

  let kind = {
    let k = form.kind.trim().to_string();
    if k.is_empty() { None } else { Some(k) }
  };

  let tags: Vec<String> = form
    .tags
    .split(',')
    .map(|t| t.trim().to_string())
    .filter(|t| !t.is_empty())
    .collect();

  let patch = ArtifactPatch {
    body: Some(form.body),
    kind,
    metadata: None,
    tags: Some(tags),
    title: Some(title),
  };

  match store::update_artifact(&state.settings, &resolved, patch) {
    Ok(_) => Redirect::to(&format!("/artifacts/{}", resolved)).into_response(),
    Err(e) => {
      log::error!("failed to update artifact {resolved}: {e}");
      (StatusCode::INTERNAL_SERVER_ERROR, Html(format!("<p>error: {e}</p>"))).into_response()
    }
  }
}

/// POST /artifacts/:id/archive — archive an artifact.
pub async fn artifact_archive(State(state): State<ServerState>, Path(id_str): Path<String>) -> Response {
  let resolved = match store::resolve_artifact_id(&state.settings, &id_str, true) {
    Ok(id) => id,
    Err(_) => return (StatusCode::NOT_FOUND, Html("<p>artifact not found</p>")).into_response(),
  };

  match store::archive_artifact(&state.settings, &resolved) {
    Ok(()) => Redirect::to("/artifacts").into_response(),
    Err(e) => {
      log::error!("failed to archive artifact {resolved}: {e}");
      (StatusCode::INTERNAL_SERVER_ERROR, Html(format!("<p>error: {e}</p>"))).into_response()
    }
  }
}

/// GET /iterations — iteration list filtered by status.
pub async fn iteration_list(State(state): State<ServerState>, Query(params): Query<IterationListParams>) -> Response {
  let filter = IterationFilter {
    all: true,
    ..Default::default()
  };

  let all_iterations = match store::list_iterations(&state.settings, &filter) {
    Ok(iterations) => iterations,
    Err(e) => {
      log::error!("failed to list iterations: {e}");
      return (
        StatusCode::INTERNAL_SERVER_ERROR,
        Html("<p>failed to load iterations</p>"),
      )
        .into_response();
    }
  };

  let active_count = all_iterations
    .iter()
    .filter(|i| i.status == IterationStatus::Active)
    .count();
  let completed_count = all_iterations
    .iter()
    .filter(|i| i.status == IterationStatus::Completed)
    .count();
  let failed_count = all_iterations
    .iter()
    .filter(|i| i.status == IterationStatus::Failed)
    .count();
  let current_status = params.status.unwrap_or_else(|| "active".to_string());

  let iterations: Vec<_> = all_iterations
    .into_iter()
    .filter(|i| i.status.as_str() == current_status)
    .collect();

  IterationListTemplate {
    iterations,
    current_status,
    active_count,
    completed_count,
    failed_count,
  }
  .into_response()
}

/// GET /iterations/:id — iteration detail with phase graph.
pub async fn iteration_detail(State(state): State<ServerState>, Path(id_str): Path<String>) -> Response {
  let id = match store::resolve_iteration_id(&state.settings, &id_str, true) {
    Ok(id) => id,
    Err(_) => return (StatusCode::NOT_FOUND, Html("<p>iteration not found</p>")).into_response(),
  };

  let iteration = match store::read_iteration(&state.settings, &id) {
    Ok(it) => it,
    Err(e) => {
      log::error!("failed to read iteration {id}: {e}");
      return (StatusCode::INTERNAL_SERVER_ERROR, Html(format!("<p>error: {e}</p>"))).into_response();
    }
  };

  let tasks = store::read_iteration_tasks(&state.settings, &iteration);

  let mut phase_map: BTreeMap<u16, Vec<crate::model::Task>> = BTreeMap::new();
  for task in &tasks {
    let phase = task.phase.unwrap_or(0);
    phase_map.entry(phase).or_default().push(task.clone());
  }
  let phases: Vec<PhaseGroup> = phase_map
    .into_iter()
    .map(|(number, tasks)| PhaseGroup {
      number,
      tasks,
    })
    .collect();

  IterationDetailTemplate {
    iteration,
    tasks,
    phases,
  }
  .into_response()
}

/// GET /iterations/:id/board — iteration kanban board.
pub async fn iteration_board(State(state): State<ServerState>, Path(id_str): Path<String>) -> Response {
  let id = match store::resolve_iteration_id(&state.settings, &id_str, true) {
    Ok(id) => id,
    Err(_) => return (StatusCode::NOT_FOUND, Html("<p>iteration not found</p>")).into_response(),
  };

  let iteration = match store::read_iteration(&state.settings, &id) {
    Ok(it) => it,
    Err(e) => {
      log::error!("failed to read iteration {id}: {e}");
      return (StatusCode::INTERNAL_SERVER_ERROR, Html(format!("<p>error: {e}</p>"))).into_response();
    }
  };

  let tasks = store::read_iteration_tasks(&state.settings, &iteration);

  let open_tasks: Vec<_> = tasks.iter().filter(|t| t.status == Status::Open).cloned().collect();
  let in_progress_tasks: Vec<_> = tasks
    .iter()
    .filter(|t| t.status == Status::InProgress)
    .cloned()
    .collect();
  let done_tasks: Vec<_> = tasks.iter().filter(|t| t.status == Status::Done).cloned().collect();
  let cancelled_tasks: Vec<_> = tasks
    .iter()
    .filter(|t| t.status == Status::Cancelled)
    .cloned()
    .collect();

  IterationBoardTemplate {
    iteration,
    open_tasks,
    in_progress_tasks,
    done_tasks,
    cancelled_tasks,
  }
  .into_response()
}

/// GET /search — search across all entity types.
pub async fn search(State(state): State<ServerState>, Query(params): Query<SearchParams>) -> Response {
  if params.q.is_empty() {
    return SearchTemplate {
      query: String::new(),
      tasks: Vec::new(),
      artifacts: Vec::new(),
      iterations: Vec::new(),
      task_count: 0,
      artifact_count: 0,
      iteration_count: 0,
    }
    .into_response();
  }

  let results = match store::search(&state.settings, &params.q, true) {
    Ok(r) => r,
    Err(e) => {
      log::error!("search failed: {e}");
      return (StatusCode::INTERNAL_SERVER_ERROR, Html("<p>search failed</p>")).into_response();
    }
  };
  let task_count = results.tasks.len();
  let artifact_count = results.artifacts.len();
  let iteration_count = results.iterations.len();

  SearchTemplate {
    query: params.q,
    tasks: results.tasks,
    artifacts: results.artifacts,
    iterations: results.iterations,
    task_count,
    artifact_count,
    iteration_count,
  }
  .into_response()
}

// ── API endpoints ─────────────────────────────────────────────────────────────

/// GET /api/search?q=... — JSON search results.
pub async fn api_search(State(state): State<ServerState>, Query(params): Query<SearchParams>) -> Response {
  if params.q.is_empty() {
    return Json(Vec::<ApiSearchResult>::new()).into_response();
  }

  let results = match store::search(&state.settings, &params.q, true) {
    Ok(r) => r,
    Err(e) => {
      log::error!("api search failed: {e}");
      return (StatusCode::INTERNAL_SERVER_ERROR, Json(Vec::<ApiSearchResult>::new())).into_response();
    }
  };

  let mut items: Vec<ApiSearchResult> = Vec::new();
  for task in results.tasks {
    items.push(ApiSearchResult {
      kind: "task".to_string(),
      id: task.id.to_string(),
      short_id: task.id.short(),
      title: task.title,
    });
  }
  for artifact in results.artifacts {
    items.push(ApiSearchResult {
      kind: "artifact".to_string(),
      id: artifact.id.to_string(),
      short_id: artifact.id.short(),
      title: artifact.title,
    });
  }
  for iteration in results.iterations {
    items.push(ApiSearchResult {
      kind: "iteration".to_string(),
      id: iteration.id.to_string(),
      short_id: iteration.id.short(),
      title: iteration.title,
    });
  }

  Json(items).into_response()
}

/// POST /api/render-markdown — render Markdown to HTML.
pub async fn api_render_markdown(Json(payload): Json<RenderMarkdownBody>) -> Response {
  let html_output = render_markdown(&payload.body);
  (
    [(axum::http::header::CONTENT_TYPE, "text/html; charset=utf-8")],
    html_output,
  )
    .into_response()
}

/// POST /tasks/:id/notes — add a note to a task.
pub async fn note_add(
  State(state): State<ServerState>,
  Path(id_str): Path<String>,
  Form(form): Form<NoteFormData>,
) -> Response {
  let id = match store::resolve_task_id(&state.settings, &id_str, true) {
    Ok(id) => id,
    Err(_) => return (StatusCode::NOT_FOUND, Html("<p>404 — task not found</p>")).into_response(),
  };

  let body = form.body.trim().to_string();
  if body.is_empty() {
    return Redirect::to(&format!("/tasks/{id}")).into_response();
  }

  let (author, author_email, author_type) = {
    match crate::cli::git::resolve_author() {
      Some(git_author) => (git_author.name, git_author.email, AuthorType::Human),
      None => ("web".to_string(), None, AuthorType::Human),
    }
  };

  let new = crate::model::NewNote {
    author,
    author_email,
    author_type,
    body,
  };

  match store::note::add_note(&state.settings, &id, new) {
    Ok(_) => Redirect::to(&format!("/tasks/{id}")).into_response(),
    Err(e) => {
      log::error!("failed to add note: {e}");
      (StatusCode::INTERNAL_SERVER_ERROR, Html(format!("<p>error: {e}</p>"))).into_response()
    }
  }
}

/// Fallback handler for unmatched routes.
pub async fn not_found() -> Response {
  (StatusCode::NOT_FOUND, Html("<p>404 — not found</p>")).into_response()
}
