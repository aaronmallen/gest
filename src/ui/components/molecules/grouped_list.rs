//! Headed list molecule with a summary line and pre-rendered rows.

use std::fmt::{self, Display, Formatter};

use yansi::Paint;

/// A headed list section with heading, summary, and rows.
pub struct Component<'a> {
  heading: &'a str,
  rows: Vec<String>,
  summary: String,
}

impl<'a> Component<'a> {
  /// Create a grouped list with the given heading and summary line.
  pub fn new(heading: &'a str, summary: impl Into<String>) -> Self {
    Self {
      heading,
      rows: Vec::new(),
      summary: summary.into(),
    }
  }

  /// Add a pre-rendered row to the list.
  pub fn row(mut self, row: impl Into<String>) -> Self {
    self.rows.push(row.into());
    self
  }
}

impl Display for Component<'_> {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    let theme = crate::ui::style::global();
    write!(
      f,
      "{}  {}",
      self.heading.paint(*theme.list_heading()),
      self.summary.paint(*theme.list_summary()),
    )?;

    if !self.rows.is_empty() {
      writeln!(f)?;
      for row in &self.rows {
        write!(f, "\n{row}")?;
      }
    }

    Ok(())
  }
}
