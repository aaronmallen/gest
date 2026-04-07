use std::fmt::{self, Display, Formatter};

use yansi::{Paint, Style};

use crate::ui::components::atoms::Icon;

/// Renders a status icon + label for a given status string.
pub struct Component<'a> {
  show_text: bool,
  status: &'a str,
}

impl<'a> Component<'a> {
  pub fn new(status: &'a str) -> Self {
    Self {
      show_text: true,
      status,
    }
  }

  /// Hide the text label, showing only the icon.
  pub fn icon_only(mut self) -> Self {
    self.show_text = false;
    self
  }

  fn icon(&self) -> Icon {
    match self.status {
      "blocked" => Icon::blocked(),
      _ => Icon::status(self.status),
    }
  }

  fn label(&self) -> &str {
    match self.status {
      "in-progress" => "in progress",
      other => other,
    }
  }

  fn style(&self) -> Style {
    let theme = crate::ui::style::global();
    match self.status {
      "cancelled" => *theme.status_cancelled(),
      "done" => *theme.status_done(),
      "in-progress" => *theme.status_in_progress(),
      "blocked" => *theme.indicator_blocked(),
      _ => *theme.status_open(),
    }
  }
}

impl Display for Component<'_> {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self.icon())?;
    if self.show_text {
      write!(f, " {}", self.label().paint(self.style()))?;
    }
    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn render(badge: &Component) -> String {
    yansi::disable();
    let out = badge.to_string();
    yansi::enable();
    out
  }

  #[test]
  fn it_renders_blocked_with_blocked_icon() {
    let badge = Component::new("blocked");
    let out = render(&badge);

    assert!(out.contains('\u{2297}'));
    assert!(out.contains("blocked"));
  }

  #[test]
  fn it_renders_icon_only() {
    let badge = Component::new("done").icon_only();
    let out = render(&badge);

    assert!(out.contains('\u{25CF}'));
    assert!(!out.contains("done"));
  }

  #[test]
  fn it_renders_in_progress_with_space() {
    let badge = Component::new("in-progress");
    let out = render(&badge);

    assert!(out.contains("in progress"));
  }

  #[test]
  fn it_renders_open_status() {
    let badge = Component::new("open");
    let out = render(&badge);

    assert!(out.contains('\u{25CB}'));
    assert!(out.contains("open"));
  }
}
