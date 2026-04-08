//! Artifact list/detail/create/edit handlers.

use askama::Template;
use axum::{
  body::Bytes,
  extract::{Form, Path, Query, State},
  response::{Html, Redirect},
};
use serde::Deserialize;

use crate::{
  store::{
    model::{artifact, note, primitives::EntityType},
    repo,
  },
  web::{
    AppState,
    forms::{self, ExistingLink, NoteFormData},
    handlers::log_err,
    markdown,
    note_display::{self, NoteDisplay},
    timeline::{self, TimelineItem},
  },
};

#[derive(Deserialize)]
pub struct ArtifactForm {
  body: Option<String>,
  tags: Option<String>,
  title: String,
}

#[derive(Deserialize)]
pub struct ArtifactListParams {
  status: Option<String>,
}

#[derive(Template)]
#[template(path = "artifacts/create.html")]
struct ArtifactCreateTemplate {
  body: String,
  error: Option<String>,
  tags: String,
  title: String,
}

#[derive(Template)]
#[template(path = "artifacts/detail_content.html")]
struct ArtifactDetailContentTemplate {
  artifact: artifact::Model,
  body_html: String,
  notes: Vec<NoteDisplay>,
  tags: Vec<String>,
  timeline_items: Vec<TimelineItem>,
}

#[derive(Template)]
#[template(path = "artifacts/detail.html")]
struct ArtifactDetailTemplate {
  artifact: artifact::Model,
  body_html: String,
  notes: Vec<NoteDisplay>,
  tags: Vec<String>,
  timeline_items: Vec<TimelineItem>,
}

#[derive(Template)]
#[template(path = "artifacts/edit.html")]
struct ArtifactEditTemplate {
  artifact: artifact::Model,
  body: String,
  error: Option<String>,
  existing_links: Vec<ExistingLink>,
  tags: String,
  title: String,
}

#[derive(Template)]
#[template(path = "artifacts/list_content.html")]
struct ArtifactListContentTemplate {
  archived_count: usize,
  artifacts: Vec<ArtifactRow>,
  current_status: String,
  open_count: usize,
}

#[derive(Template)]
#[template(path = "artifacts/list.html")]
struct ArtifactListTemplate {
  archived_count: usize,
  artifacts: Vec<ArtifactRow>,
  current_status: String,
  open_count: usize,
}

struct ArtifactRow {
  artifact: artifact::Model,
  tags: Vec<String>,
}

/// Archive an artifact.
pub async fn artifact_archive(State(state): State<AppState>, Path(id): Path<String>) -> Result<Redirect, String> {
  log::debug!("artifact_archive: artifact={id}");
  let conn = state.store().connect().await.map_err(log_err("artifact_archive"))?;
  let artifact_id = repo::resolve::resolve_id(&conn, "artifacts", &id)
    .await
    .map_err(log_err("artifact_archive"))?;
  repo::artifact::archive(&conn, &artifact_id)
    .await
    .map_err(log_err("artifact_archive"))?;

  let _ = state.reload_tx().send(());
  Ok(Redirect::to("/artifacts"))
}

/// Artifact create form.
pub async fn artifact_create_form() -> Result<Html<String>, String> {
  let tmpl = ArtifactCreateTemplate {
    title: String::new(),
    body: String::new(),
    tags: String::new(),
    error: None,
  };
  Ok(Html(tmpl.render().map_err(log_err("artifact_create_form"))?))
}

/// Handle artifact creation from form.
pub async fn artifact_create_submit(
  State(state): State<AppState>,
  Form(form): Form<ArtifactForm>,
) -> Result<Redirect, String> {
  log::debug!("artifact_create_submit: title={}", form.title);
  let conn = state
    .store()
    .connect()
    .await
    .map_err(log_err("artifact_create_submit"))?;

  let new = artifact::New {
    title: form.title,
    body: form.body.unwrap_or_default(),
    ..Default::default()
  };
  let artifact = repo::artifact::create(&conn, state.project_id(), &new)
    .await
    .map_err(log_err("artifact_create_submit"))?;

  // Attach tags
  if let Some(tags_str) = &form.tags {
    for label in forms::parse_tags(tags_str) {
      repo::tag::attach(&conn, EntityType::Artifact, artifact.id(), &label)
        .await
        .map_err(log_err("artifact_create_submit"))?;
    }
  }

  let _ = state.reload_tx().send(());
  Ok(Redirect::to(&format!("/artifacts/{}", artifact.id())))
}

/// Artifact detail page.
pub async fn artifact_detail(State(state): State<AppState>, Path(id): Path<String>) -> Result<Html<String>, String> {
  let (artifact, body_html, tags, notes, timeline_items) = build_artifact_detail_data(&state, &id).await?;
  let tmpl = ArtifactDetailTemplate {
    artifact,
    body_html,
    tags,
    notes,
    timeline_items,
  };
  Ok(Html(tmpl.render().map_err(log_err("artifact_detail"))?))
}

/// Artifact detail fragment (SSE live reload).
pub async fn artifact_detail_fragment(
  State(state): State<AppState>,
  Path(id): Path<String>,
) -> Result<Html<String>, String> {
  let (artifact, body_html, tags, notes, timeline_items) = build_artifact_detail_data(&state, &id).await?;
  let tmpl = ArtifactDetailContentTemplate {
    artifact,
    body_html,
    tags,
    notes,
    timeline_items,
  };
  Ok(Html(tmpl.render().map_err(log_err("artifact_detail_fragment"))?))
}

/// Artifact edit form.
pub async fn artifact_edit_form(State(state): State<AppState>, Path(id): Path<String>) -> Result<Html<String>, String> {
  let conn = state.store().connect().await.map_err(log_err("artifact_edit_form"))?;
  let artifact_id = repo::resolve::resolve_id(&conn, "artifacts", &id)
    .await
    .map_err(log_err("artifact_edit_form"))?;
  let artifact = repo::artifact::find_by_id(&conn, artifact_id.clone())
    .await
    .map_err(log_err("artifact_edit_form"))?
    .ok_or_else(|| {
      log::error!("artifact_edit_form: artifact not found: {id}");
      format!("artifact not found: {id}")
    })?;

  let tags = repo::tag::for_entity(&conn, EntityType::Artifact, &artifact_id)
    .await
    .map_err(log_err("artifact_edit_form"))?;

  let rels = repo::relationship::for_entity(&conn, EntityType::Artifact, &artifact_id)
    .await
    .map_err(log_err("artifact_edit_form"))?;
  let existing_links = forms::build_existing_links_for_entity(&artifact_id, EntityType::Artifact, &rels);

  let tmpl = ArtifactEditTemplate {
    title: artifact.title().to_owned(),
    body: artifact.body().to_owned(),
    tags: tags.join(", "),
    artifact,
    error: None,
    existing_links,
  };
  Ok(Html(tmpl.render().map_err(log_err("artifact_edit_form"))?))
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
  Ok(Html(tmpl.render().map_err(log_err("artifact_list"))?))
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
  Ok(Html(tmpl.render().map_err(log_err("artifact_list_fragment"))?))
}

/// Add a note to an artifact.
pub async fn artifact_note_add(
  State(state): State<AppState>,
  Path(id): Path<String>,
  Form(form): Form<NoteFormData>,
) -> Result<Redirect, String> {
  log::debug!("artifact_note_add: artifact={id}");
  let conn = state.store().connect().await.map_err(log_err("artifact_note_add"))?;
  let artifact_id = repo::resolve::resolve_id(&conn, "artifacts", &id)
    .await
    .map_err(log_err("artifact_note_add"))?;

  let new = note::New {
    body: form.body,
    author_id: state.author_id().cloned(),
  };
  repo::note::create(&conn, EntityType::Artifact, &artifact_id, &new)
    .await
    .map_err(log_err("artifact_note_add"))?;

  let _ = state.reload_tx().send(());
  Ok(Redirect::to(&format!("/artifacts/{artifact_id}")))
}

/// Handle artifact update from form.
pub async fn artifact_update(
  State(state): State<AppState>,
  Path(id): Path<String>,
  body: Bytes,
) -> Result<Redirect, String> {
  log::debug!("artifact_update: artifact={id}");
  let conn = state.store().connect().await.map_err(log_err("artifact_update"))?;
  let artifact_id = repo::resolve::resolve_id(&conn, "artifacts", &id)
    .await
    .map_err(log_err("artifact_update"))?;

  // Parse form fields from raw body
  let mut title = String::new();
  let mut body_field = String::new();
  let mut tags_str = String::new();
  let (link_rels, link_refs) = forms::extract_link_fields(&body);
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
    .map_err(log_err("artifact_update"))?;

  // Re-sync tags: detach all, then re-attach
  repo::tag::detach_all(&conn, EntityType::Artifact, &artifact_id)
    .await
    .map_err(log_err("artifact_update"))?;
  for label in forms::parse_tags(&tags_str) {
    repo::tag::attach(&conn, EntityType::Artifact, &artifact_id, &label)
      .await
      .map_err(log_err("artifact_update"))?;
  }

  // Sync relationships
  forms::sync_form_links(&conn, EntityType::Artifact, &artifact_id, &link_rels, &link_refs).await?;

  let _ = state.reload_tx().send(());
  Ok(Redirect::to(&format!("/artifacts/{artifact_id}")))
}

/// Build enriched artifact detail data.
async fn build_artifact_detail_data(
  state: &AppState,
  id: &str,
) -> Result<
  (
    artifact::Model,
    String,
    Vec<String>,
    Vec<NoteDisplay>,
    Vec<TimelineItem>,
  ),
  String,
> {
  let conn = state
    .store()
    .connect()
    .await
    .map_err(log_err("build_artifact_detail_data"))?;
  let artifact_id = repo::resolve::resolve_id(&conn, "artifacts", id)
    .await
    .map_err(log_err("build_artifact_detail_data"))?;
  let artifact = repo::artifact::find_by_id(&conn, artifact_id.clone())
    .await
    .map_err(log_err("build_artifact_detail_data"))?
    .ok_or_else(|| {
      log::error!("build_artifact_detail_data: artifact not found: {id}");
      format!("artifact not found: {id}")
    })?;

  let tags = repo::tag::for_entity(&conn, EntityType::Artifact, &artifact_id)
    .await
    .map_err(log_err("build_artifact_detail_data"))?;
  let raw_notes = repo::note::for_entity(&conn, EntityType::Artifact, &artifact_id)
    .await
    .map_err(log_err("build_artifact_detail_data"))?;

  let body_html = markdown::render_markdown_to_html(artifact.body());
  let notes = note_display::build_note_displays(&conn, raw_notes).await;
  let timeline_items = timeline::build_timeline(&conn, EntityType::Artifact, &artifact_id).await?;

  Ok((artifact, body_html, tags, notes, timeline_items))
}

/// Build the enriched artifact list data (rows with tags, counts, filtered).
async fn build_artifact_list_data(
  state: &AppState,
  status: Option<String>,
) -> Result<(Vec<ArtifactRow>, usize, usize, String), String> {
  let conn = state
    .store()
    .connect()
    .await
    .map_err(log_err("build_artifact_list_data"))?;

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
  .map_err(log_err("build_artifact_list_data"))?;

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
    .map_err(log_err("build_artifact_list_data"))?;

  let mut rows = Vec::with_capacity(artifacts.len());
  for a in artifacts {
    let tags = repo::tag::for_entity(&conn, EntityType::Artifact, a.id())
      .await
      .map_err(log_err("build_artifact_list_data"))?;
    rows.push(ArtifactRow {
      artifact: a,
      tags,
    });
  }

  Ok((rows, open_count, archived_count, current_status))
}

#[cfg(test)]
mod tests {
  use crate::{
    store::{
      self,
      model::{
        Project, artifact, note,
        primitives::{EntityType, Id},
      },
      repo,
    },
    web::timeline,
  };

  async fn setup_artifact_with_note_and_event() -> (std::sync::Arc<store::Db>, Id) {
    let (store_arc, tmp) = store::open_temp().await.unwrap();
    let conn = store_arc.connect().await.unwrap();
    let project = Project::new("/tmp/web-artifact-timeline".into());
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

    let art = repo::artifact::create(
      &conn,
      &project_id,
      &artifact::New {
        title: "Spec".into(),
        ..Default::default()
      },
    )
    .await
    .unwrap();

    let tx = repo::transaction::begin(&conn, &project_id, "artifact create")
      .await
      .unwrap();
    repo::transaction::record_semantic_event(
      &conn,
      tx.id(),
      "artifacts",
      &art.id().to_string(),
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
      EntityType::Artifact,
      art.id(),
      &note::New {
        body: "note body".into(),
        author_id: None,
      },
    )
    .await
    .unwrap();

    let art_id = art.id().clone();
    std::mem::forget(tmp);
    (store_arc, art_id)
  }

  mod artifact_detail_timeline {
    use pretty_assertions::assert_eq;

    use super::*;

    #[tokio::test]
    async fn it_merges_notes_and_semantic_events_in_chronological_order() {
      let (store_arc, art_id) = setup_artifact_with_note_and_event().await;
      let conn = store_arc.connect().await.unwrap();

      let items = timeline::build_timeline(&conn, EntityType::Artifact, &art_id)
        .await
        .unwrap();

      assert_eq!(items.len(), 2);
      assert!(items[0].as_event().is_some());
      assert!(items[1].as_note().is_some());
    }
  }
}
