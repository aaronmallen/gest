use std::fmt::{self, Display, Formatter};

use yansi::{Paint, Style};

use crate::ui::theme::Theme;

/// A single themed Unicode glyph used as a visual indicator.
pub struct Icon {
  ch: char,
  style: Style,
}

impl Icon {
  /// Circled-times icon for blocked items.
  pub fn blocked(theme: &Theme) -> Self {
    Self::new('\u{2297}', theme.indicator_blocked)
  }

  /// Exclamation icon for items that block others.
  pub fn blocking(theme: &Theme) -> Self {
    Self::new('!', theme.indicator_blocking)
  }

  /// Create an icon from an arbitrary character and style.
  pub fn new(ch: char, style: Style) -> Self {
    Self {
      ch,
      style,
    }
  }

  /// Diamond icon for iteration graph phases.
  pub fn phase(theme: &Theme) -> Self {
    Self::new('\u{25C6}', theme.iteration_graph_phase_icon)
  }

  /// Map a task status string to its corresponding icon glyph and style.
  pub fn status(status: &str, theme: &Theme) -> Self {
    match status {
      "open" => Self::new('\u{25CB}', theme.task_list_icon_open),
      "in-progress" => Self::new('\u{25D0}', theme.task_list_icon_in_progress),
      "done" => Self::new('\u{25CF}', theme.task_list_icon_done),
      "cancelled" => Self::new('\u{2298}', theme.task_list_icon_cancelled),
      _ => Self::new('\u{25CB}', theme.task_list_icon_open),
    }
  }
}

impl Display for Icon {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self.ch.paint(self.style))
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  mod blocked {
    use super::*;

    #[test]
    fn it_returns_correct_icon() {
      let theme = Theme::default();
      let icon = Icon::blocked(&theme);
      assert_eq!(icon.ch, '\u{2297}');
    }
  }

  mod blocking {
    use super::*;

    #[test]
    fn it_returns_correct_icon() {
      let theme = Theme::default();
      let icon = Icon::blocking(&theme);
      assert_eq!(icon.ch, '!');
    }
  }

  mod phase {
    use super::*;

    #[test]
    fn it_returns_correct_icon() {
      let theme = Theme::default();
      let icon = Icon::phase(&theme);
      assert_eq!(icon.ch, '\u{25C6}');
    }
  }

  mod status {
    use super::*;

    #[test]
    fn it_maps_cancelled() {
      let theme = Theme::default();
      let icon = Icon::status("cancelled", &theme);
      assert_eq!(icon.ch, '\u{2298}');
    }

    #[test]
    fn it_maps_done() {
      let theme = Theme::default();
      let icon = Icon::status("done", &theme);
      assert_eq!(icon.ch, '\u{25CF}');
    }

    #[test]
    fn it_maps_in_progress() {
      let theme = Theme::default();
      let icon = Icon::status("in-progress", &theme);
      assert_eq!(icon.ch, '\u{25D0}');
    }

    #[test]
    fn it_maps_open() {
      let theme = Theme::default();
      let icon = Icon::status("open", &theme);
      assert_eq!(icon.ch, '\u{25CB}');
    }

    #[test]
    fn it_maps_unknown_to_open() {
      let theme = Theme::default();
      let icon = Icon::status("unknown", &theme);
      assert_eq!(icon.ch, '\u{25CB}');
    }
  }

  mod display {
    use super::*;

    #[test]
    fn it_renders_correct_character() {
      let icon = Icon::new('X', Style::default());
      let rendered = format!("{icon}");
      assert!(rendered.contains('X'));
    }
  }
}
