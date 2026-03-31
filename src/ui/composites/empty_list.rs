use std::fmt::{self, Display, Formatter};

use yansi::Paint;

use crate::ui::theme::Theme;

/// Renders a "no {entity} found" placeholder for empty list views.
pub struct EmptyList<'a> {
  entity: &'a str,
  theme: &'a Theme,
}

impl<'a> EmptyList<'a> {
  pub fn new(entity: &'a str, theme: &'a Theme) -> Self {
    Self {
      entity,
      theme,
    }
  }
}

impl Display for EmptyList<'_> {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    write!(f, "{}", format!("no {} found", self.entity).paint(self.theme.muted))
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn theme() -> Theme {
    Theme::default()
  }

  #[test]
  fn it_renders_different_entity() {
    let theme = theme();
    let empty = EmptyList::new("artifacts", &theme);
    let rendered = format!("{empty}");

    assert!(rendered.contains("no artifacts found"));
  }

  #[test]
  fn it_renders_entity_name() {
    let theme = theme();
    let empty = EmptyList::new("tasks", &theme);
    let rendered = format!("{empty}");

    assert!(rendered.contains("no tasks found"));
  }
}
