//! Full project list view with Grid-aligned columns and count summary.

use std::fmt::{self, Display, Formatter};

use crate::ui::components::{
  atoms::{Column, Id, Title},
  molecules::{EmptyList, Grid, GroupedList, Row},
};

/// A single project entry for the list view.
pub struct ProjectEntry {
  pub id: String,
  pub root: String,
}

/// Full project list view using Grid for column alignment.
pub struct Component {
  entries: Vec<ProjectEntry>,
  prefix_len: usize,
}

impl Component {
  pub fn new(entries: Vec<ProjectEntry>, prefix_len: usize) -> Self {
    Self {
      entries,
      prefix_len,
    }
  }
}

impl Display for Component {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    if self.entries.is_empty() {
      return write!(f, "{}", EmptyList::new("projects"));
    }

    let theme = crate::ui::style::global();
    let count = self.entries.len();
    let summary = format!("{count} {}", if count == 1 { "project" } else { "projects" });
    let mut grid = Grid::new().spacing(2);

    for entry in &self.entries {
      let id = Id::new(&entry.id).prefix_len(self.prefix_len);
      let root = Title::new(&entry.root, *theme.project_list_root());

      let row = Row::new().col(Column::natural(id)).col(Column::natural(root));
      grid.push(row);
    }

    let list = GroupedList::new("projects", summary).row(grid.to_string());
    write!(f, "{list}")
  }
}
