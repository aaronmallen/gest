//! Row molecule — horizontal sequence of Column atoms with flex-aware width resolution.

use std::fmt::{self, Display, Formatter};

use crate::ui::components::atoms::{Column, Flex};

/// A horizontal sequence of columns with configurable spacing and terminal-width-aware
/// layout. Row owns all measurement and width resolution.
pub struct Component {
  columns: Vec<Column>,
  max_width: Option<usize>,
  spacing: usize,
}

impl Component {
  pub fn new() -> Self {
    Self {
      columns: Vec::new(),
      max_width: None,
      spacing: 2,
    }
  }

  /// Append a Column to this row.
  pub fn col(mut self, column: Column) -> Self {
    self.columns.push(column);
    self
  }

  /// Set the number of space characters between columns (default 2).
  pub fn spacing(mut self, spacing: usize) -> Self {
    self.spacing = spacing;
    self
  }

  /// Set an explicit maximum width (defaults to terminal width).
  pub fn max_width(mut self, width: usize) -> Self {
    self.max_width = Some(width);
    self
  }

  /// Returns the columns for external inspection (used by Grid).
  pub fn columns(&self) -> &[Column] {
    &self.columns
  }

  /// Returns a mutable reference to the columns (used by Grid to promote natural to fixed).
  pub fn columns_mut(&mut self) -> &mut Vec<Column> {
    &mut self.columns
  }

  /// Resolve column widths and render.
  fn resolve_widths(&self) -> Vec<(usize, &str)> {
    let available = self.max_width.unwrap_or_else(|| terminal_width() as usize);
    let n = self.columns.len();
    if n == 0 {
      return Vec::new();
    }

    let total_spacing = if n > 1 { (n - 1) * self.spacing } else { 0 };
    let mut remaining = available.saturating_sub(total_spacing);

    let mut widths = vec![0usize; n];
    let mut total_flex_weight = 0u32;

    // Pass 1: allocate fixed columns
    for (i, col) in self.columns.iter().enumerate() {
      if let Flex::Fixed(w) = col.flex_intent() {
        widths[i] = w;
        remaining = remaining.saturating_sub(w);
      }
    }

    // Pass 2: measure and allocate natural columns
    for (i, col) in self.columns.iter().enumerate() {
      if let Flex::Natural = col.flex_intent() {
        let w = display_width(col.content());
        widths[i] = w;
        remaining = remaining.saturating_sub(w);
      }
    }

    // Pass 3: collect flex weights
    for col in &self.columns {
      if let Flex::Flex(weight) = col.flex_intent() {
        total_flex_weight += weight;
      }
    }

    // Pass 4: distribute remaining space to flex columns
    if total_flex_weight > 0 {
      for (i, col) in self.columns.iter().enumerate() {
        if let Flex::Flex(weight) = col.flex_intent() {
          widths[i] = (remaining as u64 * weight as u64 / total_flex_weight as u64) as usize;
        }
      }
    }

    self
      .columns
      .iter()
      .zip(widths.iter())
      .map(|(col, &w)| (w, col.content()))
      .collect()
  }
}

impl Display for Component {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    let resolved = self.resolve_widths();
    let spacer = " ".repeat(self.spacing);

    for (i, (width, content)) in resolved.iter().enumerate() {
      if i > 0 {
        write!(f, "{spacer}")?;
      }

      let visible_width = display_width(content);
      write!(f, "{content}")?;

      // Pad to declared width if content is narrower
      if visible_width < *width {
        write!(f, "{}", " ".repeat(width - visible_width))?;
      }
    }

    Ok(())
  }
}

/// Return the visible column width of `s` after stripping ANSI escape sequences.
pub fn display_width(s: &str) -> usize {
  unicode_width::UnicodeWidthStr::width(strip_ansi(s).as_str())
}

/// Query the terminal width, falling back to 80 columns if unavailable.
pub fn terminal_width() -> u16 {
  terminal_size::terminal_size().map(|(w, _)| w.0).unwrap_or(80)
}

/// Remove ANSI CSI and OSC escape sequences from a string.
pub fn strip_ansi(s: &str) -> String {
  let mut result = String::with_capacity(s.len());
  let mut chars = s.chars().peekable();

  while let Some(c) = chars.next() {
    if c == '\x1b' {
      match chars.peek() {
        Some('[') => {
          chars.next();
          while let Some(&next) = chars.peek() {
            chars.next();
            if next.is_ascii_alphabetic() {
              break;
            }
          }
        }
        Some(']') => {
          chars.next();
          while let Some(&next) = chars.peek() {
            if next == '\x07' {
              chars.next();
              break;
            }
            if next == '\x1b' {
              chars.next();
              if chars.peek() == Some(&'\\') {
                chars.next();
              }
              break;
            }
            chars.next();
          }
        }
        _ => {}
      }
    } else {
      result.push(c);
    }
  }

  result
}

#[cfg(test)]
mod tests {
  use super::*;

  mod display_width_fn {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_counts_ascii_width() {
      assert_eq!(display_width("hello"), 5);
    }

    #[test]
    fn it_returns_zero_for_empty() {
      assert_eq!(display_width(""), 0);
    }

    #[test]
    fn it_strips_ansi_from_width() {
      assert_eq!(display_width("\x1b[31mred\x1b[0m"), 3);
    }
  }

  mod row {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_distributes_flex_proportionally() {
      let row = Component::new()
        .max_width(40)
        .spacing(2)
        .col(Column::natural("ab"))
        .col(Column::flex(1, "x"))
        .col(Column::flex(2, "y"));

      let resolved = row.resolve_widths();

      assert_eq!(resolved[0].0, 2); // natural: content width
      let flex1 = resolved[1].0;
      let flex2 = resolved[2].0;
      // flex(2) should get ~2x flex(1)
      assert!(flex2 >= flex1, "flex(2) should be >= flex(1)");
    }

    #[test]
    fn it_pads_fixed_columns() {
      let row = Component::new()
        .max_width(80)
        .col(Column::fixed(10, "hi"))
        .col(Column::natural("world"));
      let output = row.to_string();

      // "hi" should be padded to 10 chars
      assert!(
        output.starts_with("hi        "),
        "fixed column should be padded, got: '{output}'"
      );
    }

    #[test]
    fn it_renders_empty() {
      let row = Component::new().max_width(80);

      assert_eq!(row.to_string(), "");
    }

    #[test]
    fn it_renders_natural_columns() {
      let row = Component::new()
        .max_width(80)
        .col(Column::natural("a"))
        .col(Column::natural("b"))
        .col(Column::natural("c"));

      assert_eq!(row.to_string(), "a  b  c");
    }
  }
}
