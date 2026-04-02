//! Row layout container for terminal output.

use std::fmt::{self, Display, Formatter};

use unicode_width::UnicodeWidthChar;

use super::utils;

/// A horizontal sequence of display items with configurable spacing.
///
/// Columns that exceed available width are truncated with an ellipsis.
pub struct Row {
  entries: Vec<String>,
  max_width: Option<u16>,
  spacing: usize,
}

impl Row {
  pub fn new() -> Self {
    Self {
      entries: Vec::new(),
      spacing: 2,
      max_width: None,
    }
  }

  /// Append an item as the next column in the row.
  pub fn col(mut self, item: impl Display) -> Self {
    self.entries.push(item.to_string());
    self
  }

  /// Set the maximum visible width (defaults to terminal width).
  #[cfg(test)]
  pub fn max_width(mut self, width: u16) -> Self {
    self.max_width = Some(width);
    self
  }

  /// Set the number of space characters between columns (default 2).
  pub fn spacing(mut self, spacing: usize) -> Self {
    self.spacing = spacing;
    self
  }
}

impl Default for Row {
  fn default() -> Self {
    Self::new()
  }
}

impl Display for Row {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    if self.entries.is_empty() {
      return Ok(());
    }

    let max_width = self.max_width.unwrap_or_else(utils::terminal_width) as usize;
    let spacer = " ".repeat(self.spacing);

    let mut current_width: usize = 0;
    let mut first = true;

    for entry in &self.entries {
      let col_width = utils::display_width(entry);

      if !first {
        let needed = self.spacing + col_width;
        if current_width + needed > max_width {
          let remaining = max_width.saturating_sub(current_width + self.spacing);
          if remaining > 1 {
            write!(f, "{spacer}")?;
            truncate_visible(f, entry, remaining)?;
          }
          break;
        }
        write!(f, "{spacer}")?;
        current_width += self.spacing;
      }

      write!(f, "{entry}")?;
      current_width += col_width;
      first = false;
    }

    Ok(())
  }
}

/// Write `s` into `f`, truncating with an ellipsis once visible width exceeds `budget`.
fn truncate_visible(f: &mut Formatter<'_>, s: &str, budget: usize) -> fmt::Result {
  if budget == 0 {
    return Ok(());
  }

  let mut visible = 0;
  let mut chars = s.chars().peekable();

  while let Some(c) = chars.next() {
    if c == '\x1b' {
      write!(f, "{c}")?;
      if chars.peek() == Some(&'[') {
        if let Some(bracket) = chars.next() {
          write!(f, "{bracket}")?;
        }
        for inner in chars.by_ref() {
          write!(f, "{inner}")?;
          if inner.is_ascii_alphabetic() {
            break;
          }
        }
      }
      continue;
    }

    let w = UnicodeWidthChar::width(c).unwrap_or(0);
    if visible + w > budget.saturating_sub(1) {
      write!(f, "\u{2026}")?;
      break;
    }
    write!(f, "{c}")?;
    visible += w;
  }

  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;

  mod row {
    use super::*;

    #[test]
    fn it_renders_custom_spacing() {
      let row = Row::new().spacing(1).max_width(80).col("a").col("b").col("c");
      assert_eq!(row.to_string(), "a b c");
    }

    #[test]
    fn it_renders_empty() {
      let row = Row::new().max_width(80);
      assert_eq!(row.to_string(), "");
    }

    #[test]
    fn it_renders_single_item() {
      let row = Row::new().max_width(80).col("hello");
      assert_eq!(row.to_string(), "hello");
    }

    #[test]
    fn it_renders_three_items_with_default_spacing() {
      let row = Row::new().max_width(80).col("a").col("b").col("c");
      assert_eq!(row.to_string(), "a  b  c");
    }

    #[test]
    fn it_truncates_on_overflow() {
      let row = Row::new().max_width(10).col("aaaa").col("bbbbbb");
      let rendered = row.to_string();
      assert!(rendered.starts_with("aaaa  "));
      assert!(rendered.contains('\u{2026}'));
      let visible_width = utils::display_width(&rendered);
      assert!(visible_width <= 10, "visible width {visible_width} exceeds 10");
    }
  }
}
