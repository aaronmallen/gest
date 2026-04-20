use clap::Args;

use crate::{
  AppContext,
  cli::Error,
  store::{model::primitives::EntityType, repo},
  ui::{components::SuccessMessage, envelope::Envelope, json},
};

/// Archive an artifact.
#[derive(Args, Debug)]
pub struct Command {
  /// The artifact ID or prefix.
  id: String,
  #[command(flatten)]
  output: json::Flags,
}

impl Command {
  /// Resolve the artifact and mark it archived within a recorded transaction.
  pub async fn call(&self, context: &AppContext) -> Result<(), Error> {
    log::debug!("artifact archive: entry");
    let project_id = context.project_id().as_ref().ok_or(Error::UninitializedProject)?;
    let conn = context.store().connect().await?;

    let id = repo::resolve::resolve_id(&conn, repo::resolve::Table::Artifacts, &self.id).await?;
    let before_artifact = repo::artifact::find_by_id(&conn, id.clone())
      .await?
      .ok_or_else(|| Error::NotFound(format!("artifact not found: {}", self.id)))?;
    let before = serde_json::to_value(&before_artifact)?;
    let tx = repo::transaction::begin(&conn, project_id, "artifact archive").await?;
    let artifact = repo::artifact::archive(&conn, &id).await?;
    repo::transaction::record_semantic_event(
      &conn,
      tx.id(),
      "artifacts",
      &id.to_string(),
      "modified",
      Some(&before),
      Some("archived"),
      None,
      None,
    )
    .await?;

    let id_str = artifact.id().to_string();
    let prefix_lens = repo::artifact::prefix_lengths(&conn, project_id, &[id_str.as_str()]).await?;
    let prefix_len = prefix_lens[0];
    let short_id = artifact.id().short();
    let envelope = Envelope::load_one(&conn, EntityType::Artifact, artifact.id(), &artifact, true).await?;
    self.output.print_envelope(&envelope, &short_id, || {
      log::info!("archived artifact");
      SuccessMessage::new("archived artifact")
        .id(artifact.id().short())
        .prefix_len(prefix_len)
        .field("title", artifact.title().to_string())
        .to_string()
    })?;
    Ok(())
  }
}
