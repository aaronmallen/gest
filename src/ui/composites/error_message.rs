use std::fmt::{self, Display, Formatter};

use yansi::Paint;

use crate::ui::theme::Theme;

/// Renders an error message prefixed with a cross icon.
pub struct ErrorMessage<'a> {
  message: String,
  theme: &'a Theme,
}

impl<'a> ErrorMessage<'a> {
  pub fn new(message: impl Into<String>, theme: &'a Theme) -> Self {
    Self {
      message: message.into(),
      theme,
    }
  }
}

impl Display for ErrorMessage<'_> {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    write!(
      f,
      "  {}  {}",
      "\u{2717}".paint(self.theme.error),
      self.message.paint(self.theme.error),
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
  fn it_accepts_string_and_str() {
    let theme = theme();
    let _from_str = ErrorMessage::new("oops", &theme);
    let _from_string = ErrorMessage::new(String::from("oops"), &theme);
  }

  #[test]
  fn it_renders_error_with_cross_icon() {
    let theme = theme();
    let msg = ErrorMessage::new("something went wrong", &theme);
    let rendered = format!("{msg}");

    assert!(rendered.contains('\u{2717}'));
    assert!(rendered.contains("something went wrong"));
  }
}
