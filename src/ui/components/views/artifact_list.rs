//! Full artifact list view with Grid-aligned columns and count summary.

use std::fmt::{self, Display, Formatter};

use crate::ui::components::{
  atoms::{Badge, Column, Id, Tag, Title},
  molecules::{EmptyList, Grid, GroupedList, Row},
};

/// A single artifact entry for the list view.
pub struct ArtifactEntry {
  /// Whether the artifact is archived, which dims its row and appends a badge.
  pub archived: bool,
  /// Short ID string displayed as the leading column.
  pub id: String,
  /// Tag labels rendered as `#tag` chips after the title.
  pub tags: Vec<String>,
  /// Artifact title rendered in the title column.
  pub title: String,
}

/// Full artifact list view using Grid for column alignment.
pub struct Component {
  entries: Vec<ArtifactEntry>,
  prefix_len: usize,
}

impl Component {
  /// Create a list view from the entries, using `prefix_len` highlighted chars in each ID.
  pub fn new(entries: Vec<ArtifactEntry>, prefix_len: usize) -> Self {
    Self {
      entries,
      prefix_len,
    }
  }
}

impl Display for Component {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    if self.entries.is_empty() {
      return write!(f, "{}", EmptyList::new("artifacts"));
    }

    let theme = crate::ui::style::global();
    let count = self.entries.len();
    let summary = format!("{count} {}", if count == 1 { "artifact" } else { "artifacts" });
    let mut grid = Grid::new().spacing(2);

    for entry in &self.entries {
      let id = Id::new(&entry.id).prefix_len(self.prefix_len);

      let title_style = if entry.archived {
        *theme.artifact_list_title_archived()
      } else {
        *theme.artifact_list_title()
      };
      let title = Title::new(&entry.title, title_style);

      let tag_str = if !entry.tags.is_empty() {
        let tag_style = if entry.archived {
          *theme.artifact_list_tag_archived()
        } else {
          *theme.tag()
        };
        Tag::new(entry.tags.clone(), tag_style).to_string()
      } else {
        String::new()
      };

      let mut row = Row::new().col(Column::natural(id)).col(Column::natural(title));

      if !tag_str.is_empty() {
        row = row.col(Column::natural(tag_str));
      }

      if entry.archived {
        row = row.col(Column::natural(Badge::new(
          "[archived]",
          *theme.artifact_list_archived_badge(),
        )));
      }

      grid.push(row);
    }

    let list = GroupedList::new("artifacts", summary).row(grid.to_string());
    write!(f, "{list}")
  }
}
