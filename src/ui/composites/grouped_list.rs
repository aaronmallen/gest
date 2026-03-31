use std::fmt::{self, Display, Formatter};

use yansi::Paint;

use crate::ui::theme::Theme;

/// Renders a headed list section with a summary line and optional item rows.
pub struct GroupedList<'a> {
  heading: &'a str,
  rows: Vec<String>,
  summary: String,
  theme: &'a Theme,
}

impl<'a> GroupedList<'a> {
  pub fn new(heading: &'a str, summary: impl Into<String>, theme: &'a Theme) -> Self {
    Self {
      heading,
      summary: summary.into(),
      rows: Vec::new(),
      theme,
    }
  }

  /// Appends multiple display items as rows.
  pub fn rows(mut self, items: impl IntoIterator<Item = impl Display>) -> Self {
    for item in items {
      self.rows.push(item.to_string());
    }
    self
  }
}

impl Display for GroupedList<'_> {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    write!(
      f,
      "{}  {}",
      self.heading.paint(self.theme.list_heading),
      self.summary.paint(self.theme.list_summary),
    )?;

    if !self.rows.is_empty() {
      writeln!(f)?;
      for row in &self.rows {
        write!(f, "\n{row}")?;
      }
    }

    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn theme() -> Theme {
    yansi::disable();
    Theme::default()
  }

  #[test]
  fn it_adds_multiple_rows_via_rows_method() {
    let t = theme();
    let list = GroupedList::new("tasks", "2 tasks", &t).rows(["one", "two"]);
    let output = list.to_string();

    assert!(output.contains("one"));
    assert!(output.contains("two"));
  }

  #[test]
  fn it_omits_row_section_when_empty() {
    let t = theme();
    let list = GroupedList::new("iterations", "1 iteration", &t);
    let output = list.to_string();

    assert_eq!(output, "iterations  1 iteration");
  }

  #[test]
  fn it_renders_heading_and_rows() {
    let t = theme();
    let list = GroupedList::new("artifacts", "3 artifacts", &t).rows(["row-a", "row-b"]);
    let output = list.to_string();

    assert!(output.contains("artifacts"));
    assert!(output.contains("3 artifacts"));
    assert!(output.contains("\n\nrow-a"));
    assert!(output.contains("\nrow-b"));
  }

  #[test]
  fn it_renders_heading_and_summary() {
    let t = theme();
    let list = GroupedList::new("tasks", "7 tasks  \u{00b7}  2 done", &t);
    let output = list.to_string();

    assert!(output.starts_with("tasks"));
    assert!(output.contains("7 tasks"));
    assert!(output.contains("\u{00b7}"));
  }
}
