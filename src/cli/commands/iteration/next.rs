use std::{cmp::Ordering, collections::HashMap};

use clap::Args;

use crate::{
  AppContext,
  cli::Error,
  store::{
    model::primitives::{AuthorType, Id, IterationStatus, TaskStatus},
    repo,
  },
  ui::{components::FieldList, json},
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
  #[command(flatten)]
  output: json::Flags,
}

impl Command {
  /// Pick the next unblocked open task in phase and priority order, optionally claiming and assigning it.
  pub async fn call(&self, context: &AppContext) -> Result<(), Error> {
    log::debug!("iteration next: entry");
    if self.agent.is_some() && !self.claim {
      return Err(Error::Argument("--agent requires --claim".into()));
    }

    let conn = context.store().connect().await?;

    let id = repo::resolve::resolve_id(&conn, repo::resolve::Table::Iterations, &self.id).await?;
    let iteration = repo::iteration::find_required_by_id(&conn, id.clone()).await?;

    if iteration.status() != IterationStatus::Active {
      return Err(Error::InvalidState("iteration is not active".into()));
    }

    let rows = repo::iteration::tasks_with_phase(&conn, &id).await?;

    // Build a status lookup for all iteration tasks (id_short → status) so we
    // can check whether blockers have reached a terminal state without extra queries.
    let status_map: HashMap<&str, &str> = rows.iter().map(|r| (r.id_short.as_str(), r.status.as_str())).collect();

    // Filter to open, unblocked tasks using the batch-loaded blocking data.
    struct Candidate<'a> {
      full_id: &'a str,
      id_short: &'a str,
      phase: u32,
      priority: Option<u8>,
    }

    let mut candidates = Vec::new();
    for row in &rows {
      if row.status != "open" {
        continue;
      }

      // A task is blocked if any of its blockers are non-terminal.
      let blocked = row.blocked_by.iter().any(|blocker_short| {
        status_map
          .get(blocker_short.as_str())
          .map(|s| {
            // Parse to TaskStatus to check terminality
            s.parse::<TaskStatus>().map(|ts| !ts.is_terminal()).unwrap_or(true)
          })
          // Blocker not in this iteration — conservatively treat as blocking
          .unwrap_or(true)
      });

      if !blocked {
        candidates.push(Candidate {
          full_id: &row.id,
          id_short: &row.id_short,
          phase: row.phase,
          priority: row.priority,
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
      return Err(Error::NoTasksAvailable);
    };

    let task_id: Id = next
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
    let short_id = next.id_short;

    self.output.print_entity(
      &serde_json::json!({
        "assigned_to": task.assigned_to().as_ref().map(|a| a.to_string()),
        "id": task.id().to_string(),
        "phase": phase,
        "priority": task.priority(),
        "status": task.status().to_string(),
        "title": task.title(),
      }),
      short_id,
      || {
        FieldList::new()
          .field("id", short_id.to_string())
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
          )
          .to_string()
      },
    )?;

    Ok(())
  }
}
