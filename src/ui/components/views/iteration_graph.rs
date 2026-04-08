//! Phased dependency graph with box-drawing connectors.

use std::{
  collections::BTreeMap,
  fmt::{self, Display, Formatter},
};

use yansi::Paint;

use crate::ui::components::atoms::Id;

/// A task entry for the graph.
pub struct GraphTask {
  /// Short ID used to render the task's highlighted two-tone identifier.
  pub id_short: String,
  /// Phase number used to group tasks under `── phase N ──` headers.
  pub phase: u32,
  /// Task status, used to select the row icon and color.
  pub status: String,
  /// Task title rendered after the ID.
  pub title: String,
}

/// Phased dependency graph with box-drawing connectors.
pub struct Component {
  iteration_title: String,
  prefix_len: usize,
  tasks: Vec<GraphTask>,
}

impl Component {
  /// Create a graph view for the given iteration title and task list.
  pub fn new(iteration_title: impl Into<String>, tasks: Vec<GraphTask>) -> Self {
    Self {
      iteration_title: iteration_title.into(),
      prefix_len: 2,
      tasks,
    }
  }

  /// Sets the highlighted prefix length passed to rendered task IDs.
  pub fn prefix_len(mut self, len: usize) -> Self {
    self.prefix_len = len;
    self
  }
}

impl Display for Component {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    let theme = crate::ui::style::global();

    writeln!(f)?;
    writeln!(f, "  {}", self.iteration_title.paint(*theme.iteration_graph_title()))?;
    writeln!(f)?;

    if self.tasks.is_empty() {
      writeln!(f, "  no tasks in this iteration")?;
      return Ok(());
    }

    let mut phases: BTreeMap<u32, Vec<&GraphTask>> = BTreeMap::new();
    for task in &self.tasks {
      phases.entry(task.phase).or_default().push(task);
    }

    let phase_count = phases.len();
    for (i, (phase, phase_tasks)) in phases.iter().enumerate() {
      let is_last = i == phase_count - 1;
      let connector = if i == 0 { "╭" } else { "├" };
      writeln!(
        f,
        "  {} ── phase {} ──",
        connector.paint(*theme.iteration_graph_branch()),
        phase
      )?;

      for task in phase_tasks {
        let icon = match task.status.as_str() {
          "done" => "●".paint(*theme.status_done()),
          "in-progress" => "◐".paint(*theme.status_in_progress()),
          "cancelled" => "⊘".paint(*theme.status_cancelled()),
          _ => "●".paint(*theme.status_open()),
        };
        let id = Id::new(&task.id_short).prefix_len(self.prefix_len);
        writeln!(
          f,
          "  {}   {} {} {}",
          "│".paint(*theme.iteration_graph_separator()),
          icon,
          id,
          task.title,
        )?;
      }

      if is_last {
        writeln!(f, "  {}", "╰".paint(*theme.iteration_graph_branch()))?;
      }
    }

    Ok(())
  }
}
