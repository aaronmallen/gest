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

/// Delete an iteration and drop its task memberships. Tasks themselves are
/// preserved — only the `iteration_tasks` join rows are removed.
#[derive(Args, Debug)]
pub struct Command {
  /// The iteration ID or prefix.
  id: String,
  /// Reserved for future guards; accepted for UX consistency with other
  /// delete commands but currently has no effect.
  #[arg(long)]
  force: bool,
  #[command(flatten)]
  output: json::Flags,
  /// Skip the interactive confirmation prompt.
  #[arg(long)]
  yes: bool,
}

impl Command {
  /// Confirm and cascade-delete the iteration along with its task memberships, tags, notes, and relationships, writing a tombstone file.
  pub async fn call(&self, context: &AppContext) -> Result<(), Error> {
    log::debug!("iteration delete: entry");
    let _ = self.force;
    let project_id = context.project_id().as_ref().ok_or(Error::UninitializedProject)?;
    let conn = context.store().connect().await?;

    let id = repo::resolve::resolve_id(&conn, repo::resolve::Table::Iterations, &self.id).await?;
    let iteration = repo::iteration::find_required_by_id(&conn, id.clone()).await?;

    let notes = repo::note::for_entity(&conn, EntityType::Iteration, iteration.id()).await?;
    let tags = repo::tag::for_entity(&conn, EntityType::Iteration, iteration.id()).await?;
    let relationships = repo::relationship::for_entity(&conn, EntityType::Iteration, iteration.id()).await?;
    let task_memberships = repo::iteration::tasks_with_phase(&conn, iteration.id()).await?.len();

    let target = format!(
      "iteration {} ({} notes, {} tags, {} relationships, {} task memberships)",
      iteration.id().short(),
      notes.len(),
      tags.len(),
      relationships.len(),
      task_memberships
    );
    if !prompt::confirm_destructive("delete", &target, self.yes)? {
      log::info!("iteration delete: aborted by user");
      return Ok(());
    }

    // Load the envelope before deletion so sidecars are still available.
    let envelope = Envelope::load_one(&conn, EntityType::Iteration, iteration.id(), &iteration, true).await?;

    let tx = repo::transaction::begin(&conn, project_id, "iteration delete").await?;
    let report =
      repo::entity::delete::delete_with_cascade(&conn, tx.id(), EntityType::Iteration, iteration.id()).await?;

    let deleted_at = chrono::Utc::now();
    tombstone::tombstone_iteration(context.gest_dir().as_deref(), iteration.id(), deleted_at)?;
    digest::invalidate(
      &conn,
      project_id,
      &format!("{}/{}.yaml", paths::ITERATION_DIR, iteration.id()),
    )
    .await?;

    let short_id = iteration.id().short();
    self.output.print_envelope(&envelope, &short_id, || {
      log::info!("deleted iteration");
      SuccessMessage::new("deleted iteration")
        .id(short_id.clone())
        .field("title", iteration.title().to_string())
        .field("notes", report.notes.to_string())
        .field("tags", report.tags.to_string())
        .field("relationships", report.relationships.to_string())
        .field("task_memberships", report.iteration_tasks.to_string())
        .to_string()
    })?;
    Ok(())
  }
}
