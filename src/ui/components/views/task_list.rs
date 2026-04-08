//! Full task list view with Grid-aligned columns and count summary.

use std::fmt::{self, Display, Formatter};

use crate::ui::components::{
  atoms::{Badge, Column, Icon, Id, Tag, Title},
  molecules::{EmptyList, Grid, GroupedList, Indicators, Row, StatusBadge},
};

/// A single task entry for the list view.
pub struct TaskEntry {
  /// Short ID of the task blocking this one, if any; swaps the row icon to blocked.
  pub blocked_by: Option<String>,
  /// Whether this task blocks others, which renders a `blocking` indicator.
  pub blocking: bool,
  /// Short ID string displayed as the leading column.
  pub id: String,
  /// Optional priority level rendered as a `[P<n>]` badge.
  pub priority: Option<u8>,
  /// Task status string driving icon, title styling, and status badge.
  pub status: String,
  /// Tag labels rendered as `#tag` chips at the end of the row.
  pub tags: Vec<String>,
  /// Task title rendered in the title column.
  pub title: String,
}

/// Full task list view using Grid for column alignment.
pub struct Component {
  entries: Vec<TaskEntry>,
  prefix_len: usize,
}

impl Component {
  /// Create a list view from the entries, using `prefix_len` highlighted chars in each ID.
  pub fn new(entries: Vec<TaskEntry>, prefix_len: usize) -> Self {
    Self {
      entries,
      prefix_len,
    }
  }
}

impl Display for Component {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    if self.entries.is_empty() {
      return write!(f, "{}", EmptyList::new("tasks"));
    }

    let theme = crate::ui::style::global();
    let count = self.entries.len();
    let summary = format!("{count} {}", if count == 1 { "task" } else { "tasks" });
    let mut grid = Grid::new().spacing(2);

    for entry in &self.entries {
      let icon = if entry.blocked_by.is_some() {
        Icon::blocked()
      } else {
        Icon::status(&entry.status)
      };

      let id = Id::new(&entry.id).prefix_len(self.prefix_len);

      let priority_str = match entry.priority {
        Some(p) => Badge::new(format!("[P{p}]"), *theme.task_list_priority()).to_string(),
        None => String::new(),
      };

      let title_style = if entry.status == "cancelled" {
        *theme.task_list_title_cancelled()
      } else {
        *theme.task_list_title()
      };
      let title = Title::new(&entry.title, title_style);

      let badge_status = if entry.blocked_by.is_some() {
        "blocked"
      } else {
        &entry.status
      };
      let status_badge = StatusBadge::new(badge_status);

      let tag_str = if !entry.tags.is_empty() {
        Tag::new(entry.tags.clone(), *theme.tag()).to_string()
      } else {
        String::new()
      };

      // Indicators reuse the list's prefix pool. The blocked-by IDs may
      // belong to a different pool than the row task, but threading two
      // pools through every entry would complicate the API for marginal
      // value, so we accept the row task's pool as the indicator pool too.
      let blocked_by_ids: Vec<&str> = entry.blocked_by.as_deref().into_iter().collect();
      let indicators_str = Indicators::new()
        .blocking(entry.blocking)
        .blocked_by(blocked_by_ids)
        .prefix_len(self.prefix_len)
        .to_string();

      let mut row = Row::new()
        .col(Column::natural(icon))
        .col(Column::natural(id))
        .col(Column::natural(priority_str))
        .col(Column::natural(title))
        .col(Column::natural(status_badge));

      if !indicators_str.is_empty() {
        row = row.col(Column::natural(indicators_str));
      }

      if !tag_str.is_empty() {
        row = row.col(Column::natural(tag_str));
      }

      grid.push(row);
    }

    let list = GroupedList::new("tasks", summary).row(grid.to_string());
    write!(f, "{list}")
  }
}
