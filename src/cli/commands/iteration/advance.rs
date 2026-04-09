use clap::Args;

use crate::{
  AppContext,
  cli::Error,
  store::{model::primitives::TaskStatus, repo},
  ui::{components::SuccessMessage, json},
};

/// Validate the active phase is complete and advance to the next phase.
#[derive(Args, Debug)]
pub struct Command {
  /// The iteration ID or prefix.
  id: String,
  /// Advance even if the current phase has non-terminal tasks.
  #[arg(long)]
  force: bool,
  #[command(flatten)]
  output: json::Flags,
}

impl Command {
  /// Check that the active phase is terminal and report the next phase to work on.
  pub async fn call(&self, context: &AppContext) -> Result<(), Error> {
    log::debug!("iteration advance: entry");
    let project_id = context.project_id().as_ref().ok_or(Error::UninitializedProject)?;
    let conn = context.store().connect().await?;
    let id = repo::resolve::resolve_id(&conn, "iterations", &self.id).await?;
    let iteration = repo::iteration::find_required_by_id(&conn, id.clone()).await?;

    if iteration.status().is_terminal() {
      return Err(Error::Editor(format!(
        "iteration {} is {}, not active",
        id.short(),
        iteration.status()
      )));
    }

    let tasks = repo::iteration::tasks_with_phase(&conn, &id).await?;
    if tasks.is_empty() {
      return Err(Error::Editor(format!("iteration {} has no tasks", id.short())));
    }

    // Active phase = lowest phase number where any task has a non-terminal status
    let active_phase = tasks
      .iter()
      .filter(|t| {
        t.status
          .parse::<TaskStatus>()
          .map(|s| !s.is_terminal())
          .unwrap_or(false)
      })
      .map(|t| t.phase)
      .min();

    let max_phase = tasks.iter().map(|t| t.phase).max().unwrap_or(0);
    let prefix_len = repo::iteration::shortest_active_prefix(&conn, project_id).await?;

    match active_phase {
      None => {
        // All tasks in all phases are terminal
        Err(Error::Editor(format!(
          "iteration {} is already complete: all tasks in all phases are terminal",
          id.short()
        )))
      }
      Some(phase) => {
        // Count non-terminal tasks in the active phase
        let non_terminal_count = tasks
          .iter()
          .filter(|t| {
            t.phase == phase
              && t
                .status
                .parse::<TaskStatus>()
                .map(|s| !s.is_terminal())
                .unwrap_or(false)
          })
          .count();

        if non_terminal_count > 0 && !self.force {
          return Err(Error::Editor(format!(
            "phase {phase} has {non_terminal_count} non-terminal task(s); use --force to advance anyway"
          )));
        }

        // Determine the next phase (next higher phase number that exists)
        let next_phase = tasks.iter().map(|t| t.phase).filter(|&p| p > phase).min();

        match next_phase {
          Some(next) => {
            let short_id = id.short();
            let result = serde_json::json!({
              "id": id.to_string(),
              "from_phase": phase,
              "to_phase": next,
            });
            self.output.print_entity(&result, &short_id, || {
              SuccessMessage::new("advanced iteration")
                .id(id.short())
                .prefix_len(prefix_len)
                .field("title", iteration.title().to_string())
                .field("from phase", phase.to_string())
                .field("to phase", next.to_string())
                .to_string()
            })?;
          }
          None => {
            if phase == max_phase {
              return Err(Error::Editor(format!(
                "iteration {} is on the last phase ({phase}) with non-terminal tasks; \
                complete remaining tasks to finish the iteration",
                id.short()
              )));
            }
            let short_id = id.short();
            let result = serde_json::json!({
              "id": id.to_string(),
              "status": "all_phases_complete",
            });
            self.output.print_entity(&result, &short_id, || {
              SuccessMessage::new("all phases complete")
                .id(id.short())
                .prefix_len(prefix_len)
                .field("title", iteration.title().to_string())
                .to_string()
            })?;
          }
        }

        Ok(())
      }
    }
  }
}
