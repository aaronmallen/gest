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
    markdown,
    note_display::{self, NoteDisplay},
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
}

#[derive(Template)]
#[template(path = "artifacts/detail.html")]
struct ArtifactDetailTemplate {
  artifact: artifact::Model,
  body_html: String,
  notes: Vec<NoteDisplay>,
  tags: Vec<String>,
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
    for label in forms::parse_tags(tags_str) {
      repo::tag::attach(&conn, EntityType::Artifact, artifact.id(), &label)
        .await
        .map_err(|e| e.to_string())?;
    }
  }

  let _ = state.reload_tx().send(());
  Ok(Redirect::to(&format!("/artifacts/{}", artifact.id())))
}

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
  let existing_links = forms::build_existing_links_for_entity(&artifact_id, EntityType::Artifact, &rels);

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
    .map_err(|e| e.to_string())?;

  // Re-sync tags: detach all, then re-attach
  repo::tag::detach_all(&conn, EntityType::Artifact, &artifact_id)
    .await
    .map_err(|e| e.to_string())?;
  for label in forms::parse_tags(&tags_str) {
    repo::tag::attach(&conn, EntityType::Artifact, &artifact_id, &label)
      .await
      .map_err(|e| e.to_string())?;
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

  let body_html = markdown::render_markdown_to_html(artifact.body());
  let notes = note_display::build_note_displays(&conn, raw_notes).await;

  Ok((artifact, body_html, tags, notes))
}

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
