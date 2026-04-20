use std::path::PathBuf;

use chrono::Utc;
use clap::Args;

use crate::{
  AppContext,
  cli::{Error, prompt},
  store::{
    model::primitives::{EntityType, Id},
    repo::{
      self,
      purge::{
        ArchivedArtifacts, ArchivedProjects, DanglingRelationships, OrphanTombstones, Scope, TerminalIterations,
        TerminalTasks,
      },
    },
    sync::tombstone,
  },
  ui::components::{SuccessMessage, Summary},
};

/// Purge terminal, archived, and orphaned data from the store.
///
/// By default operates on the current project. Pass `--all-projects` to sweep
/// the entire store. When no selector flags are given, all selectors are
/// applied (the confirmation prompt is the safety net).
#[derive(Args, Debug)]
pub struct Command {
  /// Operate across every project in the store instead of the current one.
  #[arg(long)]
  all_projects: bool,
  /// Show what would be purged without making any changes.
  #[arg(long)]
  dry_run: bool,
  /// Purge archived artifacts.
  #[arg(long)]
  artifacts: bool,
  /// Purge archived iterations (terminal: completed/cancelled).
  #[arg(long)]
  iterations: bool,
  /// Purge archived projects (non-undoable).
  #[arg(long)]
  projects: bool,
  /// Purge dangling relationship rows.
  #[arg(long)]
  relationships: bool,
  /// Purge terminal tasks (done/cancelled).
  #[arg(long)]
  tasks: bool,
  /// Purge orphan tombstone files.
  #[arg(long)]
  tombstones: bool,
  /// Skip the interactive confirmation prompt.
  #[arg(long)]
  yes: bool,
}

impl Command {
  /// Execute the purge command.
  pub async fn call(&self, context: &AppContext) -> Result<(), Error> {
    log::debug!("purge: entry");
    let conn = context.store().connect().await?;

    let scope = if self.all_projects {
      Scope::AllProjects
    } else {
      let project_id = context.project_id().as_ref().ok_or(Error::UninitializedProject)?;
      Scope::Project(project_id.clone())
    };

    // When no selector flags are given, default to all selectors.
    let all =
      !(self.tasks || self.iterations || self.artifacts || self.projects || self.relationships || self.tombstones);
    let do_tasks = all || self.tasks;
    let do_iterations = all || self.iterations;
    let do_artifacts = all || self.artifacts;
    let do_projects = all || self.projects;
    let do_relationships = all || self.relationships;
    let do_tombstones = all || self.tombstones;

    // Collect gest dirs for tombstone scanning.
    let gest_dirs = collect_gest_dirs(&conn, &scope).await?;

    // Run all selectors.
    let tasks = if do_tasks {
      repo::purge::terminal_tasks(&conn, &scope).await?
    } else {
      Default::default()
    };
    let iterations = if do_iterations {
      repo::purge::terminal_iterations(&conn, &scope).await?
    } else {
      Default::default()
    };
    let artifacts = if do_artifacts {
      repo::purge::archived_artifacts(&conn, &scope).await?
    } else {
      Default::default()
    };
    let projects = if do_projects {
      repo::purge::archived_projects(&conn, &scope).await?
    } else {
      Default::default()
    };
    let relationships = if do_relationships {
      repo::purge::dangling_relationships(&conn, &scope).await?
    } else {
      Default::default()
    };
    let tombstones = if do_tombstones {
      repo::purge::orphan_tombstones(&conn, &scope, &gest_dirs).await?
    } else {
      Default::default()
    };

    let total =
      tasks.total() + iterations.total() + artifacts.count + projects.count + relationships.count + tombstones.count;

    let summary = build_summary(&tasks, &iterations, &artifacts, &projects, &relationships, &tombstones);
    println!("{summary}");

    if total == 0 {
      return Ok(());
    }

    if self.dry_run {
      return Ok(());
    }

    if !prompt::confirm_destructive("purge", "the items listed above", self.yes)? {
      log::info!("purge: aborted by user");
      return Ok(());
    }

    // Determine the project id for the transaction. For --all-projects we use
    // the current project if available; otherwise pick the first project.
    let tx_project_id = match &scope {
      Scope::Project(pid) => pid.clone(),
      Scope::AllProjects => {
        if let Some(pid) = context.project_id().as_ref() {
          pid.clone()
        } else {
          let all = repo::project::all(&conn, true).await?;
          all
            .first()
            .ok_or_else(|| Error::NotFound("no projects in store".into()))?
            .id()
            .clone()
        }
      }
    };

    let tx = repo::transaction::begin(&conn, &tx_project_id, "purge").await?;
    let deleted_at = Utc::now();

    // Delete terminal tasks (single batched cascade query per child table).
    if !tasks.ids.is_empty() {
      let task_refs: Vec<&Id> = tasks.ids.iter().collect();
      repo::entity::delete::delete_many_with_cascade(&conn, tx.id(), EntityType::Task, &task_refs).await?;
      for task_id in &tasks.ids {
        tombstone::tombstone_task(context.gest_dir().as_deref(), task_id, deleted_at)?;
      }
    }

    // Delete terminal iterations.
    if !iterations.ids.is_empty() {
      let iter_refs: Vec<&Id> = iterations.ids.iter().collect();
      repo::entity::delete::delete_many_with_cascade(&conn, tx.id(), EntityType::Iteration, &iter_refs).await?;
      for iteration_id in &iterations.ids {
        tombstone::tombstone_iteration(context.gest_dir().as_deref(), iteration_id, deleted_at)?;
      }
    }

    // Delete archived artifacts.
    if !artifacts.ids.is_empty() {
      let artifact_refs: Vec<&Id> = artifacts.ids.iter().collect();
      repo::entity::delete::delete_many_with_cascade(&conn, tx.id(), EntityType::Artifact, &artifact_refs).await?;
      for artifact_id in &artifacts.ids {
        tombstone::tombstone_artifact(context.gest_dir().as_deref(), artifact_id, deleted_at)?;
      }
    }

    // Delete archived projects (non-undoable).
    for project_id in &projects.ids {
      let gest_dir = gest_dirs
        .iter()
        .find(|(id, _)| id == project_id)
        .map(|(_, path)| path.as_path());
      repo::project::delete(&conn, project_id, gest_dir, deleted_at).await?;
    }

    // Delete dangling relationships.
    for rel_id in &relationships.ids {
      repo::relationship::delete(&conn, rel_id).await?;
    }

    // Delete orphan tombstone files.
    for path in &tombstones.paths {
      if path.exists() {
        std::fs::remove_file(path)?;
      }
    }

    let completion = Summary::new()
      .success(SuccessMessage::new("Purged").field("total", total.to_string()))
      .hint("Run `gest undo` to restore.");
    println!("{completion}");

    Ok(())
  }
}

/// Build the purge-summary block rendered before the confirmation prompt.
///
/// When every selector reports zero the block renders with an `empty_message`
/// in place of the row list; otherwise it renders the title followed by one
/// row per non-empty selector.
fn build_summary(
  tasks: &TerminalTasks,
  iterations: &TerminalIterations,
  artifacts: &ArchivedArtifacts,
  projects: &ArchivedProjects,
  relationships: &DanglingRelationships,
  tombstones: &OrphanTombstones,
) -> Summary {
  let mut summary = Summary::new()
    .title("Purge summary:")
    .empty_message("Nothing to purge.");

  if tasks.total() > 0 {
    let detail = format!("{} done, {} cancelled", tasks.done, tasks.cancelled);
    summary = summary.row_with_detail("tasks", tasks.total(), Some(detail));
  }
  if iterations.total() > 0 {
    let detail = format!("{} completed, {} cancelled", iterations.completed, iterations.cancelled);
    summary = summary.row_with_detail("iterations", iterations.total(), Some(detail));
  }
  if artifacts.count > 0 {
    summary = summary.row("artifacts", artifacts.count);
  }
  if projects.count > 0 {
    summary = summary.row_with_detail("projects", projects.count, Some("non-undoable"));
  }
  if relationships.count > 0 {
    summary = summary.row("relationships", relationships.count);
  }
  if tombstones.count > 0 {
    summary = summary.row("tombstones", tombstones.count);
  }

  summary
}

/// Collect `(project_id, gest_dir_path)` pairs for tombstone scanning.
async fn collect_gest_dirs(conn: &libsql::Connection, scope: &Scope) -> Result<Vec<(Id, PathBuf)>, Error> {
  let projects = match scope {
    Scope::AllProjects => repo::project::all(conn, true).await?,
    Scope::Project(pid) => match repo::project::find_by_id(conn, pid.clone()).await? {
      Some(p) => vec![p],
      None => vec![],
    },
  };

  let mut dirs = Vec::new();
  for project in projects {
    let root = project.root();
    let gest_dir = root.join(".gest");
    if gest_dir.is_dir() {
      dirs.push((project.id().clone(), gest_dir));
    }
  }
  Ok(dirs)
}
