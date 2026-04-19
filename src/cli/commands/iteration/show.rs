use clap::Args;

use crate::{
  AppContext,
  actions::{Iteration, Prefixable},
  cli::Error,
  store::{model::primitives::EntityType, repo},
  ui::{
    components::{IterationDetail, TaskCounts},
    envelope::Envelope,
    json,
  },
};

/// Show an iteration by ID or prefix.
#[derive(Args, Debug)]
pub struct Command {
  /// The iteration ID or prefix.
  id: String,
  #[command(flatten)]
  output: json::Flags,
}

impl Command {
  /// Resolve the iteration and render its details with phase and task status counts.
  pub async fn call(&self, context: &AppContext) -> Result<(), Error> {
    log::debug!("iteration show: entry");
    let project_id = context.project_id().as_ref().ok_or(Error::UninitializedProject)?;
    let conn = context.store().connect().await?;

    let id = repo::resolve::resolve_id(&conn, repo::resolve::Table::Iterations, &self.id).await?;
    let iteration = repo::iteration::find_required_by_id(&conn, id.clone()).await?;

    let short_id = iteration.id().short();
    if self.output.json || self.output.quiet {
      let envelope = Envelope::load_one(&conn, EntityType::Iteration, &id, &iteration, true).await?;
      self.output.print_envelope(&envelope, &short_id, String::new)?;
      return Ok(());
    }

    let phase_count = repo::iteration::max_phase(&conn, iteration.id())
      .await?
      .map(|m| m as usize + 1)
      .unwrap_or(0);
    let status_counts = repo::iteration::task_status_counts(&conn, iteration.id()).await?;

    let counts = TaskCounts {
      blocked: status_counts.cancelled as usize,
      done: status_counts.done as usize,
      in_progress: status_counts.in_progress as usize,
      open: status_counts.open as usize,
      total: status_counts.total as usize,
    };

    let prefix_len = Iteration::prefix_length(&conn, project_id, &iteration.id().to_string()).await?;

    let view = IterationDetail::new(
      iteration.id().short(),
      iteration.title().to_string(),
      phase_count,
      counts,
    )
    .prefix_len(prefix_len);

    crate::io::pager::page(&format!("{view}"), context)?;
    Ok(())
  }
}
