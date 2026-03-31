use std::fmt::{self, Display, Formatter};

use crate::ui::{
  composites::{
    grouped_list::GroupedList,
    iteration_detail::{IterationDetail, TaskCounts},
    iteration_graph::{IterationGraph, PhaseData},
    iteration_list_row::IterationListRow,
  },
  theme::Theme,
};

/// Renders the full detail page for a single iteration, including task status breakdown.
pub struct IterationDetailView<'a> {
  /// Aggregated task status counts for the iteration.
  pub counts: TaskCounts,
  pub id: &'a str,
  pub phase_count: usize,
  pub theme: &'a Theme,
  pub title: &'a str,
}

impl Display for IterationDetailView<'_> {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    let detail = IterationDetail::new(
      self.id,
      self.title,
      self.phase_count,
      TaskCounts {
        total: self.counts.total,
        done: self.counts.done,
        in_progress: self.counts.in_progress,
        open: self.counts.open,
        blocked: self.counts.blocked,
      },
      self.theme,
    );

    write!(f, "{detail}")
  }
}

/// Renders a phase-by-phase graph of an iteration's tasks.
pub struct IterationGraphView<'a> {
  graph: IterationGraph<'a>,
}

impl<'a> IterationGraphView<'a> {
  pub fn new(title: &'a str, phases: Vec<PhaseData<'a>>, theme: &'a Theme) -> Self {
    Self {
      graph: IterationGraph {
        title,
        phases,
        theme,
      },
    }
  }
}

impl Display for IterationGraphView<'_> {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self.graph)
  }
}

/// Data for a single row in the iteration list view.
pub struct IterationListData {
  pub id: String,
  pub phase_count: usize,
  pub task_count: usize,
  pub title: String,
}

/// Renders a grouped list of iterations with a count summary.
pub struct IterationListView<'a> {
  iterations: Vec<IterationListData>,
  theme: &'a Theme,
}

impl<'a> IterationListView<'a> {
  pub fn new(iterations: Vec<IterationListData>, theme: &'a Theme) -> Self {
    Self {
      iterations,
      theme,
    }
  }

  fn summary(&self) -> String {
    let total = self.iterations.len();
    let word = if total == 1 { "iteration" } else { "iterations" };
    format!("{total} {word}")
  }
}

impl Display for IterationListView<'_> {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    let rows: Vec<String> = self
      .iterations
      .iter()
      .map(|i| IterationListRow::new(&i.id, &i.title, i.phase_count, i.task_count, self.theme).to_string())
      .collect();

    let list = GroupedList::new("iterations", self.summary(), self.theme).rows(rows);

    write!(f, "{list}")
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::ui::composites::iteration_graph::TaskData;

  fn theme() -> Theme {
    yansi::disable();
    Theme::default()
  }

  #[test]
  fn it_renders_detail_view_all_fields() {
    let t = theme();
    let view = IterationDetailView {
      id: "q1ebvmxp",
      title: "Q1 LLM Benchmark Evaluation",
      phase_count: 3,
      counts: TaskCounts {
        total: 7,
        done: 2,
        in_progress: 1,
        open: 3,
        blocked: 1,
      },
      theme: &t,
    };
    let out = view.to_string();

    assert!(out.contains("q1"), "should contain id prefix");
    assert!(out.contains("ebvmxp"), "should contain id rest");
    assert!(out.contains("Q1 LLM Benchmark Evaluation"), "should contain title");
    assert!(out.contains("phases"), "should contain phases label");
    assert!(out.contains("tasks"), "should contain tasks label");
    assert!(out.contains("2 done"), "should contain done count");
    assert!(out.contains("1 in progress"), "should contain in_progress");
    assert!(out.contains("3 open"), "should contain open count");
    assert!(out.contains("1 blocked"), "should contain blocked count");
  }

  #[test]
  fn it_renders_detail_view_zero_counts() {
    let t = theme();
    let view = IterationDetailView {
      id: "zerotest",
      title: "Empty",
      phase_count: 0,
      counts: TaskCounts {
        total: 0,
        done: 0,
        in_progress: 0,
        open: 0,
        blocked: 0,
      },
      theme: &t,
    };
    let out = view.to_string();

    assert!(out.contains("0 done"));
    assert!(out.contains("0 open"));
  }

  #[test]
  fn it_renders_graph_view_empty_phases() {
    let t = theme();
    let view = IterationGraphView::new("Empty", vec![], &t);
    let out = view.to_string();

    assert!(out.contains("Empty"), "should contain title");
    assert!(out.contains("0 phases"), "should show zero phases");
  }

  #[test]
  fn it_renders_graph_view_title_and_phases() {
    let t = theme();
    let view = IterationGraphView::new(
      "Q1 LLM Benchmark Evaluation",
      vec![
        PhaseData {
          number: 1,
          name: Some("foundation"),
          tasks: vec![TaskData {
            status: "done",
            id: "cdrzjvwk",
            title: "sqlite storage backend",
            priority: Some(0),

            is_blocking: false,
            blocked_by: None,
          }],
        },
        PhaseData {
          number: 2,
          name: Some("delivery"),
          tasks: vec![TaskData {
            status: "open",
            id: "rwlkbpjq",
            title: "CI pipeline integration",
            priority: Some(2),

            is_blocking: false,
            blocked_by: None,
          }],
        },
      ],
      &t,
    );
    let out = view.to_string();

    assert!(out.contains("Q1 LLM Benchmark Evaluation"), "should contain title");
    assert!(out.contains("Phase 1"), "should contain Phase 1");
    assert!(out.contains("Phase 2"), "should contain Phase 2");
    assert!(out.contains("foundation"), "should contain phase name");
    assert!(out.contains("cdrzjvwk"), "should contain task id");
    assert!(out.contains("rwlkbpjq"), "should contain task id");
  }

  #[test]
  fn it_renders_list_view_empty() {
    let t = theme();
    let view = IterationListView::new(vec![], &t);
    let out = view.to_string();

    assert!(out.contains("0 iterations"), "should show zero count");
  }

  #[test]
  fn it_renders_list_view_heading_and_summary() {
    let t = theme();
    let iterations = vec![
      IterationListData {
        id: "q1ebvmxp".into(),
        title: "Q1 LLM Benchmark".into(),
        phase_count: 3,
        task_count: 7,
      },
      IterationListData {
        id: "r2fcwndy".into(),
        title: "Q2 Plugin System".into(),
        phase_count: 2,
        task_count: 5,
      },
    ];
    let view = IterationListView::new(iterations, &t);
    let out = view.to_string();

    assert!(out.contains("iterations"), "should contain heading");
    assert!(out.contains("2 iterations"), "should contain total count");
    assert!(out.contains("q1ebvmxp"), "should contain first id");
    assert!(out.contains("r2fcwndy"), "should contain second id");
  }

  #[test]
  fn it_renders_list_view_phase_and_task_counts_in_rows() {
    let t = theme();
    let iterations = vec![IterationListData {
      id: "q1ebvmxp".into(),
      title: "Q1 LLM Benchmark".into(),
      phase_count: 3,
      task_count: 7,
    }];
    let view = IterationListView::new(iterations, &t);
    let out = view.to_string();

    assert!(out.contains("3 phases"), "should show phase count in row");
    assert!(out.contains("7 tasks"), "should show task count in row");
  }

  #[test]
  fn it_renders_list_view_singular_count() {
    let t = theme();
    let iterations = vec![IterationListData {
      id: "abcd1234".into(),
      title: "Solo".into(),
      phase_count: 1,
      task_count: 1,
    }];
    let view = IterationListView::new(iterations, &t);
    let out = view.to_string();

    assert!(out.contains("1 iteration"), "should use singular");
    assert!(!out.contains("1 iterations"), "should not use plural for 1");
  }
}
