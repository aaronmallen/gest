//! Horizontal rule atom with optional centered label.

use std::fmt::{self, Display, Formatter};

use yansi::{Paint, Style};

use crate::ui::components::molecules::row;

/// A horizontal separator line with optional centered label.
pub struct Component {
  ch: char,
  label: Option<String>,
  style: Style,
  width: Option<usize>,
}

impl Component {
  /// Create a solid rule separator.
  pub fn rule(style: Style) -> Self {
    Self {
      ch: '─',
      label: None,
      style,
      width: None,
    }
  }

  /// Create a solid separator with a centered label.
  pub fn labeled(label: impl Into<String>, style: Style) -> Self {
    Self {
      ch: '─',
      label: Some(label.into()),
      style,
      width: None,
    }
  }

  /// Create a dashed separator with a centered label.
  pub fn dashed(label: impl Into<String>, style: Style) -> Self {
    Self {
      ch: '╌',
      label: Some(label.into()),
      style,
      width: None,
    }
  }

  /// Override the separator width (defaults to terminal width minus indent).
  pub fn width(mut self, width: usize) -> Self {
    self.width = Some(width);
    self
  }
}

impl Display for Component {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    let width = self.width.unwrap_or_else(|| row::terminal_width() as usize);
    let ch = self.ch.to_string();

    match &self.label {
      Some(label) => {
        let label_len = label.len() + 2; // " label "
        let left = (width.saturating_sub(label_len)) / 2;
        let right = width.saturating_sub(label_len + left);
        let left_line: String = ch.repeat(left);
        let right_line: String = ch.repeat(right);
        write!(
          f,
          "{} {} {}",
          left_line.paint(self.style),
          label.paint(self.style),
          right_line.paint(self.style),
        )
      }
      None => {
        let line: String = ch.repeat(width);
        write!(f, "{}", line.paint(self.style))
      }
    }
  }
}
