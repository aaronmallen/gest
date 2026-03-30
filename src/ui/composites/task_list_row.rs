use std::fmt::{self, Display, Formatter};

use yansi::Paint;

use crate::ui::{
  atoms::{badge::Badge, icon::Icon, id::Id, tag::Tags, title::Title},
  composites::status_badge::StatusBadge,
  layout::Row,
  theme::Theme,
  utils,
};

/// Max display width for task titles in list rows.
const TITLE_PAD: usize = 35;

/// A single row in a task list, showing status icon, id, priority, title, status badge, and indicators.
pub struct TaskListRow<'a> {
  blocked_by: Option<&'a str>,
  blocking: bool,
  blocking_pad: usize,
  id: &'a str,
  priority: Option<u8>,
  priority_pad: usize,
  status: &'a str,
  status_pad: usize,
  tags: &'a [String],
  theme: &'a Theme,
  title_text: &'a str,
}

impl<'a> TaskListRow<'a> {
  pub fn new(status: &'a str, id: &'a str, title_text: &'a str, theme: &'a Theme) -> Self {
    Self {
      status,
      id,
      title_text,
      priority: None,
      priority_pad: 0,
      tags: &[],
      blocking: false,
      blocked_by: None,
      blocking_pad: 0,
      status_pad: 0,
      theme,
    }
  }

  /// Sets the ID of the task blocking this one.
  pub fn blocked_by(mut self, id: Option<&'a str>) -> Self {
    self.blocked_by = id;
    self
  }

  /// Marks this task as blocking other tasks.
  pub fn blocking(mut self, b: bool) -> Self {
    self.blocking = b;
    self
  }

  /// Sets the task priority level for the badge.
  pub fn priority(mut self, p: Option<u8>) -> Self {
    self.priority = p;
    self
  }

  /// Returns the rendered priority badge string for this row.
  pub fn priority_badge_string(&self) -> String {
    match self.priority {
      Some(p) => Badge::new(format!("[P{p}]"), self.theme.task_list_priority).to_string(),
      None => String::new(),
    }
  }

  /// Returns the display width of the priority badge for this row.
  pub fn priority_badge_width(&self) -> usize {
    utils::display_width(&self.priority_badge_string())
  }

  /// Sets minimum display width for the blocking info column.
  pub fn blocking_pad(mut self, w: usize) -> Self {
    self.blocking_pad = w;
    self
  }

  /// Sets minimum display width for the priority badge column.
  pub fn priority_pad(mut self, w: usize) -> Self {
    self.priority_pad = w;
    self
  }

  /// Sets minimum display width for the status badge column.
  pub fn status_pad(mut self, w: usize) -> Self {
    self.status_pad = w;
    self
  }

  /// Sets the tags to append after the status badge.
  pub fn tags(mut self, t: &'a [String]) -> Self {
    self.tags = t;
    self
  }

  /// Returns the rendered blocking info string for this row.
  pub fn blocking_info_string(&self) -> String {
    let mut parts: Vec<String> = Vec::new();

    if self.blocking {
      let icon = Icon::blocking(self.theme);
      parts.push(Badge::new(format!("{icon} blocking"), self.theme.indicator_blocking).to_string());
    }

    if let Some(blocker_id) = self.blocked_by {
      let label = "blocked-by".paint(self.theme.indicator_blocked_by_label);
      let id = Id::new(blocker_id, self.theme);
      parts.push(format!("{label} {id}"));
    }

    parts.join("  ")
  }

  /// Returns the display width of the blocking info for this row.
  pub fn blocking_info_width(&self) -> usize {
    utils::display_width(&self.blocking_info_string())
  }

  /// Returns the rendered status badge string for this row.
  pub fn status_badge_string(&self) -> String {
    self.status_badge().to_string()
  }

  /// Returns the display width of the status badge for this row.
  pub fn status_badge_width(&self) -> usize {
    utils::display_width(&self.status_badge_string())
  }

  fn leading_icon(&self) -> Icon {
    if self.blocked_by.is_some() {
      Icon::blocked(self.theme)
    } else {
      Icon::status(self.status, self.theme)
    }
  }

  fn status_badge(&self) -> StatusBadge<'_> {
    let status = if self.blocked_by.is_some() {
      "blocked"
    } else {
      self.status
    };
    StatusBadge::new(status, self.theme)
  }

  fn title(&self) -> Title {
    let style = if self.status == "cancelled" {
      self.theme.task_list_title_cancelled
    } else {
      self.theme.task_list_title
    };
    Title::new(self.title_text, style)
      .max_width(TITLE_PAD)
      .pad_to(TITLE_PAD)
  }
}

impl Display for TaskListRow<'_> {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    let mut row = Row::new().spacing(2);

    row = row.col(self.leading_icon());

    row = row.col(Id::new(self.id, self.theme));

    let priority_str = self.priority_badge_string();
    if self.priority_pad > 0 {
      let pad = self.priority_pad.saturating_sub(utils::display_width(&priority_str));
      row = row.col(format!("{priority_str}{}", " ".repeat(pad)));
    } else if !priority_str.is_empty() {
      row = row.col(priority_str);
    }

    row = row.col(self.title());

    let status_str = self.status_badge_string();
    if self.status_pad > 0 {
      let pad = self.status_pad.saturating_sub(utils::display_width(&status_str));
      row = row.col(format!("{status_str}{}", " ".repeat(pad)));
    } else {
      row = row.col(status_str);
    }

    let blocking_str = self.blocking_info_string();
    if self.blocking_pad > 0 {
      let pad = self.blocking_pad.saturating_sub(utils::display_width(&blocking_str));
      row = row.col(format!("{blocking_str}{}", " ".repeat(pad)));
    } else if !blocking_str.is_empty() {
      row = row.col(blocking_str);
    }

    if !self.tags.is_empty() {
      row = row.col(Tags::new(self.tags, self.theme.tag));
    }

    write!(f, "{row}")
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn theme() -> Theme {
    Theme::default()
  }

  fn render(row: &TaskListRow) -> String {
    yansi::disable();
    let out = row.to_string();
    yansi::enable();
    out
  }

  #[test]
  fn it_appends_tags_when_present() {
    let theme = theme();
    let tags = vec!["urgent".to_string(), "backend".to_string()];
    let row = TaskListRow::new("open", "qtsdwcaz", "probe dedup", &theme).tags(&tags);
    let output = render(&row);

    assert!(output.contains("#urgent"), "should contain first tag");
    assert!(output.contains("#backend"), "should contain second tag");
  }

  #[test]
  fn it_omits_priority_badge_when_none() {
    let theme = theme();
    let row = TaskListRow::new("open", "qtsdwcaz", "probe dedup by content hash", &theme);
    let output = render(&row);

    assert!(!output.contains("[P"), "should not contain priority badge");
  }

  #[test]
  fn it_renders_blocked_row_with_blocked_by() {
    let theme = theme();
    let row = TaskListRow::new("open", "mxdtqrbn", "ctx window", &theme).blocked_by(Some("hpvrlbme"));
    let output = render(&row);

    assert!(output.contains("blocked"), "should show blocked status");
    assert!(output.contains("blocked-by"), "should show blocked-by label");
    assert!(output.contains("hpvrlbme"), "should show blocking task id");
    assert!(output.contains('\u{2297}'), "should use blocked icon");
  }

  #[test]
  fn it_renders_cancelled_row_with_dim_style() {
    let theme = theme();
    let row = TaskListRow::new("cancelled", "bsylatpq", "redis cache layer", &theme);
    let output = render(&row);

    assert!(output.contains("bsylatpq"));
    assert!(output.contains("redis cache layer"));
    assert!(output.contains("cancelled"));
  }

  #[test]
  fn it_renders_done_row_with_id_and_status() {
    let theme = theme();
    let row = TaskListRow::new("done", "cdrzjvwk", "sqlite storage backend", &theme).priority(Some(0));
    let output = render(&row);

    assert!(output.contains("cdrzjvwk"), "should contain the task id");
    assert!(output.contains("[P0]"), "should contain priority badge");
    assert!(output.contains("sqlite storage backend"), "should contain title");
    assert!(output.contains("done"), "should contain status text");
  }

  #[test]
  fn it_renders_in_progress_row() {
    let theme = theme();
    let row = TaskListRow::new("in-progress", "nfkbqmrx", "openai streaming adapter", &theme).priority(Some(1));
    let output = render(&row);

    assert!(output.contains("nfkbqmrx"));
    assert!(output.contains("[P1]"));
    assert!(output.contains("in progress"));
  }

  #[test]
  fn it_reserves_priority_slot_when_padded() {
    let theme = theme();
    let with_priority = TaskListRow::new("open", "aaaaaaaa", "has priority", &theme)
      .priority(Some(1))
      .priority_pad(4);
    let without_priority = TaskListRow::new("open", "bbbbbbbb", "no priority", &theme).priority_pad(4);

    let out_with = render(&with_priority);
    let out_without = render(&without_priority);

    assert!(out_with.contains("[P1]"), "should contain priority badge");
    assert!(!out_without.contains("[P"), "should not contain priority badge");

    let title_pos_with = out_with.find("has priority").unwrap();
    let title_pos_without = out_without.find("no priority").unwrap();
    assert_eq!(
      title_pos_with, title_pos_without,
      "titles should align when priority_pad is set"
    );
  }

  #[test]
  fn it_omits_priority_column_when_pad_is_zero() {
    let theme = theme();
    let row = TaskListRow::new("open", "aaaaaaaa", "no priority", &theme);
    let output = render(&row);

    assert_eq!(row.priority_badge_width(), 0, "width should be zero with no priority");
    assert!(!output.contains("  [P"), "should not contain priority column");
  }

  #[test]
  fn it_shows_blocking_indicator() {
    let theme = theme();
    let row = TaskListRow::new("done", "hpvrlbme", "finalize probe schema v2", &theme)
      .priority(Some(0))
      .blocking(true);
    let output = render(&row);

    assert!(output.contains("blocking"), "should show blocking badge");
    assert!(output.contains('!'), "should contain blocking icon");
  }
}
