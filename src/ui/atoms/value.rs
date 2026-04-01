use std::fmt::{self, Display, Formatter};

use yansi::{Paint, Style};

/// A styled text value, typically paired with a [`super::label::Label`] in detail views.
pub struct Value {
  style: Style,
  text: String,
}

impl Value {
  /// Create a value with the given display text and style.
  pub fn new(text: impl Into<String>, style: Style) -> Self {
    Self {
      text: text.into(),
      style,
    }
  }
}

impl Display for Value {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self.text.paint(self.style))
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  mod display {
    use super::*;

    #[test]
    fn it_renders_styled_text() {
      let value = Value::new("in-progress", Style::new().bold());
      let rendered = format!("{value}");
      assert!(rendered.contains("in-progress"));
    }
  }

  mod new {
    use super::*;

    #[test]
    fn it_accepts_string_and_str() {
      let _from_str = Value::new("ok", Style::default());
      let _from_string = Value::new(String::from("ok"), Style::default());
    }
  }
}
