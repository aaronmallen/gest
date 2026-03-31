use std::fmt::{self, Display, Formatter};

use yansi::{Paint, Style};

/// A single styled tag, rendered with a leading `#`.
pub struct Tag {
  name: String,
  style: Style,
}

impl Tag {
  /// Create a tag from a name (the `#` prefix is added on display).
  pub fn new(name: impl Into<String>, style: Style) -> Self {
    Self {
      name: name.into(),
      style,
    }
  }
}

impl Display for Tag {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    write!(f, "{}", format!("#{}", self.name).paint(self.style))
  }
}

/// A space-separated collection of [`Tag`]s.
pub struct Tags {
  tags: Vec<Tag>,
}

impl Tags {
  /// Build a tag list from a slice of names, all sharing the same style.
  pub fn new(names: &[String], style: Style) -> Self {
    Self {
      tags: names.iter().map(|n| Tag::new(n.as_str(), style)).collect(),
    }
  }
}

impl Display for Tags {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    for (i, tag) in self.tags.iter().enumerate() {
      if i > 0 {
        write!(f, "  ")?;
      }
      write!(f, "{tag}")?;
    }
    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  mod tag {
    use super::*;

    mod display {
      use super::*;

      #[test]
      fn it_renders_hash_name() {
        yansi::disable();
        let tag = Tag::new("urgent", Style::default());

        assert_eq!(tag.to_string(), "#urgent");
      }
    }
  }

  mod tags {
    use super::*;

    mod display {
      use super::*;

      #[test]
      fn it_renders_empty_string_for_no_tags() {
        let tags = Tags::new(&[], Style::default());

        assert_eq!(tags.to_string(), "");
      }

      #[test]
      fn it_renders_multiple_space_separated() {
        yansi::disable();
        let names = vec!["bug".to_string(), "ui".to_string(), "v2".to_string()];
        let tags = Tags::new(&names, Style::default());

        assert_eq!(tags.to_string(), "#bug  #ui  #v2");
      }
    }
  }
}
