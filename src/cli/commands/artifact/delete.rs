use clap::Args;

use crate::{
  AppContext,
  cli::{Error, prompt},
  store::{model::primitives::EntityType, repo, sync::tombstone},
  ui::{components::SuccessMessage, json},
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

    let id = repo::resolve::resolve_id(&conn, "artifacts", &self.id).await?;
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

    let tx = repo::transaction::begin(&conn, project_id, "artifact delete").await?;
    let report = repo::entity::delete::delete_with_cascade(&conn, tx.id(), EntityType::Artifact, artifact.id()).await?;

    let deleted_at = chrono::Utc::now();
    tombstone::tombstone_artifact(context.gest_dir().as_deref(), artifact.id(), deleted_at)?;
    // Drop the sync-digest entry for the tombstoned file so that if the user
    // later runs `undo`, the subsequent export will see the cache as stale
    // and rewrite a clean (non-tombstoned) file from the restored row. Without
    // this, the cached digest still matches the pre-delete content and the
    // stale tombstone would be re-imported and delete the row again.
    if let Some(gest_dir) = context.gest_dir().as_deref() {
      let relative = format!("{}/{}.md", crate::store::sync::paths::ARTIFACT_DIR, artifact.id());
      conn
        .execute(
          "DELETE FROM sync_digests WHERE relative_path = ?1 AND project_id = ?2",
          [relative, project_id.to_string()],
        )
        .await
        .map_err(crate::store::Error::from)?;
      let _ = gest_dir;
    }

    let short_id = artifact.id().short();
    self.output.print_entity(&artifact, &short_id, || {
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
