use std::fmt::{self, Display, Formatter};

use crate::ui::{
  composites::{
    grouped_list::GroupedList, success_message::SuccessMessage, task_detail::TaskDetail, task_list_row::TaskListRow,
  },
  theme::Theme,
};

/// Renders a success message after creating a task.
pub struct TaskCreateView<'a> {
  /// Label-value pairs to display below the confirmation line.
  pub fields: Vec<(&'a str, String)>,
  pub id: &'a str,
  pub theme: &'a Theme,
}

impl Display for TaskCreateView<'_> {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    let mut msg = SuccessMessage::new("created task", self.theme).id(self.id);
    for (label, value) in &self.fields {
      msg = msg.field(*label, value.as_str());
    }
    write!(f, "{msg}")
  }
}

/// Renders the full detail page for a single task.
pub struct TaskDetailView<'a> {
  pub assigned: Option<&'a str>,
  /// Optional markdown body rendered as the description section.
  pub body: Option<&'a str>,
  pub id: &'a str,
  /// Relation-target pairs (e.g., `("blocked-by", "<id>")`).
  pub links: Vec<(&'a str, &'a str)>,
  /// Phase number and optional phase name.
  pub phase: Option<(u32, Option<&'a str>)>,
  pub priority: Option<u8>,
  pub status: &'a str,
  pub tags: &'a [String],
  pub theme: &'a Theme,
  pub title: &'a str,
}

impl Display for TaskDetailView<'_> {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    let detail = TaskDetail::new(self.id, self.title, self.status, self.theme)
      .priority(self.priority)
      .phase(self.phase)
      .assigned(self.assigned)
      .tags(self.tags)
      .links(self.links.clone())
      .body(self.body);

    write!(f, "{detail}")
  }
}

/// Renders a grouped list of tasks with a status-breakdown summary.
pub struct TaskListView<'a> {
  tasks: Vec<TaskViewData<'a>>,
  theme: &'a Theme,
}

impl<'a> TaskListView<'a> {
  pub fn new(tasks: Vec<TaskViewData<'a>>, theme: &'a Theme) -> Self {
    Self {
      tasks,
      theme,
    }
  }

  fn summary(&self) -> String {
    let total = self.tasks.len();
    let done = self.tasks.iter().filter(|t| t.status == "done").count();
    let in_progress = self.tasks.iter().filter(|t| t.status == "in-progress").count();
    let open = self.tasks.iter().filter(|t| t.status == "open").count();
    let cancelled = self.tasks.iter().filter(|t| t.status == "cancelled").count();

    let mut parts = vec![format!("{total} tasks")];
    if done > 0 {
      parts.push(format!("{done} done"));
    }
    if in_progress > 0 {
      parts.push(format!("{in_progress} in progress"));
    }
    if open > 0 {
      parts.push(format!("{open} open"));
    }
    if cancelled > 0 {
      parts.push(format!("{cancelled} cancelled"));
    }

    parts.join("  \u{00B7}  ")
  }
}

impl Display for TaskListView<'_> {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    let proto_rows: Vec<TaskListRow> = self
      .tasks
      .iter()
      .map(|t| {
        TaskListRow::new(t.status, t.id, t.title, self.theme)
          .priority(t.priority)
          .tags(t.tags)
          .blocking(t.is_blocking)
          .blocked_by(t.blocked_by)
      })
      .collect();

    let max_status = proto_rows.iter().map(|r| r.status_badge_width()).max().unwrap_or(0);
    let max_blocking = proto_rows.iter().map(|r| r.blocking_info_width()).max().unwrap_or(0);

    let rows: Vec<String> = proto_rows
      .into_iter()
      .map(|r| r.status_pad(max_status).blocking_pad(max_blocking).to_string())
      .collect();

    let list = GroupedList::new("tasks", self.summary(), self.theme).rows(rows);

    write!(f, "{list}")
  }
}

/// Renders a success message after updating a task.
pub struct TaskUpdateView<'a> {
  /// Label-value pairs for the changed fields.
  pub fields: Vec<(&'a str, String)>,
  pub id: &'a str,
  pub theme: &'a Theme,
}

impl Display for TaskUpdateView<'_> {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    let mut msg = SuccessMessage::new("updated task", self.theme).id(self.id);
    for (label, value) in &self.fields {
      msg = msg.field(*label, value.as_str());
    }
    write!(f, "{msg}")
  }
}

/// Data for a single row in the task list view.
pub struct TaskViewData<'a> {
  /// ID of the task that blocks this one, if any.
  pub blocked_by: Option<&'a str>,
  pub id: &'a str,
  /// Whether this task blocks other tasks.
  pub is_blocking: bool,
  pub priority: Option<u8>,
  pub status: &'a str,
  pub tags: &'a [String],
  pub title: &'a str,
}

#[cfg(test)]
mod tests {
  use super::*;

  fn theme() -> Theme {
    yansi::disable();
    Theme::default()
  }

  mod task_create_view {
    use super::*;

    mod display {
      use super::*;

      #[test]
      fn it_renders_with_fields() {
        let t = theme();
        let view = TaskCreateView {
          id: "nfkbqmrx",
          fields: vec![
            ("title", "openai streaming adapter".to_string()),
            ("status", "\u{25CB} open".to_string()),
          ],
          theme: &t,
        };
        let out = view.to_string();

        assert!(out.contains('\u{2713}'), "should contain check icon");
        assert!(out.contains("created task"), "should contain action text");
        assert!(out.contains("nf"), "should contain id prefix");
        assert!(out.contains("openai streaming adapter"), "should contain title field");
      }

      #[test]
      fn it_renders_without_fields() {
        let t = theme();
        let view = TaskCreateView {
          id: "abcd1234",
          fields: vec![],
          theme: &t,
        };
        let out = view.to_string();

        assert!(out.contains("created task"));
      }
    }
  }

  mod task_detail_view {
    use super::*;

    mod display {
      use super::*;

      #[test]
      fn it_renders_all_fields() {
        let t = theme();
        let tags = vec!["adapter".to_string()];
        let view = TaskDetailView {
          id: "nfkbqmrx",
          title: "openai streaming adapter",
          status: "in-progress",
          priority: Some(1),
          phase: Some((2, Some("core implementation"))),
          assigned: Some("claude-code"),
          tags: &tags,
          links: vec![("blocked-by", "hpvrlbme")],
          body: Some("## heading\n\nBody text."),
          theme: &t,
        };
        let out = view.to_string();

        assert!(out.contains("nf"), "should contain id prefix");
        assert!(out.contains("kbqmrx"), "should contain id rest");
        assert!(out.contains("openai streaming adapter"), "should contain title");
        assert!(out.contains("in progress"), "should contain status");
        assert!(out.contains("P1"), "should contain priority");
        assert!(out.contains("core implementation"), "should contain phase name");
        assert!(out.contains("claude-code"), "should contain assignee");
        assert!(out.contains("#adapter"), "should contain tag");
        assert!(out.contains("blocked-by"), "should contain link relation");
        assert!(out.contains("description"), "should contain body section");
      }

      #[test]
      fn it_renders_minimal() {
        let t = theme();
        let view = TaskDetailView {
          id: "abcd1234",
          title: "minimal task",
          status: "open",
          priority: None,
          phase: None,
          assigned: None,
          tags: &[],
          links: vec![],
          body: None,
          theme: &t,
        };
        let out = view.to_string();

        assert!(out.contains("minimal task"));
        assert!(!out.contains("description"), "should not contain body section");
      }
    }
  }

  mod task_list_view {
    use super::*;

    mod display {
      use super::*;

      #[test]
      fn it_omits_zero_counts_in_summary() {
        let t = theme();
        let tasks = vec![TaskViewData {
          status: "open",
          id: "aaaaaaaa",
          title: "only open",
          priority: None,
          tags: &[],
          is_blocking: false,
          blocked_by: None,
        }];
        let view = TaskListView::new(tasks, &t);
        let out = view.to_string();

        assert!(out.contains("1 tasks"), "should contain total");
        assert!(out.contains("1 open"), "should contain open count");
        assert!(!out.contains("done"), "should not contain done");
        assert!(!out.contains("in progress"), "should not contain in progress");
        assert!(!out.contains("cancelled"), "should not contain cancelled");
      }

      #[test]
      fn it_renders_blocked_task() {
        let t = theme();
        let tasks = vec![TaskViewData {
          status: "open",
          id: "mxdtqrbn",
          title: "ctx window",
          priority: None,
          tags: &[],
          is_blocking: false,
          blocked_by: Some("hpvrlbme"),
        }];
        let view = TaskListView::new(tasks, &t);
        let out = view.to_string();

        assert!(out.contains("blocked-by"), "should show blocked-by");
        assert!(out.contains("hpvrlbme"), "should show blocker id");
      }

      #[test]
      fn it_renders_empty_tasks() {
        let t = theme();
        let view = TaskListView::new(vec![], &t);
        let out = view.to_string();

        assert!(out.contains("0 tasks"), "should show zero count");
      }

      #[test]
      fn it_renders_heading_and_summary() {
        let t = theme();
        let tags = vec!["backend".to_string()];
        let tasks = vec![
          TaskViewData {
            status: "done",
            id: "cdrzjvwk",
            title: "sqlite storage backend",
            priority: Some(0),
            tags: &tags,
            is_blocking: false,
            blocked_by: None,
          },
          TaskViewData {
            status: "open",
            id: "qtsdwcaz",
            title: "probe dedup",
            priority: None,
            tags: &[],
            is_blocking: false,
            blocked_by: None,
          },
        ];
        let view = TaskListView::new(tasks, &t);
        let out = view.to_string();

        assert!(out.contains("tasks"), "should contain heading");
        assert!(out.contains("2 tasks"), "should contain total count");
        assert!(out.contains("1 done"), "should contain done count");
        assert!(out.contains("1 open"), "should contain open count");
        assert!(out.contains("cdrzjvwk"), "should contain first task id");
        assert!(out.contains("qtsdwcaz"), "should contain second task id");
      }
    }
  }

  mod task_update_view {
    use super::*;

    mod display {
      use super::*;

      #[test]
      fn it_renders_with_fields() {
        let t = theme();
        let view = TaskUpdateView {
          id: "nfkbqmrx",
          fields: vec![("status", "\u{25CF} done".to_string())],
          theme: &t,
        };
        let out = view.to_string();

        assert!(out.contains('\u{2713}'), "should contain check icon");
        assert!(out.contains("updated task"), "should contain action text");
        assert!(out.contains("nf"), "should contain id prefix");
        assert!(out.contains("done"), "should contain status field");
      }

      #[test]
      fn it_renders_without_fields() {
        let t = theme();
        let view = TaskUpdateView {
          id: "abcd1234",
          fields: vec![],
          theme: &t,
        };
        let out = view.to_string();

        assert!(out.contains("updated task"));
      }
    }
  }
}
