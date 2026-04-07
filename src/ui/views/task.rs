use std::fmt::{self, Display, Formatter};

use chrono::{DateTime, Utc};
use yansi::Paint;

use crate::{
  model::{
    Note,
    event::{Event, EventKind},
  },
  ui::{
    atoms::{id::Id, label::Label, separator::Separator, value::Value},
    composites::{
      grouped_list::GroupedList, status_badge::StatusBadge, success_message::SuccessMessage, task_detail::TaskDetail,
      task_list_row::TaskListRow,
    },
    markdown,
    theming::theme::Theme,
    utils,
  },
};

/// Renders a success message after creating a task.
pub struct TaskCreateView<'a> {
  /// Label-value pairs to display below the confirmation line.
  pub fields: Vec<(&'a str, String)>,
  pub id: &'a str,
  /// The task status string for the status badge.
  pub status: &'a str,
  pub theme: &'a Theme,
}

impl Display for TaskCreateView<'_> {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    let mut msg = SuccessMessage::new("created task", self.theme).id(self.id);
    for (label, value) in &self.fields {
      msg = msg.field(*label, value.as_str());
    }
    msg = msg.styled_field("status", StatusBadge::new(self.status, self.theme));
    write!(f, "{msg}")
  }
}

/// Renders the full detail page for a single task.
pub struct TaskDetailView<'a> {
  pub assigned: Option<&'a str>,
  /// Optional markdown body rendered as the description section.
  pub body: Option<&'a str>,
  /// Events attached to this task.
  pub events: &'a [Event],
  pub id: &'a str,
  /// Relation-target pairs (e.g., `("blocked-by", "<id>")`).
  pub links: Vec<(&'a str, &'a str)>,
  /// Notes attached to this task.
  pub notes: &'a [Note],
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

    write!(f, "{detail}")?;

    let timeline = build_timeline(self.notes, self.events);

    if !timeline.is_empty() {
      writeln!(f)?;
      writeln!(f)?;
      let sep = Separator::labeled("activity", self.theme.task_detail_separator);
      writeln!(f, "  {sep}")?;

      let max_label = 7;
      let width = utils::terminal_width() as usize;
      for (i, entry) in timeline.iter().enumerate() {
        writeln!(f)?;
        match entry {
          TimelineEntry::Note(note) => {
            let short_id = note.id.short();
            let id_atom = Id::new(&short_id, self.theme);
            writeln!(f, "  {id_atom}")?;

            let author_display = format_note_author(note);
            let label = Label::new("author", self.theme.task_detail_label).pad_to(max_label);
            let val = Value::new(&author_display, self.theme.task_detail_value);
            writeln!(f, "    {label}  {val}")?;

            let created = note.created_at.format("%Y-%m-%d %H:%M").to_string();
            let label = Label::new("created", self.theme.task_detail_label).pad_to(max_label);
            let val = Value::new(&created, self.theme.task_detail_value);
            writeln!(f, "    {label}  {val}")?;

            let rendered = markdown::render(&note.body);
            for line in rendered.lines() {
              writeln!(f, "    {line}")?;
            }
          }
          TimelineEntry::Event(event) => {
            let description = format_event_description(event);
            let dimmed = format!("{}", description.dim());
            writeln!(f, "    {dimmed}")?;

            let author_display = format_event_author(event);
            let created = event.created_at.format("%Y-%m-%d %H:%M").to_string();
            let meta = format!("{}", format!("{author_display}  {created}").dim());
            writeln!(f, "    {meta}")?;
          }
        }

        if i < timeline.len() - 1 {
          writeln!(f)?;
        }
      }

      writeln!(f)?;
      let rule = Separator::rule(self.theme.task_detail_separator);
      write!(f, "  {rule}")?;
    }

    Ok(())
  }
}

/// Renders a grouped list of tasks with a status-breakdown summary.
pub struct TaskListView<'a> {
  tasks: Vec<TaskViewData>,
  theme: &'a Theme,
}

impl<'a> TaskListView<'a> {
  pub fn new(tasks: Vec<TaskViewData>, theme: &'a Theme) -> Self {
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
        TaskListRow::new(&t.status, &t.id, &t.title, self.theme)
          .priority(t.priority)
          .tags(&t.tags)
          .blocking(t.is_blocking)
          .blocked_by(t.blocked_by.as_deref())
      })
      .collect();

    let max_priority = proto_rows.iter().map(|r| r.priority_badge_width()).max().unwrap_or(0);
    let max_status = proto_rows.iter().map(|r| r.status_badge_width()).max().unwrap_or(0);
    let max_blocking = proto_rows.iter().map(|r| r.blocking_info_width()).max().unwrap_or(0);

    let rows: Vec<String> = proto_rows
      .into_iter()
      .map(|r| {
        r.priority_pad(max_priority)
          .status_pad(max_status)
          .blocking_pad(max_blocking)
          .to_string()
      })
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
  /// The task status string, shown as a badge when present.
  pub status: Option<&'a str>,
  pub theme: &'a Theme,
}

impl Display for TaskUpdateView<'_> {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    let mut msg = SuccessMessage::new("updated task", self.theme).id(self.id);
    if let Some(status) = self.status {
      msg = msg.styled_field("status", StatusBadge::new(status, self.theme));
    }
    for (label, value) in &self.fields {
      msg = msg.field(*label, value.as_str());
    }
    write!(f, "{msg}")
  }
}

/// Data for a single row in the task list view.
pub struct TaskViewData {
  /// ID of the task that blocks this one, if any.
  pub blocked_by: Option<String>,
  pub id: String,
  /// Whether this task blocks other tasks.
  pub is_blocking: bool,
  pub priority: Option<u8>,
  pub status: String,
  pub tags: Vec<String>,
  pub title: String,
}

enum TimelineEntry<'a> {
  Event(&'a Event),
  Note(&'a Note),
}

impl TimelineEntry<'_> {
  fn created_at(&self) -> DateTime<Utc> {
    match self {
      Self::Event(e) => e.created_at,
      Self::Note(n) => n.created_at,
    }
  }
}

fn build_timeline<'a>(notes: &'a [Note], events: &'a [Event]) -> Vec<TimelineEntry<'a>> {
  let mut timeline: Vec<TimelineEntry<'a>> = Vec::with_capacity(notes.len() + events.len());
  timeline.extend(notes.iter().map(TimelineEntry::Note));
  timeline.extend(events.iter().map(TimelineEntry::Event));
  timeline.sort_by_key(|e| e.created_at());
  timeline
}

fn format_event_author(event: &Event) -> String {
  use crate::model::note::AuthorType;
  match event.author_type {
    AuthorType::Agent => format!("{} (agent)", event.author),
    AuthorType::Human => match &event.author_email {
      Some(email) => format!("{} <{}>", event.author, email),
      None => event.author.clone(),
    },
  }
}

fn format_event_description(event: &Event) -> String {
  match &event.kind {
    EventKind::PhaseChange {
      from,
      to,
    } => {
      let from_str = from.map_or("none".to_string(), |v| v.to_string());
      let to_str = to.map_or("none".to_string(), |v| v.to_string());
      format!("phase changed from {from_str} to {to_str}")
    }
    EventKind::PriorityChange {
      from,
      to,
    } => {
      let from_str = from.map_or("none".to_string(), |v| format!("P{v}"));
      let to_str = to.map_or("none".to_string(), |v| format!("P{v}"));
      format!("priority changed from {from_str} to {to_str}")
    }
    EventKind::StatusChange {
      from,
      to,
    } => {
      format!("status changed from {from} to {to}")
    }
  }
}

fn format_note_author(note: &Note) -> String {
  use crate::model::note::AuthorType;
  match note.author_type {
    AuthorType::Agent => format!("{} (agent)", note.author),
    AuthorType::Human => match &note.author_email {
      Some(email) => format!("{} <{}>", note.author, email),
      None => note.author.clone(),
    },
  }
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
          fields: vec![("title", "openai streaming adapter".to_string())],
          status: "open",
          theme: &t,
        };
        let out = view.to_string();

        assert!(out.contains('\u{2713}'), "should contain check icon");
        assert!(out.contains("created task"), "should contain action text");
        assert!(out.contains("nf"), "should contain id prefix");
        assert!(out.contains("openai streaming adapter"), "should contain title field");
        assert!(out.contains('\u{25CB}'), "should contain status icon");
        assert!(out.contains("open"), "should contain status text");
      }

      #[test]
      fn it_renders_without_fields() {
        let t = theme();
        let view = TaskCreateView {
          id: "abcd1234",
          fields: vec![],
          status: "open",
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
      fn it_omits_activity_section_when_empty() {
        let t = theme();
        let view = TaskDetailView {
          id: "abcd1234",
          title: "test task",
          status: "open",
          priority: None,
          phase: None,
          assigned: None,
          tags: &[],
          links: vec![],
          events: &[],
          notes: &[],
          body: None,
          theme: &t,
        };
        let out = view.to_string();

        assert!(
          !out.contains("activity"),
          "should not contain activity section when empty"
        );
      }

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
          events: &[],
          notes: &[],
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
      fn it_renders_event_in_activity_section() {
        use crate::model::{event::EventKind, note::AuthorType};

        let t = theme();
        let now = chrono::Utc::now();
        let events = vec![Event {
          author: "alice".to_string(),
          author_email: None,
          author_type: AuthorType::Human,
          created_at: now,
          description: None,
          id: "zyxwvutsrqponmlkzyxwvutsrqponmlk".parse().unwrap(),
          kind: EventKind::StatusChange {
            from: "open".to_string(),
            to: "in-progress".to_string(),
          },
        }];
        let view = TaskDetailView {
          id: "abcd1234",
          title: "test task",
          status: "in-progress",
          priority: None,
          phase: None,
          assigned: None,
          tags: &[],
          links: vec![],
          events: &events,
          notes: &[],
          body: None,
          theme: &t,
        };
        let out = view.to_string();

        assert!(out.contains("activity"), "should contain activity section header");
        assert!(
          out.contains("status changed from open to in-progress"),
          "should contain event description"
        );
        assert!(out.contains("alice"), "should contain event author");
      }

      #[test]
      fn it_renders_merged_timeline_sorted_by_created_at() {
        use crate::model::{event::EventKind, note::AuthorType};

        let t = theme();
        let now = chrono::Utc::now();
        let earlier = now - chrono::Duration::hours(1);

        let events = vec![Event {
          author: "bot".to_string(),
          author_email: None,
          author_type: AuthorType::Agent,
          created_at: earlier,
          description: None,
          id: "zyxwvutsrqponmlkzyxwvutsrqponmlk".parse().unwrap(),
          kind: EventKind::StatusChange {
            from: "open".to_string(),
            to: "in-progress".to_string(),
          },
        }];
        let notes = vec![Note {
          author: "bob".to_string(),
          author_email: None,
          author_type: AuthorType::Human,
          body: "Starting work on this.".to_string(),
          created_at: now,
          id: "kkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkk".parse().unwrap(),
          updated_at: now,
        }];
        let view = TaskDetailView {
          id: "abcd1234",
          title: "test task",
          status: "in-progress",
          priority: None,
          phase: None,
          assigned: None,
          tags: &[],
          links: vec![],
          events: &events,
          notes: &notes,
          body: None,
          theme: &t,
        };
        let out = view.to_string();

        let event_pos = out.find("status changed").unwrap();
        let note_pos = out.find("Starting work").unwrap();
        assert!(event_pos < note_pos, "event (earlier) should appear before note (now)");
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
          events: &[],
          notes: &[],
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
          status: "open".into(),
          id: "aaaaaaaa".into(),
          title: "only open".into(),
          priority: None,
          tags: vec![],
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
          status: "open".into(),
          id: "mxdtqrbn".into(),
          title: "ctx window".into(),
          priority: None,
          tags: vec![],
          is_blocking: false,
          blocked_by: Some("hpvrlbme".into()),
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
        let tasks = vec![
          TaskViewData {
            status: "done".into(),
            id: "cdrzjvwk".into(),
            title: "sqlite storage backend".into(),
            priority: Some(0),
            tags: vec!["backend".into()],
            is_blocking: false,
            blocked_by: None,
          },
          TaskViewData {
            status: "open".into(),
            id: "qtsdwcaz".into(),
            title: "probe dedup".into(),
            priority: None,
            tags: vec![],
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
      fn it_renders_with_status() {
        let t = theme();
        let view = TaskUpdateView {
          id: "nfkbqmrx",
          fields: vec![],
          status: Some("done"),
          theme: &t,
        };
        let out = view.to_string();

        assert!(out.contains('\u{2713}'), "should contain check icon");
        assert!(out.contains("updated task"), "should contain action text");
        assert!(out.contains("nf"), "should contain id prefix");
        assert!(out.contains('\u{25CF}'), "should contain done icon");
        assert!(out.contains("done"), "should contain status text");
      }

      #[test]
      fn it_renders_without_fields() {
        let t = theme();
        let view = TaskUpdateView {
          id: "abcd1234",
          fields: vec![],
          status: None,
          theme: &t,
        };
        let out = view.to_string();

        assert!(out.contains("updated task"));
      }
    }
  }
}
