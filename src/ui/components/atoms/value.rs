//! Styled text value for terminal output.

use std::fmt::{self, Display, Formatter};

use yansi::{Paint, Style};

/// A styled text value, typically paired with a [`Label`](super::Label) in detail views.
pub struct Component {
  style: Style,
  text: String,
}

impl Component {
  /// Create a value with the given display text and style.
  pub fn new(text: impl Into<String>, style: Style) -> Self {
    Self {
      style,
      text: text.into(),
    }
  }
}

impl Display for Component {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self.text.paint(self.style))
  }
}
