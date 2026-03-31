use std::fmt::{self, Display, Formatter};

use yansi::Paint;

use crate::ui::{
  atoms::{id::Id, title::Title},
  theme::Theme,
};

/// A single row in an iteration list, showing id, title, and phase/task counts.
pub struct IterationListRow<'a> {
  id: &'a str,
  phase_count: usize,
  task_count: usize,
  theme: &'a Theme,
  title_text: &'a str,
}

impl<'a> IterationListRow<'a> {
  pub fn new(id: &'a str, title_text: &'a str, phase_count: usize, task_count: usize, theme: &'a Theme) -> Self {
    Self {
      id,
      title_text,
      phase_count,
      task_count,
      theme,
    }
  }
}

impl Display for IterationListRow<'_> {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    let id = Id::new(self.id, self.theme);
    let title = Title::new(self.title_text, self.theme.iteration_list_title).pad_to(30);
    let summary = format!(
      "{} {} · {} {}",
      self.phase_count,
      if self.phase_count == 1 { "phase" } else { "phases" },
      self.task_count,
      if self.task_count == 1 { "task" } else { "tasks" },
    );

    write!(
      f,
      "{}  {}  {}",
      id,
      title,
      summary.paint(self.theme.iteration_list_summary),
    )
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn theme() -> Theme {
    Theme::default()
  }

  #[test]
  fn it_renders_id() {
    let theme = theme();
    let row = IterationListRow::new("q1ebvmxp", "Q1 LLM Benchmark", 3, 7, &theme);
    let rendered = format!("{row}");

    assert!(rendered.contains("q1"));
    assert!(rendered.contains("ebvmxp"));
  }

  #[test]
  fn it_renders_summary_with_plural_forms() {
    let theme = theme();
    let row = IterationListRow::new("q1ebvmxp", "Q1 LLM Benchmark", 3, 7, &theme);
    let rendered = format!("{row}");

    assert!(rendered.contains("3 phases"));
    assert!(rendered.contains("7 tasks"));
    assert!(rendered.contains("·"));
  }

  #[test]
  fn it_renders_summary_with_singular_forms() {
    let theme = theme();
    let row = IterationListRow::new("abcdefgh", "Solo Run", 1, 1, &theme);
    let rendered = format!("{row}");

    assert!(rendered.contains("1 phase"));
    assert!(rendered.contains("1 task"));
    assert!(!rendered.contains("phases"));
    assert!(!rendered.contains("tasks"));
  }

  #[test]
  fn it_renders_title() {
    let theme = theme();
    let row = IterationListRow::new("q1ebvmxp", "Q1 LLM Benchmark", 3, 7, &theme);
    let rendered = format!("{row}");

    assert!(rendered.contains("Q1 LLM Benchmark"));
  }

  #[test]
  fn it_renders_zero_counts() {
    let theme = theme();
    let row = IterationListRow::new("zerotest", "Empty Iteration", 0, 0, &theme);
    let rendered = format!("{row}");

    assert!(rendered.contains("0 phases"));
    assert!(rendered.contains("0 tasks"));
  }
}
