//! Tag list atom rendering `#`-prefixed labels.

use std::fmt::{self, Display, Formatter};

use yansi::{Paint, Style};

/// A collection of tag labels rendered with `#` prefix and double-space separation.
pub struct Component {
  labels: Vec<String>,
  style: Style,
}

impl Component {
  /// Create a tag display from a list of labels with the given style.
  pub fn new(labels: Vec<String>, style: Style) -> Self {
    Self {
      labels,
      style,
    }
  }
}

impl Display for Component {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    for (i, label) in self.labels.iter().enumerate() {
      if i > 0 {
        write!(f, "  ")?;
      }
      write!(f, "{}", format!("#{label}").paint(self.style))?;
    }
    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn render(c: &Component) -> String {
    yansi::disable();
    let out = c.to_string();
    yansi::enable();
    out
  }

  #[test]
  fn it_renders_single_tag_without_spacing() {
    let tags = Component::new(vec!["core".into()], Style::default());

    let out = render(&tags);

    assert_eq!(out, "#core");
  }

  #[test]
  fn it_uses_double_space_between_tags() {
    let tags = Component::new(vec!["bug".into(), "ui".into(), "v2".into()], Style::default());

    let out = render(&tags);

    assert_eq!(out, "#bug  #ui  #v2");
  }
}
