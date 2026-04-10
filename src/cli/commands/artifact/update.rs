use clap::Args;
use serde_json::Value;

use crate::{
  AppContext,
  cli::{Error, meta_args, tag_arg},
  store::{
    model::{artifact::Patch, primitives::EntityType},
    repo,
  },
  ui::{components::SuccessMessage, json},
};

/// Update an artifact.
#[derive(Args, Debug)]
pub struct Command {
  /// The artifact ID or prefix.
  id: String,
  /// Set the artifact body (markdown).
  #[arg(long, short)]
  body: Option<String>,
  /// Open `$EDITOR` pre-filled with the current body for editing.
  #[arg(long, short, conflicts_with = "body")]
  edit: bool,
  /// Set a metadata key=value pair (repeatable; supports dot-paths and scalar inference).
  #[arg(long = "metadata", short = 'm', value_name = "KEY=VALUE")]
  metadata: Vec<String>,
  /// Merge a JSON object into metadata (repeatable; applied after --metadata pairs).
  #[arg(long = "metadata-json", value_name = "JSON")]
  metadata_json: Vec<String>,
  /// Replace all tags on the artifact (can be repeated; comma-separated values split into multiple tags).
  #[arg(long, short, value_delimiter = ',', value_parser = tag_arg::trim_tag)]
  tag: Vec<String>,
  /// Set the artifact title.
  #[arg(long, short = 'T')]
  title: Option<String>,
  #[command(flatten)]
  output: json::Flags,
}

impl Command {
  /// Apply title, body, metadata, and tag changes to the resolved artifact within a recorded transaction.
  pub async fn call(&self, context: &AppContext) -> Result<(), Error> {
    log::debug!("artifact update: entry");
    let project_id = context.project_id().as_ref().ok_or(Error::UninitializedProject)?;
    let conn = context.store().connect().await?;

    let id = repo::resolve::resolve_id(&conn, repo::resolve::Table::Artifacts, &self.id).await?;

    let body = if self.edit {
      let artifact = repo::artifact::find_by_id(&conn, id.clone())
        .await?
        .ok_or_else(|| Error::Editor(format!("artifact {} not found", self.id)))?;
      let edited =
        crate::io::editor::edit_text_with_suffix(artifact.body(), ".md").map_err(|e| Error::Editor(e.to_string()))?;
      if edited.trim().is_empty() {
        return Err(Error::Editor("Aborting: empty body".into()));
      }
      Some(edited)
    } else {
      self.body.clone()
    };

    let before_artifact = repo::artifact::find_by_id(&conn, id.clone())
      .await?
      .ok_or(Error::UninitializedProject)?;
    let metadata = self.build_metadata(before_artifact.metadata())?;

    let before = serde_json::to_value(&before_artifact)?;
    let tx = repo::transaction::begin(&conn, project_id, "artifact update").await?;
    let patch = Patch {
      body,
      metadata,
      title: self.title.clone(),
    };

    let artifact = repo::artifact::update(&conn, &id, &patch).await?;
    repo::transaction::record_event(&conn, tx.id(), "artifacts", &id.to_string(), "modified", Some(&before)).await?;

    if !self.tag.is_empty() {
      // Replace all tags: remove existing, then attach new ones
      let existing_tags = repo::tag::for_entity(&conn, EntityType::Artifact, &id).await?;
      for label in &existing_tags {
        repo::tag::detach(&conn, EntityType::Artifact, &id, label).await?;
      }
      for label in tag_arg::normalize_tags(&self.tag) {
        let tag = repo::tag::attach(&conn, EntityType::Artifact, &id, &label).await?;
        repo::transaction::record_event(&conn, tx.id(), "entity_tags", &tag.id().to_string(), "created", None).await?;
      }
    }

    let prefix_len = if artifact.is_archived() {
      repo::artifact::shortest_all_prefix(&conn, project_id).await?
    } else {
      repo::artifact::shortest_active_prefix(&conn, project_id).await?
    };
    let short_id = artifact.id().short();
    self.output.print_entity(&artifact, &short_id, || {
      log::info!("updated artifact");
      SuccessMessage::new("updated artifact")
        .id(artifact.id().short())
        .prefix_len(prefix_len)
        .field("title", artifact.title().to_string())
        .to_string()
    })?;
    Ok(())
  }

  fn build_metadata(&self, existing: &Value) -> Result<Option<Value>, Error> {
    if self.metadata.is_empty() && self.metadata_json.is_empty() {
      return Ok(None);
    }
    meta_args::build_metadata(Some(existing.clone()), &self.metadata, &self.metadata_json)
  }
}
