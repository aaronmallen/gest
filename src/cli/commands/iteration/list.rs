use clap::Args;

use crate::{
  AppContext,
  cli::Error,
  store::{
    model::{iteration::Filter, primitives::IterationStatus},
    repo,
  },
  ui::{
    components::{IterationEntry, IterationListView},
    json,
  },
};

/// List iterations in the current project.
#[derive(Args, Debug)]
pub struct Command {
  /// Show all iterations, including completed.
  #[arg(long, short)]
  all: bool,
  /// Only show iterations that have unclaimed (open) tasks.
  #[arg(long)]
  has_available: bool,
  /// Filter by status.
  #[arg(long, short)]
  status: Option<IterationStatus>,
  /// Filter by tag.
  #[arg(long, short)]
  tag: Option<String>,
  #[command(flatten)]
  output: json::Flags,
}

impl Command {
  pub async fn call(&self, context: &AppContext) -> Result<(), Error> {
    let project_id = context.project_id().as_ref().ok_or(Error::UninitializedProject)?;
    let conn = context.store().connect().await?;

    let filter = Filter {
      all: self.all,
      has_available: self.has_available,
      status: self.status,
      tag: self.tag.clone(),
    };

    let iterations = repo::iteration::all(&conn, project_id, &filter).await?;

    let id_shorts: Vec<String> = iterations.iter().map(|i| i.id().short().to_string()).collect();

    if self.output.json {
      let json = serde_json::to_string_pretty(&iterations)?;
      println!("{json}");
      return Ok(());
    }

    if self.output.quiet {
      for id in &id_shorts {
        println!("{id}");
      }
      return Ok(());
    }

    // Use the active pool by default, the all pool when --all is set or any
    // filter that may surface terminal iterations is in effect, so prefix
    // highlighting matches the resolver's pool selection.
    let use_all_pool = self.all
      || matches!(
        self.status,
        Some(IterationStatus::Completed) | Some(IterationStatus::Cancelled)
      );
    let prefix_len = if use_all_pool {
      repo::iteration::shortest_all_prefix(&conn, project_id).await?
    } else {
      repo::iteration::shortest_active_prefix(&conn, project_id).await?
    };

    let mut entries = Vec::new();
    for (iteration, id_short) in iterations.iter().zip(id_shorts.iter()) {
      let phase_count = repo::iteration::max_phase(&conn, iteration.id())
        .await?
        .map(|m| m as usize + 1)
        .unwrap_or(0);
      let status_counts = repo::iteration::task_status_counts(&conn, iteration.id()).await?;
      let summary = format!(
        "{} {} · {} tasks",
        phase_count,
        if phase_count == 1 { "phase" } else { "phases" },
        status_counts.total,
      );
      entries.push(IterationEntry {
        id: id_short.clone(),
        summary,
        title: iteration.title().to_string(),
      });
    }

    crate::io::pager::page(&format!("{}\n", IterationListView::new(entries, prefix_len)), context)?;

    Ok(())
  }
}
