use clap::Args;

use crate::{
  AppContext,
  cli::Error,
  store::repo,
  ui::{components::SuccessMessage, json},
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
  pub async fn call(&self, context: &AppContext) -> Result<(), Error> {
    log::debug!("artifact archive: entry");
    let project_id = context.project_id().as_ref().ok_or(Error::UninitializedProject)?;
    let conn = context.store().connect().await?;

    let id = repo::resolve::resolve_id(&conn, "artifacts", &self.id).await?;
    let before_artifact = repo::artifact::find_by_id(&conn, id.clone())
      .await?
      .ok_or(Error::UninitializedProject)?;
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

    let prefix_len = repo::artifact::shortest_all_prefix(&conn, project_id).await?;
    let short_id = artifact.id().short();
    self.output.print_entity(&artifact, &short_id, || {
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
