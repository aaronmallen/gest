//! Artifact list/detail/create/edit handlers.

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
    model::{artifact, note, primitives::EntityType},
    repo,
  },
  web::{
    self, AppState,
    forms::{self, ExistingLink, NoteFormData},
    markdown,
    timeline::{self, TimelineItem},
  },
};

/// Form body for artifact create and edit submissions.
#[derive(Deserialize)]
pub struct ArtifactForm {
  body: Option<String>,
  tags: Option<String>,
  title: String,
}

/// Query parameters for the artifact list view (status tab selection).
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

/// Shared data backing both the full artifact detail page and the SSE fragment.
struct ArtifactDetailData {
  artifact: artifact::Model,
  body_html: String,
  tags: Vec<String>,
  timeline_items: Vec<TimelineItem>,
}

#[derive(Template)]
#[template(path = "artifacts/detail_content.html")]
struct ArtifactDetailContentTemplate {
  artifact: artifact::Model,
  body_html: String,
  tags: Vec<String>,
  timeline_items: Vec<TimelineItem>,
}

#[derive(Template)]
#[template(path = "artifacts/detail.html")]
struct ArtifactDetailTemplate {
  artifact: artifact::Model,
  body_html: String,
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

/// Shared data backing both the full artifact list page and the SSE fragment.
struct ArtifactListData {
  archived_count: usize,
  artifacts: Vec<ArtifactRow>,
  current_status: String,
  open_count: usize,
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
  link_count: usize,
  tags: Vec<String>,
}

/// Archive an artifact.
pub async fn artifact_archive(State(state): State<AppState>, Path(id): Path<String>) -> Result<Redirect, web::Error> {
  log::debug!("artifact_archive: artifact={id}");
  let conn = state.store().connect().await?;
  let artifact_id = repo::resolve::resolve_id(&conn, repo::resolve::Table::Artifacts, &id).await?;
  repo::artifact::archive(&conn, &artifact_id).await?;

  let _ = state.reload_tx().send(());
  Ok(Redirect::to("/artifacts"))
}

/// Artifact create form.
pub async fn artifact_create_form() -> Result<Html<String>, web::Error> {
  let tmpl = ArtifactCreateTemplate {
    title: String::new(),
    body: String::new(),
    tags: String::new(),
    error: None,
  };
  Ok(Html(tmpl.render()?))
}

/// Handle artifact creation from form.
pub async fn artifact_create_submit(
  State(state): State<AppState>,
  Form(form): Form<ArtifactForm>,
) -> Result<Redirect, web::Error> {
  log::debug!("artifact_create_submit: title={}", form.title);
  let conn = state.store().connect().await?;

  let new = artifact::New {
    title: form.title,
    body: form.body.unwrap_or_default(),
    ..Default::default()
  };
  let artifact = repo::artifact::create(&conn, state.project_id(), &new).await?;

  // Attach tags
  if let Some(tags_str) = &form.tags {
    for label in forms::parse_tags(tags_str) {
      repo::tag::attach(&conn, EntityType::Artifact, artifact.id(), &label).await?;
    }
  }

  let _ = state.reload_tx().send(());
  Ok(Redirect::to(&format!("/artifacts/{}", artifact.id())))
}

/// Artifact detail page.
pub async fn artifact_detail(
  State(state): State<AppState>,
  Path(id): Path<String>,
) -> Result<Html<String>, web::Error> {
  let data = load_artifact_detail(&state, &id).await?;
  let tmpl = ArtifactDetailTemplate {
    artifact: data.artifact,
    body_html: data.body_html,
    tags: data.tags,
    timeline_items: data.timeline_items,
  };
  Ok(Html(tmpl.render()?))
}

/// Artifact detail fragment (SSE live reload).
pub async fn artifact_detail_fragment(
  State(state): State<AppState>,
  Path(id): Path<String>,
) -> Result<Html<String>, web::Error> {
  let data = load_artifact_detail(&state, &id).await?;
  let tmpl = ArtifactDetailContentTemplate {
    artifact: data.artifact,
    body_html: data.body_html,
    tags: data.tags,
    timeline_items: data.timeline_items,
  };
  Ok(Html(tmpl.render()?))
}

/// Artifact edit form.
pub async fn artifact_edit_form(
  State(state): State<AppState>,
  Path(id): Path<String>,
) -> Result<Html<String>, web::Error> {
  let conn = state.store().connect().await?;
  let artifact_id = repo::resolve::resolve_id(&conn, repo::resolve::Table::Artifacts, &id).await?;
  let artifact = repo::artifact::find_by_id(&conn, artifact_id.clone())
    .await?
    .ok_or(web::Error::NotFound)?;

  let tags = repo::tag::for_entity(&conn, EntityType::Artifact, &artifact_id).await?;

  let rels = repo::relationship::for_entity(&conn, EntityType::Artifact, &artifact_id).await?;
  let existing_links = forms::build_existing_links_for_entity(&artifact_id, EntityType::Artifact, &rels);

  let tmpl = ArtifactEditTemplate {
    title: artifact.title().to_owned(),
    body: artifact.body().to_owned(),
    tags: tags.join(", "),
    artifact,
    error: None,
    existing_links,
  };
  Ok(Html(tmpl.render()?))
}

/// Artifact list page.
pub async fn artifact_list(
  State(state): State<AppState>,
  Query(params): Query<ArtifactListParams>,
) -> Result<Html<String>, web::Error> {
  let data = load_artifact_list(&state, params.status).await?;
  let tmpl = ArtifactListTemplate {
    artifacts: data.artifacts,
    open_count: data.open_count,
    archived_count: data.archived_count,
    current_status: data.current_status,
  };
  Ok(Html(tmpl.render()?))
}

/// Artifact list fragment (SSE live reload).
pub async fn artifact_list_fragment(
  State(state): State<AppState>,
  Query(params): Query<ArtifactListParams>,
) -> Result<Html<String>, web::Error> {
  let data = load_artifact_list(&state, params.status).await?;
  let tmpl = ArtifactListContentTemplate {
    artifacts: data.artifacts,
    open_count: data.open_count,
    archived_count: data.archived_count,
    current_status: data.current_status,
  };
  Ok(Html(tmpl.render()?))
}

/// Add a note to an artifact.
pub async fn artifact_note_add(
  State(state): State<AppState>,
  Path(id): Path<String>,
  Form(form): Form<NoteFormData>,
) -> Result<Redirect, web::Error> {
  log::debug!("artifact_note_add: artifact={id}");
  let conn = state.store().connect().await?;
  let artifact_id = repo::resolve::resolve_id(&conn, repo::resolve::Table::Artifacts, &id).await?;

  let new = note::New {
    body: form.body,
    author_id: state.author_id().clone(),
  };
  repo::note::create(&conn, EntityType::Artifact, &artifact_id, &new).await?;

  let _ = state.reload_tx().send(());
  Ok(Redirect::to(&format!("/artifacts/{artifact_id}")))
}

/// Handle artifact update from form.
pub async fn artifact_update(
  State(state): State<AppState>,
  Path(id): Path<String>,
  body: Bytes,
) -> Result<Redirect, web::Error> {
  log::debug!("artifact_update: artifact={id}");
  let conn = state.store().connect().await?;
  let artifact_id = repo::resolve::resolve_id(&conn, repo::resolve::Table::Artifacts, &id).await?;

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
  repo::artifact::update(&conn, &artifact_id, &patch).await?;

  // Re-sync tags: detach all, then re-attach
  repo::tag::detach_all(&conn, EntityType::Artifact, &artifact_id).await?;
  for label in forms::parse_tags(&tags_str) {
    repo::tag::attach(&conn, EntityType::Artifact, &artifact_id, &label).await?;
  }

  // Sync relationships
  forms::sync_form_links(&conn, EntityType::Artifact, &artifact_id, &link_rels, &link_refs)
    .await
    .map_err(web::Error::Internal)?;

  let _ = state.reload_tx().send(());
  Ok(Redirect::to(&format!("/artifacts/{artifact_id}")))
}

/// Build the enriched artifact rows for a page of artifacts, batching tag and
/// relationship lookups into single queries to avoid N+1 fan-out across the
/// row set.
async fn build_artifact_rows(
  conn: &Connection,
  artifacts: Vec<artifact::Model>,
) -> Result<Vec<ArtifactRow>, web::Error> {
  if artifacts.is_empty() {
    return Ok(Vec::new());
  }

  let ids: Vec<_> = artifacts.iter().map(|a| a.id().clone()).collect();
  let tags_by_id = repo::tag::for_entities(conn, EntityType::Artifact, &ids).await?;
  let rels_by_id = repo::relationship::for_entities(conn, EntityType::Artifact, &ids).await?;

  let mut rows = Vec::with_capacity(artifacts.len());
  for a in artifacts {
    let tags = tags_by_id
      .get(a.id())
      .map(|ts| ts.iter().map(|t| t.label().to_owned()).collect::<Vec<_>>())
      .unwrap_or_default();
    let link_count = rels_by_id.get(a.id()).map_or(0, Vec::len);
    rows.push(ArtifactRow {
      artifact: a,
      link_count,
      tags,
    });
  }

  Ok(rows)
}

/// Load the shared artifact detail payload used by both the full page and the
/// fragment handler.
async fn load_artifact_detail(state: &AppState, id: &str) -> Result<ArtifactDetailData, web::Error> {
  let conn = state.store().connect().await?;
  let artifact_id = repo::resolve::resolve_id(&conn, repo::resolve::Table::Artifacts, id).await?;
  let artifact = repo::artifact::find_by_id(&conn, artifact_id.clone())
    .await?
    .ok_or(web::Error::NotFound)?;

  let tags = repo::tag::for_entity(&conn, EntityType::Artifact, &artifact_id).await?;

  let body_html = markdown::render_markdown_to_html(artifact.body());
  let timeline_items = timeline::build_timeline(&conn, EntityType::Artifact, &artifact_id)
    .await
    .map_err(web::Error::Internal)?;

  Ok(ArtifactDetailData {
    artifact,
    body_html,
    tags,
    timeline_items,
  })
}

/// Load the shared artifact list payload (rows, counts, current status filter)
/// used by both the full page and the fragment handler.
async fn load_artifact_list(state: &AppState, status: Option<String>) -> Result<ArtifactListData, web::Error> {
  let conn = state.store().connect().await?;

  // Fetch all artifacts to compute counts
  let all_artifacts = repo::artifact::all(
    &conn,
    state.project_id(),
    &artifact::Filter {
      all: true,
      ..Default::default()
    },
  )
  .await?;

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

  let artifacts = repo::artifact::all(&conn, state.project_id(), &filter).await?;
  let rows = build_artifact_rows(&conn, artifacts).await?;

  Ok(ArtifactListData {
    archived_count,
    artifacts: rows,
    current_status,
    open_count,
  })
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

  mod artifact_not_found_response {
    use axum::{
      extract::{Path, State},
      http::StatusCode,
      response::IntoResponse,
    };
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::web::{self, AppState};

    async fn empty_state() -> AppState {
      let (store_arc, tmp) = store::open_temp().await.unwrap();
      let conn = store_arc.connect().await.unwrap();
      let project = Project::new("/tmp/web-artifact-not-found".into());
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
      std::mem::forget(tmp);
      AppState::new(store_arc, project.id().clone())
    }

    #[tokio::test]
    async fn it_renders_404_when_the_artifact_prefix_does_not_resolve() {
      let state = empty_state().await;

      let err = super::super::artifact_detail(State(state), Path("llllllll".into()))
        .await
        .expect_err("missing artifact should surface as NotFound");

      assert!(matches!(err, web::Error::NotFound), "got {err:?}");
      assert_eq!(err.into_response().status(), StatusCode::NOT_FOUND);
    }
  }
}
