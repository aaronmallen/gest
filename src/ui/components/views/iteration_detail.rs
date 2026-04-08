//! Single-iteration detail view composing title, phase count, and task status breakdown.

use std::fmt::{self, Display, Formatter};

use yansi::Paint;

use super::super::atoms::{Id, Label, Value};
use crate::ui::style;

/// Fixed padding width for field labels.
const LABEL_PAD: usize = 8;

/// Renders the full detail view for a single iteration, including title, phase count, and task breakdown.
pub struct Component {
  counts: TaskCounts,
  id: String,
  phase_count: usize,
  prefix_len: usize,
  title: String,
}

impl Component {
  /// Create a detail view for the iteration with its title, phase count, and task status totals.
  pub fn new(id: impl Into<String>, title: impl Into<String>, phase_count: usize, counts: TaskCounts) -> Self {
    Self {
      id: id.into(),
      title: title.into(),
      phase_count,
      prefix_len: 2,
      counts,
    }
  }

  /// Sets the highlighted prefix length passed to the rendered ID.
  pub fn prefix_len(mut self, len: usize) -> Self {
    self.prefix_len = len;
    self
  }

  fn task_counts_line(&self) -> String {
    let theme = style::global();
    let sep = format!("{}", " \u{00b7} ".paint(*theme.muted()));
    let total = format!(
      "{}",
      self.counts.total.to_string().paint(*theme.iteration_detail_value()),
    );
    let done = format!(
      "{} {}",
      self.counts.done.to_string().paint(*theme.iteration_detail_count_done()),
      "done".paint(*theme.iteration_detail_count_done()),
    );
    let in_progress = format!(
      "{} {}",
      self
        .counts
        .in_progress
        .to_string()
        .paint(*theme.iteration_detail_count_in_progress()),
      "in progress".paint(*theme.iteration_detail_count_in_progress()),
    );
    let open = format!(
      "{} {}",
      self.counts.open.to_string().paint(*theme.iteration_detail_count_open()),
      "open".paint(*theme.iteration_detail_count_open()),
    );
    let blocked = format!(
      "{} {}",
      self
        .counts
        .blocked
        .to_string()
        .paint(*theme.iteration_detail_count_blocked()),
      "blocked".paint(*theme.iteration_detail_count_blocked()),
    );

    format!("{total}{sep}{done}{sep}{in_progress}{sep}{open}{sep}{blocked}")
  }
}

impl Display for Component {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    let theme = style::global();

    let id = Id::new(&self.id).prefix_len(self.prefix_len);

    let title_label = Label::new("title", *theme.iteration_detail_label()).pad_to(LABEL_PAD);
    let title_value = Value::new(&self.title, *theme.iteration_detail_value());

    let phases_label = Label::new("phases", *theme.iteration_detail_label()).pad_to(LABEL_PAD);
    let phases_value = Value::new(self.phase_count.to_string(), *theme.iteration_detail_value());

    let tasks_label = Label::new("tasks", *theme.iteration_detail_label()).pad_to(LABEL_PAD);
    let tasks_value = self.task_counts_line();

    writeln!(f, "{id}")?;
    writeln!(f)?;
    writeln!(f, "  {title_label}{title_value}")?;
    writeln!(f, "  {phases_label}{phases_value}")?;
    write!(f, "  {tasks_label}{tasks_value}")
  }
}

/// Aggregated task status counts for an iteration summary.
pub struct TaskCounts {
  /// Number of tasks currently blocked by another task.
  pub blocked: usize,
  /// Number of tasks marked as done.
  pub done: usize,
  /// Number of tasks currently in progress.
  pub in_progress: usize,
  /// Number of tasks still open and not yet started.
  pub open: usize,
  /// Total number of tasks across all statuses.
  pub total: usize,
}

#[cfg(test)]
mod tests {
  use super::*;

  fn render(detail: &Component) -> String {
    yansi::disable();
    let out = detail.to_string();
    yansi::enable();
    out
  }

  fn sample_counts() -> TaskCounts {
    TaskCounts {
      total: 7,
      done: 2,
      in_progress: 1,
      open: 3,
      blocked: 1,
    }
  }

  mod display {
    use super::*;

    #[test]
    fn it_renders_blank_line_after_id() {
      let detail = Component::new("q1ebvmxp", "Q1 LLM Benchmark Evaluation", 3, sample_counts());
      let output = render(&detail);
      let lines: Vec<&str> = output.lines().collect();

      assert_eq!(lines[1], "", "second line should be blank");
    }

    #[test]
    fn it_renders_five_lines_of_output() {
      let detail = Component::new("q1ebvmxp", "Q1 LLM Benchmark Evaluation", 3, sample_counts());
      let output = render(&detail);
      let line_count = output.lines().count();

      assert_eq!(
        line_count, 5,
        "should render exactly 5 lines (id, blank, title, phases, tasks)"
      );
    }

    #[test]
    fn it_renders_id_on_first_line() {
      let detail = Component::new("q1ebvmxp", "Q1 LLM Benchmark Evaluation", 3, sample_counts());
      let output = render(&detail);
      let first_line = output.lines().next().unwrap();

      assert!(first_line.contains("q1ebvmxp"), "first line should contain the id");
    }

    #[test]
    fn it_renders_phases_field() {
      let detail = Component::new("q1ebvmxp", "Q1 LLM Benchmark Evaluation", 3, sample_counts());
      let output = render(&detail);

      assert!(output.contains("phases"), "should contain phases label");
      assert!(output.contains('3'), "should contain phase count");
    }

    #[test]
    fn it_renders_task_counts_separated_by_dot() {
      let detail = Component::new("q1ebvmxp", "Q1 LLM Benchmark Evaluation", 3, sample_counts());
      let output = render(&detail);
      let task_line = output.lines().last().unwrap();

      assert!(
        task_line.contains(" \u{00b7} "),
        "task counts should be separated by ' \u{00b7} '"
      );
    }

    #[test]
    fn it_renders_tasks_field_with_counts() {
      let detail = Component::new("q1ebvmxp", "Q1 LLM Benchmark Evaluation", 3, sample_counts());
      let output = render(&detail);

      assert!(output.contains("tasks"), "should contain tasks label");
      assert!(output.contains("7"), "should contain total count");
      assert!(output.contains("2 done"), "should contain done count");
      assert!(output.contains("1 in progress"), "should contain in_progress count");
      assert!(output.contains("3 open"), "should contain open count");
      assert!(output.contains("1 blocked"), "should contain blocked count");
    }

    #[test]
    fn it_renders_title_field() {
      let detail = Component::new("q1ebvmxp", "Q1 LLM Benchmark Evaluation", 3, sample_counts());
      let output = render(&detail);

      assert!(output.contains("title"), "should contain title label");
      assert!(
        output.contains("Q1 LLM Benchmark Evaluation"),
        "should contain title value"
      );
    }

    #[test]
    fn it_renders_zero_counts() {
      let counts = TaskCounts {
        total: 0,
        done: 0,
        in_progress: 0,
        open: 0,
        blocked: 0,
      };
      let detail = Component::new("zerotest", "Empty Iteration", 0, counts);
      let output = render(&detail);

      assert!(output.contains("0 done"));
      assert!(output.contains("0 in progress"));
      assert!(output.contains("0 open"));
      assert!(output.contains("0 blocked"));
    }
  }
}
