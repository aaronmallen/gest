use std::fmt::{self, Display, Formatter};

use yansi::{Paint, Style};

use crate::ui::theme::Theme;

/// A themed status badge that renders a status icon with optional label text.
///
/// Supports two display modes:
/// - **Icon + text** (default): renders both the glyph and label (e.g., `● done`)
/// - **Icon only**: renders just the glyph (e.g., `●`)
///
/// Both icon and text are colored with a single style from the theme's `status_*` tokens.
pub struct StatusBadge<'a> {
  show_text: bool,
  status: &'a str,
  theme: &'a Theme,
}

impl<'a> StatusBadge<'a> {
  /// Create a status badge for the given status string.
  ///
  /// Recognized statuses: `"open"`, `"in-progress"`, `"done"`, `"cancelled"`, `"blocked"`.
  /// Unknown statuses fall back to the `open` style.
  pub fn new(status: &'a str, theme: &'a Theme) -> Self {
    Self {
      status,
      theme,
      show_text: true,
    }
  }

  fn glyph(&self) -> char {
    match self.status {
      "open" => '\u{25CB}',
      "in-progress" => '\u{25D0}',
      "done" => '\u{25CF}',
      "cancelled" => '\u{2298}',
      "blocked" => '\u{2297}',
      _ => '\u{25CB}',
    }
  }

  fn label(&self) -> &str {
    match self.status {
      "open" => "open",
      "in-progress" => "in progress",
      "done" => "done",
      "cancelled" => "cancelled",
      "blocked" => "blocked",
      _ => self.status,
    }
  }

  fn style(&self) -> Style {
    match self.status {
      "open" => self.theme.status_open,
      "in-progress" => self.theme.status_in_progress,
      "done" => self.theme.status_done,
      "cancelled" => self.theme.status_cancelled,
      "blocked" => self.theme.indicator_blocked,
      _ => self.theme.status_open,
    }
  }
}

impl Display for StatusBadge<'_> {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    let style = self.style();
    if self.show_text {
      write!(f, "{}", format!("{} {}", self.glyph(), self.label()).paint(style))
    } else {
      write!(f, "{}", self.glyph().paint(style))
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn theme() -> Theme {
    Theme::default()
  }

  fn render(badge: &StatusBadge) -> String {
    yansi::disable();
    let out = badge.to_string();
    yansi::enable();
    out
  }

  mod new {
    use super::*;

    #[test]
    fn it_defaults_to_icon_and_text() {
      let t = theme();
      let badge = StatusBadge::new("done", &t);
      let out = render(&badge);

      assert!(out.contains('\u{25CF}'), "should contain done icon");
      assert!(out.contains("done"), "should contain label text");
    }
  }

  mod display {
    use super::*;

    #[test]
    fn it_renders_open() {
      let t = theme();
      let out = render(&StatusBadge::new("open", &t));

      assert!(out.contains('\u{25CB}'), "should contain open icon");
      assert!(out.contains("open"), "should contain open label");
    }

    #[test]
    fn it_renders_in_progress() {
      let t = theme();
      let out = render(&StatusBadge::new("in-progress", &t));

      assert!(out.contains('\u{25D0}'), "should contain in-progress icon");
      assert!(out.contains("in progress"), "should contain spaced label");
    }

    #[test]
    fn it_renders_done() {
      let t = theme();
      let out = render(&StatusBadge::new("done", &t));

      assert!(out.contains('\u{25CF}'), "should contain done icon");
      assert!(out.contains("done"), "should contain done label");
    }

    #[test]
    fn it_renders_cancelled() {
      let t = theme();
      let out = render(&StatusBadge::new("cancelled", &t));

      assert!(out.contains('\u{2298}'), "should contain cancelled icon");
      assert!(out.contains("cancelled"), "should contain cancelled label");
    }

    #[test]
    fn it_renders_blocked() {
      let t = theme();
      let out = render(&StatusBadge::new("blocked", &t));

      assert!(out.contains('\u{2297}'), "should contain blocked icon");
      assert!(out.contains("blocked"), "should contain blocked label");
    }

    #[test]
    fn it_falls_back_to_open_for_unknown_status() {
      let t = theme();
      let out = render(&StatusBadge::new("mystery", &t));

      assert!(out.contains('\u{25CB}'), "should use open icon for unknown status");
      assert!(out.contains("mystery"), "should use raw status as label");
    }
  }
}
