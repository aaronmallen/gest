//! Multi-line summary molecule for commands that report tallies of work.
//!
//! Composes a titled block of count rows, an optional completion message, and
//! an optional trailing hint. A dedicated empty-state message replaces the row
//! block when no rows are pushed, letting callers hand a single component both
//! the "nothing to do" and "here's what happened" paths.

use std::fmt::{self, Display, Formatter};

use yansi::Paint;

use super::SuccessMessage;

/// Renders a titled summary block with per-row counts, plus optional completion
/// and hint lines beneath it.
///
/// A single [`Component`] handles both outcomes a tally-style command needs to
/// print:
///
/// - When rows are pushed, the title is followed by an indented list of rows.
/// - When no rows are pushed and an `empty_message` is set, the empty message
///   replaces the rows (the title is suppressed).
///
/// The optional [`SuccessMessage`] trailer and muted hint line render after the
/// summary block, separated by blank lines.
pub struct Component {
  empty_message: Option<String>,
  hint: Option<String>,
  rows: Vec<Row>,
  success: Option<SuccessMessage>,
  title: Option<String>,
}

impl Component {
  /// Create an empty summary with no title, rows, message, or hint.
  pub fn new() -> Self {
    Self {
      empty_message: None,
      hint: None,
      rows: Vec::new(),
      success: None,
      title: None,
    }
  }

  /// Set the message rendered in place of the row block when no rows are pushed.
  pub fn empty_message(mut self, message: impl Into<String>) -> Self {
    self.empty_message = Some(message.into());
    self
  }

  /// Append a trailing hint line rendered with the muted theme.
  pub fn hint(mut self, hint: impl Into<String>) -> Self {
    self.hint = Some(hint.into());
    self
  }

  /// Report whether any rows have been pushed.
  #[allow(dead_code)]
  pub fn is_empty(&self) -> bool {
    self.rows.is_empty()
  }

  /// Append a row with a label, count, and no detail text.
  pub fn row(self, label: impl Into<String>, count: usize) -> Self {
    self.row_with_detail(label, count, None::<String>)
  }

  /// Append a row with a label, count, and optional parenthesized detail text.
  pub fn row_with_detail(mut self, label: impl Into<String>, count: usize, detail: Option<impl Into<String>>) -> Self {
    self.rows.push(Row {
      count,
      detail: detail.map(Into::into),
      label: label.into(),
    });
    self
  }

  /// Attach a completion [`SuccessMessage`] rendered beneath the summary block.
  pub fn success(mut self, message: SuccessMessage) -> Self {
    self.success = Some(message);
    self
  }

  /// Set the title rendered above the row block (e.g. `Purge summary:`).
  pub fn title(mut self, title: impl Into<String>) -> Self {
    self.title = Some(title.into());
    self
  }
}

impl Default for Component {
  fn default() -> Self {
    Self::new()
  }
}

/// A single row in a [`Component`] summary.
struct Row {
  count: usize,
  detail: Option<String>,
  label: String,
}

impl Display for Component {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    let theme = crate::ui::style::global();
    let mut wrote = false;

    if self.rows.is_empty() {
      if let Some(message) = &self.empty_message {
        write!(f, "  {}", message.as_str().paint(*theme.muted()))?;
        wrote = true;
      }
    } else {
      if let Some(title) = &self.title {
        write!(f, "{}", title.as_str().paint(*theme.list_heading()))?;
        wrote = true;
      }
      for row in &self.rows {
        if wrote {
          writeln!(f)?;
        }
        write_row(f, row)?;
        wrote = true;
      }
    }

    if let Some(message) = &self.success {
      if wrote {
        write!(f, "\n\n")?;
      }
      write!(f, "{message}")?;
      wrote = true;
    }

    if let Some(hint) = &self.hint {
      if wrote {
        write!(f, "\n\n")?;
      }
      write!(f, "  {}", hint.as_str().paint(*theme.muted()))?;
    }

    Ok(())
  }
}

/// Format a single row as `  <label>: <count>` with an optional ` (<detail>)` suffix.
fn write_row(f: &mut Formatter<'_>, row: &Row) -> fmt::Result {
  let theme = crate::ui::style::global();

  write!(
    f,
    "  {}{} {}",
    row.label.as_str().paint(*theme.muted()),
    ":".paint(*theme.muted()),
    row.count
  )?;

  if let Some(detail) = &row.detail {
    write!(f, " {}", format!("({detail})").paint(*theme.muted()))?;
  }

  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::ui::components::molecules::row::strip_ansi;

  fn render(component: &Component) -> String {
    strip_ansi(&component.to_string())
  }

  mod fmt {
    use super::*;

    #[test]
    fn it_renders_empty_message_with_zero_rows() {
      let component = Component::new()
        .title("Purge summary:")
        .empty_message("Nothing to purge.");

      let output = render(&component);

      assert_eq!(output, "  Nothing to purge.");
      assert!(!output.contains("Purge summary:"));
      assert_eq!(output.lines().count(), 1);
    }

    #[test]
    fn it_renders_nothing_when_nothing_is_configured() {
      let component = Component::new();

      assert_eq!(render(&component), "");
    }

    #[test]
    fn it_renders_title_and_rows_with_details() {
      let component = Component::new()
        .title("Purge summary:")
        .row_with_detail("tasks", 5, Some("3 done, 2 cancelled"))
        .row("artifacts", 2)
        .hint("Run `gest undo` to restore.");

      let output = render(&component);
      let lines: Vec<&str> = output.lines().collect();

      assert_eq!(lines.len(), 5, "title + 2 rows + blank + hint");
      assert_eq!(lines[0], "Purge summary:");
      assert_eq!(lines[1], "  tasks: 5 (3 done, 2 cancelled)");
      assert_eq!(lines[2], "  artifacts: 2");
      assert_eq!(lines[3], "");
      assert_eq!(lines[4], "  Run `gest undo` to restore.");
    }

    #[test]
    fn it_renders_rows_without_title() {
      let component = Component::new().row("tasks", 3);

      let output = render(&component);
      let lines: Vec<&str> = output.lines().collect();

      assert_eq!(lines, vec!["  tasks: 3"]);
    }

    #[test]
    fn it_renders_success_message_after_rows() {
      let component = Component::new()
        .title("Purge summary:")
        .row("tasks", 2)
        .success(SuccessMessage::new("Purged").field("total", "2"));

      let output = render(&component);
      let lines: Vec<&str> = output.lines().collect();

      assert_eq!(lines[0], "Purge summary:");
      assert_eq!(lines[1], "  tasks: 2");
      assert_eq!(lines[2], "");
      assert!(lines[3].contains("Purged"));
      assert!(output.contains("total"));
    }

    #[test]
    fn it_renders_success_message_after_empty_message() {
      let component = Component::new()
        .empty_message("Nothing to purge.")
        .success(SuccessMessage::new("Purged").field("total", "0"));

      let output = render(&component);
      let lines: Vec<&str> = output.lines().collect();

      assert_eq!(lines[0], "  Nothing to purge.");
      assert_eq!(lines[1], "");
      assert!(lines[2].contains("Purged"));
    }

    #[test]
    fn it_separates_hint_from_preceding_content_with_blank_line() {
      let component = Component::new()
        .empty_message("Nothing to purge.")
        .hint("Run `gest undo` to restore.");

      let output = render(&component);
      let lines: Vec<&str> = output.lines().collect();

      assert_eq!(lines.len(), 3);
      assert_eq!(lines[1], "");
    }
  }

  mod is_empty {
    use super::*;

    #[test]
    fn it_returns_false_when_rows_are_present() {
      let component = Component::new().row("tasks", 1);

      assert!(!component.is_empty());
    }

    #[test]
    fn it_returns_true_when_no_rows_are_present() {
      let component = Component::new().title("Purge summary:");

      assert!(component.is_empty());
    }
  }
}
