use std::fmt::{self, Display, Formatter};

use yansi::Paint;

use crate::ui::{
  atoms::{id::Id, label::Label, value::Value},
  theme::Theme,
};

/// Fixed padding width for field labels.
const LABEL_PAD: usize = 8;

/// Renders the full detail view for a single iteration, including title, phase count, and task breakdown.
pub struct IterationDetail<'a> {
  counts: TaskCounts,
  id: &'a str,
  phase_count: usize,
  theme: &'a Theme,
  title: &'a str,
}

impl<'a> IterationDetail<'a> {
  pub fn new(id: &'a str, title: &'a str, phase_count: usize, counts: TaskCounts, theme: &'a Theme) -> Self {
    Self {
      id,
      title,
      phase_count,
      counts,
      theme,
    }
  }

  fn task_counts_line(&self) -> String {
    let sep = format!("{}", " · ".paint(self.theme.muted));
    let total = format!(
      "{}",
      self.counts.total.to_string().paint(self.theme.iteration_detail_value),
    );
    let done = format!(
      "{} {}",
      self
        .counts
        .done
        .to_string()
        .paint(self.theme.iteration_detail_count_done),
      "done".paint(self.theme.iteration_detail_count_done),
    );
    let in_progress = format!(
      "{} {}",
      self
        .counts
        .in_progress
        .to_string()
        .paint(self.theme.iteration_detail_count_in_progress),
      "in progress".paint(self.theme.iteration_detail_count_in_progress),
    );
    let open = format!(
      "{} {}",
      self
        .counts
        .open
        .to_string()
        .paint(self.theme.iteration_detail_count_open),
      "open".paint(self.theme.iteration_detail_count_open),
    );
    let blocked = format!(
      "{} {}",
      self
        .counts
        .blocked
        .to_string()
        .paint(self.theme.iteration_detail_count_blocked),
      "blocked".paint(self.theme.iteration_detail_count_blocked),
    );

    format!("{total}{sep}{done}{sep}{in_progress}{sep}{open}{sep}{blocked}")
  }
}

impl Display for IterationDetail<'_> {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    let id = Id::new(self.id, self.theme);

    let title_label = Label::new("title", self.theme.iteration_detail_label).pad_to(LABEL_PAD);
    let title_value = Value::new(self.title, self.theme.iteration_detail_value);

    let phases_label = Label::new("phases", self.theme.iteration_detail_label).pad_to(LABEL_PAD);
    let phases_value = Value::new(self.phase_count.to_string(), self.theme.iteration_detail_value);

    let tasks_label = Label::new("tasks", self.theme.iteration_detail_label).pad_to(LABEL_PAD);
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
  pub blocked: usize,
  pub done: usize,
  pub in_progress: usize,
  pub open: usize,
  pub total: usize,
}

#[cfg(test)]
mod tests {
  use super::*;

  fn theme() -> Theme {
    Theme::default()
  }

  fn render(detail: &IterationDetail) -> String {
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
      let theme = theme();
      let detail = IterationDetail::new("q1ebvmxp", "Q1 LLM Benchmark Evaluation", 3, sample_counts(), &theme);
      let output = render(&detail);
      let lines: Vec<&str> = output.lines().collect();

      assert_eq!(lines[1], "", "second line should be blank");
    }

    #[test]
    fn it_renders_five_lines_of_output() {
      let theme = theme();
      let detail = IterationDetail::new("q1ebvmxp", "Q1 LLM Benchmark Evaluation", 3, sample_counts(), &theme);
      let output = render(&detail);
      let line_count = output.lines().count();

      assert_eq!(
        line_count, 5,
        "should render exactly 5 lines (id, blank, title, phases, tasks)"
      );
    }

    #[test]
    fn it_renders_id_on_first_line() {
      let theme = theme();
      let detail = IterationDetail::new("q1ebvmxp", "Q1 LLM Benchmark Evaluation", 3, sample_counts(), &theme);
      let output = render(&detail);
      let first_line = output.lines().next().unwrap();

      assert!(first_line.contains("q1ebvmxp"), "first line should contain the id");
    }

    #[test]
    fn it_renders_phases_field() {
      let theme = theme();
      let detail = IterationDetail::new("q1ebvmxp", "Q1 LLM Benchmark Evaluation", 3, sample_counts(), &theme);
      let output = render(&detail);

      assert!(output.contains("phases"), "should contain phases label");
      assert!(output.contains('3'), "should contain phase count");
    }

    #[test]
    fn it_renders_task_counts_separated_by_dot() {
      let theme = theme();
      let detail = IterationDetail::new("q1ebvmxp", "Q1 LLM Benchmark Evaluation", 3, sample_counts(), &theme);
      let output = render(&detail);
      let task_line = output.lines().last().unwrap();

      assert!(task_line.contains(" · "), "task counts should be separated by ' · '");
    }

    #[test]
    fn it_renders_tasks_field_with_counts() {
      let theme = theme();
      let detail = IterationDetail::new("q1ebvmxp", "Q1 LLM Benchmark Evaluation", 3, sample_counts(), &theme);
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
      let theme = theme();
      let detail = IterationDetail::new("q1ebvmxp", "Q1 LLM Benchmark Evaluation", 3, sample_counts(), &theme);
      let output = render(&detail);

      assert!(output.contains("title"), "should contain title label");
      assert!(
        output.contains("Q1 LLM Benchmark Evaluation"),
        "should contain title value"
      );
    }

    #[test]
    fn it_renders_zero_counts() {
      let theme = theme();
      let counts = TaskCounts {
        total: 0,
        done: 0,
        in_progress: 0,
        open: 0,
        blocked: 0,
      };
      let detail = IterationDetail::new("zerotest", "Empty Iteration", 0, counts, &theme);
      let output = render(&detail);

      assert!(output.contains("0 done"));
      assert!(output.contains("0 in progress"));
      assert!(output.contains("0 open"));
      assert!(output.contains("0 blocked"));
    }
  }
}
