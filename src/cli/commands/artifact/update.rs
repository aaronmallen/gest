use clap::Args;
use serde_json::Value;

use crate::{
  AppContext,
  cli::Error,
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
  /// Set custom metadata as a JSON string (e.g. `--metadata '{"key":"value"}'`).
  #[arg(long, short)]
  metadata: Option<String>,
  /// Replace all tags on the artifact (can be repeated).
  #[arg(long, short)]
  tag: Vec<String>,
  /// Set the artifact title.
  #[arg(long, short = 'T')]
  title: Option<String>,
  #[command(flatten)]
  output: json::Flags,
}

impl Command {
  pub async fn call(&self, context: &AppContext) -> Result<(), Error> {
    let project_id = context.project_id().as_ref().ok_or(Error::UninitializedProject)?;
    let conn = context.store().connect().await?;

    let id = repo::resolve::resolve_id(&conn, "artifacts", &self.id).await?;

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

    let metadata = self.parse_metadata()?;

    let before_artifact = repo::artifact::find_by_id(&conn, id.clone())
      .await?
      .ok_or(Error::UninitializedProject)?;
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
      for label in &self.tag {
        let tag = repo::tag::attach(&conn, EntityType::Artifact, &id, label).await?;
        repo::transaction::record_event(&conn, tx.id(), "entity_tags", &tag.id().to_string(), "created", None).await?;
      }
    }

    let short_id = artifact.id().short();
    self.output.print_entity(&artifact, &short_id, || {
      SuccessMessage::new("updated artifact")
        .id(artifact.id().short())
        .field("title", artifact.title().to_string())
        .to_string()
    })?;
    Ok(())
  }

  fn parse_metadata(&self) -> Result<Option<Value>, Error> {
    match &self.metadata {
      Some(json_str) => {
        let value: Value =
          serde_json::from_str(json_str).map_err(|e| Error::Editor(format!("invalid metadata JSON: {e}")))?;
        Ok(Some(value))
      }
      None => Ok(None),
    }
  }
}
