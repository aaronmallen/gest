use clap::Args;

use crate::{
  AppContext,
  cli::{Error, prompt},
  store::{
    model::primitives::EntityType,
    repo,
    sync::{digest, paths, tombstone},
  },
  ui::{components::SuccessMessage, envelope::Envelope, json},
};

/// Delete an artifact and all of its dependent rows.
#[derive(Args, Debug)]
pub struct Command {
  /// The artifact ID or prefix.
  id: String,
  /// Reserved for future guards; accepted for UX consistency with other delete
  /// commands but currently has no effect — artifacts have no blocking guards.
  #[arg(long)]
  force: bool,
  #[command(flatten)]
  output: json::Flags,
  /// Skip the interactive confirmation prompt.
  #[arg(long)]
  yes: bool,
}

impl Command {
  /// Confirm and cascade-delete the artifact along with its notes, tags, and relationships, writing a tombstone file.
  pub async fn call(&self, context: &AppContext) -> Result<(), Error> {
    log::debug!("artifact delete: entry");
    let _ = self.force;
    let project_id = context.project_id().as_ref().ok_or(Error::UninitializedProject)?;
    let conn = context.store().connect().await?;

    let id = repo::resolve::resolve_id(&conn, repo::resolve::Table::Artifacts, &self.id).await?;
    let artifact = repo::artifact::find_required_by_id(&conn, id.clone()).await?;

    let notes = repo::note::for_entity(&conn, EntityType::Artifact, artifact.id()).await?;
    let tags = repo::tag::for_entity(&conn, EntityType::Artifact, artifact.id()).await?;
    let relationships = repo::relationship::for_entity(&conn, EntityType::Artifact, artifact.id()).await?;

    let target = format!(
      "artifact {} ({} notes, {} tags, {} relationships)",
      artifact.id().short(),
      notes.len(),
      tags.len(),
      relationships.len()
    );
    if !prompt::confirm_destructive("delete", &target, self.yes)? {
      log::info!("artifact delete: aborted by user");
      return Ok(());
    }

    // Build the envelope before cascade-delete removes sidecars from the database.
    let envelope = Envelope::load_one(&conn, EntityType::Artifact, artifact.id(), &artifact, true).await?;

    let tx = repo::transaction::begin(&conn, project_id, "artifact delete").await?;
    let report = repo::entity::delete::delete_with_cascade(&conn, tx.id(), EntityType::Artifact, artifact.id()).await?;

    let deleted_at = chrono::Utc::now();
    tombstone::tombstone_artifact(context.gest_dir().as_deref(), artifact.id(), deleted_at)?;
    digest::invalidate(
      &conn,
      project_id,
      &format!("{}/{}.md", paths::ARTIFACT_DIR, artifact.id()),
    )
    .await?;

    let short_id = artifact.id().short();
    self.output.print_envelope(&envelope, &short_id, || {
      log::info!("deleted artifact");
      SuccessMessage::new("deleted artifact")
        .id(short_id.clone())
        .field("title", artifact.title().to_string())
        .field("notes", report.notes.to_string())
        .field("tags", report.tags.to_string())
        .field("relationships", report.relationships.to_string())
        .to_string()
    })?;
    Ok(())
  }
}
