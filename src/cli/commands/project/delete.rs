use clap::Args;

use crate::{
  AppContext,
  cli::{Error, prompt},
  store::repo::{self, resolve::Table},
  ui::{components::SuccessMessage, json},
};

/// Delete a project and all of its owned entities.
///
/// This is a hard delete that cascades through every task, iteration, and
/// artifact owned by the project, along with all their notes, tags, and
/// relationships. The operation writes tombstone files and is **not**
/// reversible via `gest undo`.
#[derive(Args, Debug)]
pub struct Command {
  /// The project ID or prefix.
  id: String,
  #[command(flatten)]
  output: json::Flags,
  /// Skip the interactive confirmation prompt.
  #[arg(long)]
  yes: bool,
}

impl Command {
  /// Confirm and cascade-delete the project along with all owned entities, writing tombstone files.
  pub async fn call(&self, context: &AppContext) -> Result<(), Error> {
    log::debug!("project delete: entry");
    let conn = context.store().connect().await?;

    let id = repo::resolve::resolve_id(&conn, Table::Projects, &self.id).await?;
    let project = repo::project::find_by_id(&conn, id.clone())
      .await?
      .ok_or_else(|| Error::Argument(format!("project {} not found", id.short())))?;

    let task_count = repo::project::count_owned(&conn, Table::Tasks, &id).await?;
    let iteration_count = repo::project::count_owned(&conn, Table::Iterations, &id).await?;
    let artifact_count = repo::project::count_owned(&conn, Table::Artifacts, &id).await?;

    let target = format!(
      "project {} ({}). This will delete {} tasks, {} iterations, {} artifacts, \
      and all associated notes, tags, and relationships. This cannot be undone",
      id.short(),
      project.root().display(),
      task_count,
      iteration_count,
      artifact_count,
    );
    if !prompt::confirm_destructive("delete", &target, self.yes)? {
      log::info!("project delete: aborted by user");
      return Ok(());
    }

    let deleted_at = chrono::Utc::now();
    let summary = repo::project::delete(&conn, &id, context.gest_dir().as_deref(), deleted_at).await?;

    let short_id = id.short();
    if self.output.json {
      let json = serde_json::json!({
        "id": id.to_string(),
        "root": project.root().display().to_string(),
        "tasks": summary.tasks,
        "iterations": summary.iterations,
        "artifacts": summary.artifacts,
        "notes": summary.notes,
        "tags": summary.tags,
        "relationships": summary.relationships,
      });
      println!("{}", serde_json::to_string_pretty(&json)?);
    } else if self.output.quiet {
      println!("{short_id}");
    } else {
      log::info!("deleted project");
      let msg = SuccessMessage::new("deleted project")
        .id(short_id)
        .field("root", project.root().display().to_string())
        .field("tasks", summary.tasks.to_string())
        .field("iterations", summary.iterations.to_string())
        .field("artifacts", summary.artifacts.to_string())
        .field("notes", summary.notes.to_string())
        .field("tags", summary.tags.to_string())
        .field("relationships", summary.relationships.to_string());
      println!("{msg}");
    }
    Ok(())
  }
}
