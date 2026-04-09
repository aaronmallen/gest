use std::cmp::Ordering;

use clap::Args;
use serde_json::json;

use crate::{
  AppContext,
  cli::Error,
  store::{
    model::primitives::{AuthorType, EntityType, IterationStatus, RelationshipType, TaskStatus},
    repo,
  },
  ui::components::FieldList,
};

/// Return the next available task in an iteration.
#[derive(Args, Debug)]
pub struct Command {
  /// The iteration ID or prefix.
  id: String,
  /// Agent name for assignment (requires --claim).
  #[arg(long)]
  agent: Option<String>,
  /// Claim the task (set to in-progress).
  #[arg(long)]
  claim: bool,
  /// Output as JSON.
  #[arg(short, long)]
  json: bool,
  /// Print only the task ID.
  #[arg(short, long)]
  quiet: bool,
}

impl Command {
  /// Pick the next unblocked open task in phase and priority order, optionally claiming and assigning it.
  pub async fn call(&self, context: &AppContext) -> Result<(), Error> {
    log::debug!("iteration next: entry");
    if self.agent.is_some() && !self.claim {
      eprintln!("--agent requires --claim");
      std::process::exit(1);
    }

    let conn = context.store().connect().await?;

    let id = repo::resolve::resolve_id(&conn, "iterations", &self.id).await?;
    let iteration = repo::iteration::find_required_by_id(&conn, id.clone()).await?;

    if iteration.status() != IterationStatus::Active {
      eprintln!("iteration is not active");
      std::process::exit(1);
    }

    let rows = repo::iteration::tasks_with_phase(&conn, &id).await?;

    // Filter to open tasks only
    let open_rows: Vec<_> = rows.iter().filter(|r| r.status == "open").collect();

    // Check which open tasks are blocked
    struct Candidate {
      full_id: String,
      phase: u32,
      priority: Option<u8>,
    }

    let mut candidates = Vec::new();
    for row in &open_rows {
      // Resolve full task ID from short prefix
      let full_id = repo::resolve::resolve_id(&conn, "tasks", &row.id_short).await?;
      let task = repo::task::find_required_by_id(&conn, full_id.clone()).await?;

      // Check if task is blocked
      let rels = repo::relationship::for_entity(&conn, EntityType::Task, &full_id).await?;
      let mut blocked = false;
      for rel in &rels {
        // Task is blocked if it's the target of a "blocks" relationship
        // or the source of a "blocked-by" relationship
        let blocker_id = if rel.rel_type() == RelationshipType::Blocks && rel.target_id() == &full_id {
          Some(rel.source_id())
        } else if rel.rel_type() == RelationshipType::BlockedBy && rel.source_id() == &full_id {
          Some(rel.target_id())
        } else {
          None
        };

        if let Some(blocker_id) = blocker_id
          && let Some(blocker) = repo::task::find_by_id(&conn, blocker_id.clone()).await?
          && !blocker.status().is_terminal()
        {
          blocked = true;
          break;
        }
      }

      if !blocked {
        candidates.push(Candidate {
          full_id: full_id.to_string(),
          phase: row.phase,
          priority: task.priority(),
        });
      }
    }

    // Sort by phase ascending, then priority ascending (lower number = higher priority)
    candidates.sort_by(|a, b| {
      a.phase.cmp(&b.phase).then_with(|| match (a.priority, b.priority) {
        (Some(ap), Some(bp)) => ap.cmp(&bp),
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => Ordering::Equal,
      })
    });

    let Some(next) = candidates.first() else {
      eprintln!("no available tasks");
      std::process::exit(2);
    };

    let task_id: crate::store::model::primitives::Id = next
      .full_id
      .parse()
      .map_err(|e: String| Error::Argument(format!("invalid task id: {e}")))?;

    // If --claim, update the task
    let task = if self.claim {
      let project_id = context.project_id().as_ref().ok_or(Error::UninitializedProject)?;

      let before_task = repo::task::find_required_by_id(&conn, task_id.clone()).await?;
      let before = serde_json::to_value(&before_task)?;

      let mut patch = crate::store::model::task::Patch {
        status: Some(TaskStatus::InProgress),
        ..Default::default()
      };

      if let Some(agent_name) = &self.agent {
        let author = repo::author::find_or_create(&conn, agent_name, None, AuthorType::Agent).await?;
        patch.assigned_to = Some(Some(author.id().clone()));
      }

      let tx = repo::transaction::begin(&conn, project_id, "iteration next claim").await?;
      let updated = repo::task::update(&conn, &task_id, &patch).await?;
      repo::transaction::record_semantic_event(
        &conn,
        tx.id(),
        "tasks",
        &task_id.to_string(),
        "modified",
        Some(&before),
        Some("status-change"),
        Some(&before_task.status().to_string()),
        Some(&updated.status().to_string()),
      )
      .await?;
      updated
    } else {
      repo::task::find_required_by_id(&conn, task_id.clone()).await?
    };

    let phase = next.phase;

    if self.json {
      let json_out = json!({
        "assigned_to": task.assigned_to().as_ref().map(|a| a.to_string()),
        "id": task.id().to_string(),
        "phase": phase,
        "priority": task.priority(),
        "status": task.status().to_string(),
        "title": task.title(),
      });
      println!("{}", serde_json::to_string_pretty(&json_out)?);
      return Ok(());
    }

    if self.quiet {
      println!("{}", task.id().short());
      return Ok(());
    }

    let fields = FieldList::new()
      .field("id", task.id().short())
      .field("title", task.title().to_string())
      .field("status", task.status().to_string())
      .field("phase", phase.to_string())
      .field(
        "priority",
        task
          .priority()
          .map(|p| p.to_string())
          .unwrap_or_else(|| "-".to_string()),
      )
      .field(
        "assigned to",
        task
          .assigned_to()
          .as_ref()
          .map(|a| a.short())
          .unwrap_or_else(|| "-".to_string()),
      );

    println!("{fields}");
    Ok(())
  }
}
