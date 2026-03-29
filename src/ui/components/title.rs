use std::fmt;

/// Atomic component for rendering a title with optional truncation.
///
/// By default, renders the title text as-is. When `max_width` is set
/// and the text exceeds that width, truncates with an ellipsis character.
pub struct Title<'a> {
  text: &'a str,
  max_width: Option<usize>,
}

impl<'a> Title<'a> {
  pub fn new(text: &'a str) -> Self {
    Self {
      text,
      max_width: None,
    }
  }

  pub fn max_width(mut self, width: usize) -> Self {
    self.max_width = Some(width);
    self
  }
}

impl fmt::Display for Title<'_> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self.max_width {
      Some(max) if self.text.chars().count() > max && max > 1 => {
        let truncated: String = self.text.chars().take(max - 1).collect();
        write!(f, "{truncated}\u{2026}")
      }
      Some(max) if self.text.chars().count() > max => {
        let truncated: String = self.text.chars().take(max).collect();
        write!(f, "{truncated}")
      }
      _ => write!(f, "{}", self.text),
    }
  }
}

#[cfg(test)]
mod tests {
  use pretty_assertions::assert_eq;

  use super::*;

  #[test]
  fn it_renders_text_as_is_by_default() {
    let title = Title::new("My Title");
    assert_eq!(title.to_string(), "My Title");
  }

  #[test]
  fn it_truncates_with_ellipsis_when_exceeding_max_width() {
    let title = Title::new("Very long title text").max_width(13);
    assert_eq!(title.to_string(), "Very long ti\u{2026}");
  }

  #[test]
  fn it_renders_full_text_when_shorter_than_max_width() {
    let title = Title::new("Short").max_width(20);
    assert_eq!(title.to_string(), "Short");
  }

  #[test]
  fn it_renders_full_text_when_equal_to_max_width() {
    let title = Title::new("Exact").max_width(5);
    assert_eq!(title.to_string(), "Exact");
  }
}
