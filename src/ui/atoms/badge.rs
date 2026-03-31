use std::fmt::{self, Display, Formatter};

use yansi::{Paint, Style};

/// A short styled text fragment, typically used for inline status indicators.
pub struct Badge {
  style: Style,
  text: String,
}

impl Badge {
  /// Create a badge with the given display text and style.
  pub fn new(text: impl Into<String>, style: Style) -> Self {
    Self {
      text: text.into(),
      style,
    }
  }
}

impl Display for Badge {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self.text.paint(self.style))
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  mod new {
    use super::*;

    #[test]
    fn it_accepts_string_and_str() {
      let _from_str = Badge::new("ok", Style::default());
      let _from_string = Badge::new(String::from("ok"), Style::default());
    }
  }

  mod display {
    use super::*;

    #[test]
    fn it_renders_styled_text() {
      let badge = Badge::new("● done", Style::new().bold());
      let rendered = format!("{badge}");
      assert!(rendered.contains("● done"));
    }
  }
}
