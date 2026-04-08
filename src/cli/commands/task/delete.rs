use clap::Args;

use crate::{
  AppContext,
  cli::{Error, prompt},
  store::{
    model::primitives::{EntityType, Id},
    repo,
    sync::{paths, tombstone},
  },
  ui::{components::SuccessMessage, json},
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
  pub async fn call(&self, context: &AppContext) -> Result<(), Error> {
    log::debug!("task delete: entry");
    let project_id = context.project_id().as_ref().ok_or(Error::UninitializedProject)?;
    let conn = context.store().connect().await?;

    let id = repo::resolve::resolve_id(&conn, "tasks", &self.id).await?;
    let task = repo::task::find_by_id(&conn, id.clone())
      .await?
      .ok_or_else(|| Error::Resolve(repo::resolve::Error::NotFound(self.id.clone())))?;

    let iteration_ids = iteration_memberships(&conn, task.id()).await?;
    if !iteration_ids.is_empty() && !self.force {
      let shorts: Vec<String> = iteration_ids.iter().map(|i| i.short()).collect();
      return Err(Error::Argument(format!(
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

    let tx = repo::transaction::begin(&conn, project_id, "task delete").await?;
    let report = repo::entity::delete::delete_with_cascade(&conn, tx.id(), EntityType::Task, task.id()).await?;

    let deleted_at = chrono::Utc::now();
    tombstone::tombstone_task(context.gest_dir().as_deref(), task.id(), deleted_at)?;
    invalidate_sync_digest(&conn, project_id, task.id()).await?;

    let short_id = task.id().short();
    self.output.print_entity(&task, &short_id, || {
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

/// Drop the digest-cache entry for the tombstoned task file so that a
/// follow-up `undo` can rewrite a clean file from the restored row.
async fn invalidate_sync_digest(conn: &libsql::Connection, project_id: &Id, task_id: &Id) -> Result<(), Error> {
  let relative = format!("{}/{}.yaml", paths::TASK_DIR, task_id);
  conn
    .execute(
      "DELETE FROM sync_digests WHERE relative_path = ?1 AND project_id = ?2",
      [relative, project_id.to_string()],
    )
    .await
    .map_err(crate::store::Error::from)?;
  Ok(())
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
    .map_err(|e| Error::Store(crate::store::Error::from(e)))?;
  let mut out = Vec::new();
  while let Some(row) = rows
    .next()
    .await
    .map_err(|e| Error::Store(crate::store::Error::from(e)))?
  {
    let id_str: String = row.get(0).map_err(|e| Error::Store(crate::store::Error::from(e)))?;
    let id: Id = id_str
      .parse()
      .map_err(|e: String| Error::Argument(format!("invalid iteration id in iteration_tasks: {e}")))?;
    out.push(id);
  }
  Ok(out)
}
