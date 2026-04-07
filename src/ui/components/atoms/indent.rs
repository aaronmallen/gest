//! Layout atom that wraps content with a left margin.

use std::fmt::{self, Display, Formatter};

/// Wraps content with a left margin, prepending the margin to every line
/// including lines after embedded newlines.
pub struct Component {
  content: String,
  margin: usize,
}

impl Component {
  /// Create an indented wrapper with the given margin width in spaces.
  pub fn new(margin: usize, content: impl Display) -> Self {
    Self {
      content: content.to_string(),
      margin,
    }
  }
}

impl Display for Component {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    let prefix = " ".repeat(self.margin);
    for (i, line) in self.content.lines().enumerate() {
      if i > 0 {
        writeln!(f)?;
      }
      write!(f, "{prefix}{line}")?;
    }
    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn it_indents_single_line() {
    let indent = Component::new(4, "hello");

    assert_eq!(indent.to_string(), "    hello");
  }

  #[test]
  fn it_indents_every_line() {
    let indent = Component::new(2, "line one\nline two\nline three");

    assert_eq!(indent.to_string(), "  line one\n  line two\n  line three");
  }

  #[test]
  fn it_handles_zero_margin() {
    let indent = Component::new(0, "no indent");

    assert_eq!(indent.to_string(), "no indent");
  }
}
