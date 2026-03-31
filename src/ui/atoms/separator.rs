use std::fmt::{self, Display, Formatter};

use yansi::{Paint, Style};

use crate::ui::utils;

/// A full-width horizontal rule, optionally carrying a centered label.
pub struct Separator {
  ch: char,
  label: Option<String>,
  style: Style,
  width: Option<usize>,
}

impl Separator {
  /// Create a dashed separator (`╌`) with a label.
  pub fn dashed(label: impl Into<String>, style: Style) -> Self {
    Self {
      ch: '╌',
      style,
      label: Some(label.into()),
      width: None,
    }
  }

  /// Create a solid separator (`─`) with a label.
  pub fn labeled(label: impl Into<String>, style: Style) -> Self {
    Self {
      ch: '─',
      style,
      label: Some(label.into()),
      width: None,
    }
  }

  /// Create a plain solid rule with no label.
  pub fn rule(style: Style) -> Self {
    Self {
      ch: '─',
      style,
      label: None,
      width: None,
    }
  }

  /// Override the line width (defaults to the current terminal width).
  pub fn width(mut self, width: usize) -> Self {
    self.width = Some(width);
    self
  }
}

impl Display for Separator {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    let total_width = self.width.unwrap_or(utils::terminal_width() as usize);
    let ch_str = self.ch.to_string();

    match &self.label {
      Some(label) => {
        let prefix = format!("{ch_str}{ch_str} {label} ");
        let prefix_width = utils::display_width(&prefix);
        let remaining = total_width.saturating_sub(prefix_width);
        let suffix: String = std::iter::repeat_n(self.ch, remaining).collect();
        write!(f, "{}", format!("{prefix}{suffix}").paint(self.style))
      }
      None => {
        let line: String = std::iter::repeat_n(self.ch, total_width).collect();
        write!(f, "{}", line.paint(self.style))
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn render(sep: Separator) -> String {
    format!("{sep}")
  }

  mod dashed {
    use super::*;

    #[test]
    fn it_uses_dashed_char() {
      let s = Separator::dashed("task id", Style::default()).width(25);
      let out = render(s);

      assert!(out.starts_with("╌╌ task id "));
      assert!(out.contains('╌'));
      assert!(!out.contains('─'));
    }
  }

  mod labeled {
    use super::*;

    #[test]
    fn it_fills_remaining_width() {
      let s = Separator::labeled("hi", Style::default()).width(20);
      let out = render(s);

      assert_eq!(utils::display_width(&out), 20);
    }

    #[test]
    fn it_includes_label_text() {
      let s = Separator::labeled("description", Style::default()).width(30);
      let out = render(s);

      assert!(out.starts_with("── description "));
      assert_eq!(utils::display_width(&out), 30);
    }

    #[test]
    fn it_prevents_panic_on_tiny_width() {
      let s = Separator::labeled("a long label here", Style::default()).width(5);
      let _out = render(s);
    }
  }

  mod rule {
    use super::*;

    #[test]
    fn it_fills_width() {
      let s = Separator::rule(Style::default()).width(10);
      let out = render(s);

      assert_eq!(out, "──────────");
    }

    #[test]
    fn it_respects_explicit_width() {
      let s = Separator::rule(Style::default()).width(5);
      let out = render(s);

      assert_eq!(out, "─────");
    }
  }
}
