use clap::Args;

use crate::{
  AppContext,
  cli::{Error, prompt},
  store::{
    model::primitives::{EntityType, Id},
    repo,
    sync::{digest, paths, tombstone},
  },
  ui::{components::SuccessMessage, envelope::Envelope, json},
};

/// Delete a task and all of its dependent rows.
#[derive(Args, Debug)]
pub struct Command {
  /// The task ID or prefix.
  id: String,
  /// Remove the task from every iteration it belongs to before deleting.
  /// Without this flag the command refuses to delete a task that is still a
  /// member of one or more iterations so members of an active sprint are not
  /// removed by accident.
  #[arg(long)]
  force: bool,
  #[command(flatten)]
  output: json::Flags,
  /// Skip the interactive confirmation prompt.
  #[arg(long)]
  yes: bool,
}

impl Command {
  /// Confirm and cascade-delete the task along with its notes, tags, relationships, and iteration
  /// memberships, writing a tombstone file.
  pub async fn call(&self, context: &AppContext) -> Result<(), Error> {
    log::debug!("task delete: entry");
    let project_id = context.project_id().as_ref().ok_or(Error::UninitializedProject)?;
    let conn = context.store().connect().await?;

    let id = repo::resolve::resolve_id(&conn, repo::resolve::Table::Tasks, &self.id).await?;
    let task = repo::task::find_required_by_id(&conn, id).await?;

    let iteration_ids = iteration_memberships(&conn, task.id()).await?;
    if !iteration_ids.is_empty() && !self.force {
      let shorts: Vec<String> = iteration_ids.iter().map(|i| i.short()).collect();
      return Err(Error::InvalidState(format!(
        "task {} is a member of {} iteration(s): {}. Use --force to remove from all iterations and delete.",
        task.id().short(),
        iteration_ids.len(),
        shorts.join(", ")
      )));
    }

    let notes = repo::note::for_entity(&conn, EntityType::Task, task.id()).await?;
    let tags = repo::tag::for_entity(&conn, EntityType::Task, task.id()).await?;
    let relationships = repo::relationship::for_entity(&conn, EntityType::Task, task.id()).await?;

    let target = format!(
      "task {} ({} notes, {} tags, {} relationships, {} iteration memberships)",
      task.id().short(),
      notes.len(),
      tags.len(),
      relationships.len(),
      iteration_ids.len()
    );
    if !prompt::confirm_destructive("delete", &target, self.yes)? {
      log::info!("task delete: aborted by user");
      return Ok(());
    }

    // Capture the envelope before the cascade deletes sidecars.
    let envelope = Envelope::load_one(&conn, EntityType::Task, task.id(), &task, true).await?;

    let tx = repo::transaction::begin(&conn, project_id, "task delete").await?;
    let report = repo::entity::delete::delete_with_cascade(&conn, tx.id(), EntityType::Task, task.id()).await?;

    let deleted_at = chrono::Utc::now();
    tombstone::tombstone_task(context.gest_dir().as_deref(), task.id(), deleted_at)?;
    digest::invalidate(&conn, project_id, &format!("{}/{}.yaml", paths::TASK_DIR, task.id())).await?;

    let short_id = task.id().short();
    self.output.print_envelope(&envelope, &short_id, || {
      log::info!("deleted task");
      SuccessMessage::new("deleted task")
        .id(short_id.clone())
        .field("title", task.title().to_string())
        .field("notes", report.notes.to_string())
        .field("tags", report.tags.to_string())
        .field("relationships", report.relationships.to_string())
        .field("iteration_memberships", report.iteration_tasks.to_string())
        .to_string()
    })?;
    Ok(())
  }
}

/// Return every iteration id that currently holds this task in its
/// `iteration_tasks` join table.
async fn iteration_memberships(conn: &libsql::Connection, task_id: &Id) -> Result<Vec<Id>, Error> {
  let mut rows = conn
    .query(
      "SELECT iteration_id FROM iteration_tasks WHERE task_id = ?1 ORDER BY iteration_id",
      [task_id.to_string()],
    )
    .await
    .map_err(crate::store::Error::from)?;
  let mut out = Vec::new();
  while let Some(row) = rows.next().await.map_err(crate::store::Error::from)? {
    let id_str: String = row.get(0).map_err(crate::store::Error::from)?;
    let id: Id = id_str.parse().map_err(|e: String| {
      crate::store::Error::InvalidValue(format!("invalid iteration id in iteration_tasks: {e}"))
    })?;
    out.push(id);
  }
  Ok(out)
}
