//! Grid molecule — multi-row table that normalizes column widths across all rows.

use std::fmt::{self, Display, Formatter};

use super::row::display_width;
use crate::ui::components::{
  atoms::{Column, Flex},
  molecules::Row,
};

/// A multi-row table that computes column widths across all rows before rendering.
///
/// Grid pre-scans all rows to normalize `natural` column widths — for each column
/// position, it computes the max display_width across all rows and promotes each
/// `natural` column to `fixed(max_width)` before delegating to Row for rendering.
pub struct Component {
  max_width: Option<usize>,
  rows: Vec<Row>,
  spacing: usize,
}

impl Component {
  /// Create an empty grid with default spacing of 2 columns.
  pub fn new() -> Self {
    Self {
      max_width: None,
      rows: Vec::new(),
      spacing: 2,
    }
  }

  /// Set the number of space characters between columns (default 2).
  pub fn spacing(mut self, spacing: usize) -> Self {
    self.spacing = spacing;
    self
  }

  /// Set an explicit maximum width for all rows.
  #[cfg(test)]
  pub fn max_width(mut self, width: usize) -> Self {
    self.max_width = Some(width);
    self
  }

  /// Add a row to the grid.
  pub fn push(&mut self, row: Row) {
    self.rows.push(row);
  }
}

impl Display for Component {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    if self.rows.is_empty() {
      return Ok(());
    }

    // Compute max natural width per column position
    let max_cols = self.rows.iter().map(|r| r.columns().len()).max().unwrap_or(0);
    let mut max_widths = vec![0usize; max_cols];

    for row in &self.rows {
      for (i, col) in row.columns().iter().enumerate() {
        if let Flex::Natural = col.flex_intent() {
          let w = display_width(col.content());
          if w > max_widths[i] {
            max_widths[i] = w;
          }
        }
      }
    }

    // Render each row with promoted natural columns
    for (row_idx, row) in self.rows.iter().enumerate() {
      if row_idx > 0 {
        writeln!(f)?;
      }

      // Build a new row with natural columns promoted to fixed
      let mut new_row = Row::new().spacing(self.spacing);
      if let Some(w) = self.max_width {
        new_row = new_row.max_width(w);
      }

      for (i, col) in row.columns().iter().enumerate() {
        let new_col = match col.flex_intent() {
          Flex::Natural if i < max_widths.len() && max_widths[i] > 0 => Column::fixed(max_widths[i], col.content()),
          Flex::Fixed(w) => Column::fixed(w, col.content()),
          Flex::Flex(w) => Column::flex(w, col.content()),
          _ => Column::natural(col.content()),
        };
        new_row = new_row.col(new_col);
      }

      write!(f, "{new_row}")?;
    }

    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn it_aligns_natural_columns_across_rows() {
    let mut grid = Component::new().spacing(2).max_width(80);

    grid.push(Row::new().col(Column::natural("a")).col(Column::natural("bb")));
    grid.push(Row::new().col(Column::natural("ccc")).col(Column::natural("d")));

    let output = grid.to_string();
    let lines: Vec<&str> = output.lines().collect();

    assert_eq!(lines.len(), 2);
    // First column should be padded to 3 (max of "a" and "ccc")
    assert!(
      lines[0].starts_with("a  "),
      "first row first col should be padded, got: '{}'",
      lines[0]
    );
    assert!(
      lines[1].starts_with("ccc"),
      "second row first col should not need padding"
    );
  }

  #[test]
  fn it_preserves_flex_columns() {
    let mut grid = Component::new().spacing(2).max_width(40);

    grid.push(Row::new().col(Column::natural("id")).col(Column::flex(1, "title")));
    grid.push(Row::new().col(Column::natural("longer")).col(Column::flex(1, "t")));

    let output = grid.to_string();
    let lines: Vec<&str> = output.lines().collect();

    assert_eq!(lines.len(), 2);
  }

  #[test]
  fn it_renders_empty_grid() {
    let grid = Component::new();

    assert_eq!(grid.to_string(), "");
  }
}
