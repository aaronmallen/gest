//! Task list/detail/create/edit/notes handlers.

use askama::Template;
use axum::{
  body::Bytes,
  extract::{Form, Path, Query, State},
  response::{Html, IntoResponse, Redirect, Response},
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
    self, AppState,
    forms::{self, ExistingLink, NoteFormData},
    markdown,
    timeline::{self, TimelineItem},
  },
};

/// Query parameters for the task list view (status tab selection).
#[derive(Deserialize)]
pub struct TaskListParams {
  status: Option<String>,
}

/// Shared data backing both the full task detail page and the SSE fragment.
struct TaskDetailData {
  blocking: bool,
  description_html: String,
  display_links: Vec<DisplayLink>,
  is_blocked: bool,
  tags: Vec<String>,
  task: task::Model,
  timeline_items: Vec<TimelineItem>,
}

/// Shared data backing both the full task list page and the SSE fragment.
struct TaskListData {
  cancelled_count: usize,
  current_status: String,
  done_count: usize,
  in_progress_count: usize,
  open_count: usize,
  rows: Vec<TaskRow>,
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
  description: String,
  error: Option<String>,
  priority_options: Vec<PriorityOption>,
  title: String,
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

/// Parsed scalar fields from the task update form body (link pairs are
/// extracted separately alongside the same single parsing pass).
#[derive(Default)]
struct TaskUpdateFields {
  description: String,
  link_rels: Vec<String>,
  link_refs: Vec<String>,
  priority: String,
  status: String,
  tags: String,
  title: String,
}

/// Add a note to a task.
pub async fn note_add(
  State(state): State<AppState>,
  Path(id): Path<String>,
  Form(form): Form<NoteFormData>,
) -> Result<Redirect, web::Error> {
  log::debug!("note_add: task={id}");
  let conn = state.store().connect().await?;
  let task_id = repo::resolve::resolve_id(&conn, repo::resolve::Table::Tasks, &id).await?;

  let new = note::New {
    body: form.body,
    author_id: state.author_id().clone(),
  };
  repo::note::create(&conn, EntityType::Task, &task_id, &new).await?;

  let _ = state.reload_tx().send(());
  Ok(Redirect::to(&format!("/tasks/{task_id}")))
}

/// Task create form.
pub async fn task_create_form() -> Result<Html<String>, web::Error> {
  let tmpl = TaskCreateTemplate {
    description: String::new(),
    error: None,
    priority_options: build_priority_options(None),
    title: String::new(),
  };
  Ok(Html(tmpl.render()?))
}

/// Handle task creation from form.
pub async fn task_create_submit(State(state): State<AppState>, body: Bytes) -> Result<Response, web::Error> {
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
    match parse_priority_byte(&priority_str) {
      Some(byte) => Some(byte),
      None => {
        log::error!("task_create_submit: invalid priority: {priority_str}");
        return render_create_form_error(title, description, priority_str, "invalid priority".to_owned());
      }
    }
  };

  let conn = state.store().connect().await?;
  let new = task::New {
    description,
    priority,
    title,
    ..Default::default()
  };
  let task = repo::task::create(&conn, state.project_id(), &new).await?;
  let _ = state.reload_tx().send(());
  Ok(Redirect::to(&format!("/tasks/{}", task.id())).into_response())
}

/// Task detail page.
pub async fn task_detail(State(state): State<AppState>, Path(id): Path<String>) -> Result<Html<String>, web::Error> {
  let data = load_task_detail(&state, &id).await?;
  let tmpl = TaskDetailTemplate {
    blocking: data.blocking,
    description_html: data.description_html,
    display_links: data.display_links,
    is_blocked: data.is_blocked,
    tags: data.tags,
    task: data.task,
    timeline_items: data.timeline_items,
  };
  Ok(Html(tmpl.render()?))
}

/// Task detail fragment (for SSE live reload).
pub async fn task_detail_fragment(
  State(state): State<AppState>,
  Path(id): Path<String>,
) -> Result<Html<String>, web::Error> {
  let data = load_task_detail(&state, &id).await?;
  let tmpl = TaskDetailFragmentTemplate {
    blocking: data.blocking,
    description_html: data.description_html,
    display_links: data.display_links,
    is_blocked: data.is_blocked,
    tags: data.tags,
    task: data.task,
    timeline_items: data.timeline_items,
  };
  Ok(Html(tmpl.render()?))
}

/// Task edit form.
pub async fn task_edit_form(State(state): State<AppState>, Path(id): Path<String>) -> Result<Html<String>, web::Error> {
  let conn = state.store().connect().await?;
  let task_id = repo::resolve::resolve_id(&conn, repo::resolve::Table::Tasks, &id).await?;
  let task = repo::task::find_by_id(&conn, task_id.clone())
    .await?
    .ok_or(web::Error::NotFound)?;

  let tags = repo::tag::for_entity(&conn, EntityType::Task, &task_id).await?;

  let rels = repo::relationship::for_entity(&conn, EntityType::Task, &task_id).await?;
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
  Ok(Html(tmpl.render()?))
}

/// Task list page.
pub async fn task_list(
  State(state): State<AppState>,
  Query(params): Query<TaskListParams>,
) -> Result<Html<String>, web::Error> {
  let data = load_task_list(&state, params.status).await?;
  let tmpl = TaskListTemplate {
    cancelled_count: data.cancelled_count,
    current_status: data.current_status,
    done_count: data.done_count,
    in_progress_count: data.in_progress_count,
    open_count: data.open_count,
    rows: data.rows,
  };
  Ok(Html(tmpl.render()?))
}

/// Task list fragment (for SSE live reload).
pub async fn task_list_fragment(
  State(state): State<AppState>,
  Query(params): Query<TaskListParams>,
) -> Result<Html<String>, web::Error> {
  let data = load_task_list(&state, params.status).await?;
  let tmpl = TaskListFragmentTemplate {
    cancelled_count: data.cancelled_count,
    current_status: data.current_status,
    done_count: data.done_count,
    in_progress_count: data.in_progress_count,
    open_count: data.open_count,
    rows: data.rows,
  };
  Ok(Html(tmpl.render()?))
}

/// Handle task update from edit form.
pub async fn task_update(
  State(state): State<AppState>,
  Path(id): Path<String>,
  body: Bytes,
) -> Result<Redirect, web::Error> {
  log::debug!("task_update: task={id}");
  let conn = state.store().connect().await?;
  let task_id = repo::resolve::resolve_id(&conn, repo::resolve::Table::Tasks, &id).await?;

  let fields = parse_task_update_fields(&body);

  let status: Option<TaskStatus> = if fields.status.is_empty() {
    None
  } else {
    Some(
      fields
        .status
        .parse()
        .map_err(|e: String| web::Error::BadRequest(format!("invalid status: {e}")))?,
    )
  };

  let priority: Option<Option<u8>> = if fields.priority.is_empty() {
    Some(None)
  } else {
    let byte =
      parse_priority_byte(&fields.priority).ok_or_else(|| web::Error::BadRequest("invalid priority".to_owned()))?;
    Some(Some(byte))
  };

  let patch = task::Patch {
    title: Some(fields.title),
    description: Some(fields.description),
    status,
    priority,
    ..Default::default()
  };

  repo::task::update(&conn, &task_id, &patch).await?;

  repo::tag::detach_all(&conn, EntityType::Task, &task_id).await?;
  for label in forms::parse_tags(&fields.tags) {
    repo::tag::attach(&conn, EntityType::Task, &task_id, &label).await?;
  }

  forms::sync_form_links(&conn, EntityType::Task, &task_id, &fields.link_rels, &fields.link_refs)
    .await
    .map_err(web::Error::Internal)?;

  let _ = state.reload_tx().send(());
  Ok(Redirect::to(&format!("/tasks/{task_id}")))
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
      EntityType::Task => Some(format!("/tasks/{other_id}")),
      EntityType::Artifact => Some(format!("/artifacts/{other_id}")),
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

/// Build enriched task rows from a list of tasks, batching tag and
/// relationship lookups into single queries to avoid N+1 fan-out.
async fn build_task_rows(conn: &Connection, tasks: Vec<task::Model>) -> Result<Vec<TaskRow>, web::Error> {
  if tasks.is_empty() {
    return Ok(Vec::new());
  }

  let ids: Vec<_> = tasks.iter().map(|t| t.id().clone()).collect();
  let tags_by_id = repo::tag::for_entities(conn, EntityType::Task, &ids).await?;
  let rels_by_id = repo::relationship::for_entities(conn, EntityType::Task, &ids).await?;

  let empty_rels: Vec<relationship::Model> = Vec::new();
  let mut rows = Vec::with_capacity(tasks.len());
  for task in tasks {
    let task_id = task.id().clone();
    let tags = tags_by_id
      .get(&task_id)
      .map(|ts| ts.iter().map(|t| t.label().to_owned()).collect::<Vec<_>>())
      .unwrap_or_default();
    let rels = rels_by_id.get(&task_id).unwrap_or(&empty_rels);
    let (is_blocked, blocking, blocked_by_display) = compute_blocking(&task_id, rels);

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
        is_blocked = true;
        blocked_by_ids.push(rel.target_id().short());
      }
      RelationshipType::Blocks if rel.source_id() == task_id => {
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

/// Load the shared task detail payload used by both the full page and the
/// fragment handler.
async fn load_task_detail(state: &AppState, id: &str) -> Result<TaskDetailData, web::Error> {
  let conn = state.store().connect().await?;
  let task_id = repo::resolve::resolve_id(&conn, repo::resolve::Table::Tasks, id).await?;
  let task = repo::task::find_by_id(&conn, task_id.clone())
    .await?
    .ok_or(web::Error::NotFound)?;

  let tags = repo::tag::for_entity(&conn, EntityType::Task, &task_id).await?;
  let rels = repo::relationship::for_entity(&conn, EntityType::Task, &task_id).await?;

  let description_html = if task.description().is_empty() {
    String::new()
  } else {
    markdown::render_markdown_to_html(task.description())
  };

  let (is_blocked, blocking, _) = compute_blocking(&task_id, &rels);
  let display_links = build_display_links(&task_id, &rels);

  let timeline_items = timeline::build_timeline(&conn, EntityType::Task, &task_id)
    .await
    .map_err(web::Error::Internal)?;

  Ok(TaskDetailData {
    blocking,
    description_html,
    display_links,
    is_blocked,
    tags,
    task,
    timeline_items,
  })
}

/// Load the shared task list payload used by both the full page and the
/// fragment handler.
///
/// When `status_param` is `None`, defaults to `all`. The special value `all`
/// bypasses status filtering. Status counts always reflect every task in the
/// project so the tab badges stay stable regardless of the current filter.
async fn load_task_list(state: &AppState, status_param: Option<String>) -> Result<TaskListData, web::Error> {
  let conn = state.store().connect().await?;

  let counts = repo::task::status_counts(&conn, state.project_id()).await?;

  let current_status = status_param.unwrap_or_else(|| "all".to_owned());

  let filter = match current_status.as_str() {
    "all" => task::Filter::all(),
    s => task::Filter {
      status: s.parse::<TaskStatus>().ok(),
      ..task::Filter::all()
    },
  };

  let tasks = repo::task::all(&conn, state.project_id(), &filter).await?;
  let rows = build_task_rows(&conn, tasks).await?;

  Ok(TaskListData {
    cancelled_count: counts.cancelled as usize,
    current_status,
    done_count: counts.done as usize,
    in_progress_count: counts.in_progress as usize,
    open_count: counts.open as usize,
    rows,
  })
}

/// Parse a form-supplied priority string (numeric byte or label) into the byte
/// value stored in the task row. Returns `None` for any unrecognized input.
fn parse_priority_byte(raw: &str) -> Option<u8> {
  if let Ok(byte) = raw.parse::<u8>() {
    return Priority::try_from(byte).ok().map(|_| byte);
  }
  raw.parse::<Priority>().ok().map(|p| p.into())
}

/// Parse scalar task-update fields and repeated link pairs from a url-encoded
/// form body in a single pass.
fn parse_task_update_fields(body: &[u8]) -> TaskUpdateFields {
  let mut fields = TaskUpdateFields::default();
  for (key, value) in form_urlencoded::parse(body) {
    match key.as_ref() {
      "title" => fields.title = value.into_owned(),
      "description" => fields.description = value.into_owned(),
      "status" => fields.status = value.into_owned(),
      "priority" => fields.priority = value.into_owned(),
      "tags" => fields.tags = value.into_owned(),
      "link_rel[]" => fields.link_rels.push(value.into_owned()),
      "link_ref[]" => fields.link_refs.push(value.into_owned()),
      _ => {}
    }
  }
  fields
}

/// Re-render the task create form, preserving user input and surfacing an error message.
fn render_create_form_error(
  title: String,
  description: String,
  priority_str: String,
  error: String,
) -> Result<Response, web::Error> {
  let current_priority = priority_str.parse::<u8>().ok();
  let tmpl = TaskCreateTemplate {
    description,
    error: Some(error),
    priority_options: build_priority_options(current_priority),
    title,
  };
  Ok(Html(tmpl.render()?).into_response())
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

  mod parse_priority_byte {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_accepts_numeric_bytes_in_range() {
      assert_eq!(parse_priority_byte("0"), Some(0));
      assert_eq!(parse_priority_byte("2"), Some(2));
      assert_eq!(parse_priority_byte("4"), Some(4));
    }

    #[test]
    fn it_accepts_labels_case_insensitively() {
      assert_eq!(parse_priority_byte("critical"), Some(0));
      assert_eq!(parse_priority_byte("Medium"), Some(2));
      assert_eq!(parse_priority_byte("LOWEST"), Some(4));
    }

    #[test]
    fn it_rejects_out_of_range_bytes() {
      assert_eq!(parse_priority_byte("9"), None);
      assert_eq!(parse_priority_byte("255"), None);
    }

    #[test]
    fn it_rejects_unknown_labels() {
      assert_eq!(parse_priority_byte("urgent"), None);
      assert_eq!(parse_priority_byte(""), None);
    }
  }

  mod parse_task_update_fields {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_collects_scalars_and_link_pairs_in_a_single_pass() {
      let body =
        b"title=Refactor&description=body&status=in-progress&priority=high&tags=a,b&link_rel[]=blocks&link_ref[]=tasks/abc&link_rel[]=blocked-by&link_ref[]=tasks/def";

      let fields = parse_task_update_fields(body);

      assert_eq!(fields.title, "Refactor");
      assert_eq!(fields.description, "body");
      assert_eq!(fields.status, "in-progress");
      assert_eq!(fields.priority, "high");
      assert_eq!(fields.tags, "a,b");
      assert_eq!(fields.link_rels, vec!["blocks", "blocked-by"]);
      assert_eq!(fields.link_refs, vec!["tasks/abc", "tasks/def"]);
    }

    #[test]
    fn it_returns_empty_defaults_when_no_known_keys_are_present() {
      let fields = parse_task_update_fields(b"unrelated=value");

      assert!(fields.title.is_empty());
      assert!(fields.link_rels.is_empty());
      assert!(fields.link_refs.is_empty());
    }
  }

  mod task_create_submit {
    use axum::{
      body::{Bytes, to_bytes},
      extract::State,
      http::StatusCode,
    };
    use pretty_assertions::assert_eq;

    use super::*;

    #[tokio::test]
    async fn it_re_renders_the_create_form_with_preserved_input_when_priority_is_out_of_range() {
      let state = setup().await;

      let body = Bytes::from("title=bad-priority&description=keep+this&priority=9");
      let response = super::super::task_create_submit(State(state), body)
        .await
        .expect("validation failure should re-render, not error");

      let status = response.status();
      let body_bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
      let html = String::from_utf8(body_bytes.to_vec()).unwrap();

      assert_eq!(status, StatusCode::OK);
      assert!(html.contains("invalid priority"));
      assert!(html.contains("value=\"bad-priority\""));
      assert!(html.contains("keep this"));
      assert!(html.contains("md-preview-toggle"));
    }

    #[tokio::test]
    async fn it_treats_an_empty_priority_as_no_priority_and_redirects() {
      let state = setup().await;

      let body = Bytes::from("title=no-priority&description=&priority=");
      let response = super::super::task_create_submit(State(state), body)
        .await
        .expect("empty priority should succeed");

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
    async fn it_renders_html_404_when_a_well_formed_prefix_matches_no_task() {
      let state = setup().await;

      // Valid base32 characters but no matching task: should propagate as NotFound → 404.
      let err = task_detail(State(state), Path("zzzzzzzz".into()))
        .await
        .expect_err("unknown id prefix should propagate as not-found");
      let response = err.into_response();
      let status = response.status();
      let body_bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
      let body = String::from_utf8(body_bytes.to_vec()).unwrap();

      assert_eq!(status, StatusCode::NOT_FOUND);
      assert!(body.contains("<html"));
      assert!(body.contains("404"));
    }

    #[tokio::test]
    async fn it_renders_html_500_when_the_id_prefix_is_malformed() {
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

  mod load_task_list {
    use pretty_assertions::assert_eq;

    use super::*;

    #[tokio::test]
    async fn it_defaults_to_all_when_no_status_given() {
      let state = setup().await;

      let data = load_task_list(&state, None).await.unwrap();

      assert_eq!(data.current_status, "all");
      assert_eq!(data.rows.len(), 5);
      assert_eq!(
        (
          data.open_count,
          data.in_progress_count,
          data.done_count,
          data.cancelled_count
        ),
        (2, 1, 1, 1),
      );
    }

    #[tokio::test]
    async fn it_filters_to_a_specific_status_at_the_db_layer() {
      let state = setup().await;

      let data = load_task_list(&state, Some("done".into())).await.unwrap();

      assert_eq!(data.current_status, "done");
      assert_eq!(data.rows.len(), 1);
      assert_eq!(data.rows[0].task.status(), TaskStatus::Done);
    }

    #[tokio::test]
    async fn it_reports_counts_across_every_status_regardless_of_filter() {
      let state = setup().await;

      let all_data = load_task_list(&state, None).await.unwrap();
      let done_data = load_task_list(&state, Some("done".into())).await.unwrap();

      assert_eq!(
        (
          all_data.open_count,
          all_data.in_progress_count,
          all_data.done_count,
          all_data.cancelled_count,
        ),
        (2, 1, 1, 1),
      );
      assert_eq!(
        (
          done_data.open_count,
          done_data.in_progress_count,
          done_data.done_count,
          done_data.cancelled_count,
        ),
        (2, 1, 1, 1),
      );
    }

    #[tokio::test]
    async fn it_returns_every_task_when_status_is_all() {
      let state = setup().await;

      let data = load_task_list(&state, Some("all".into())).await.unwrap();

      assert_eq!(data.current_status, "all");
      assert_eq!(data.rows.len(), 5);
      assert_eq!(
        (
          data.open_count,
          data.in_progress_count,
          data.done_count,
          data.cancelled_count
        ),
        (2, 1, 1, 1),
      );
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
