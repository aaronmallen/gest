//! Unified activity timeline combining notes and semantic transaction events.
//!
//! Provides a tagged [`TimelineItem`] enum used by the task/artifact/iteration
//! detail templates, plus a builder that merges notes and semantic events for a
//! given entity and sorts them chronologically.

use chrono::{DateTime, Utc};
use libsql::Connection;

use crate::{
  store::{
    model::{
      note,
      primitives::{AuthorType, EntityType, Id},
    },
    repo::{self, transaction::SemanticEvent},
  },
  web::{gravatar, markdown},
};

/// A display-ready event entry for the activity timeline.
///
/// Fields are read by askama templates which the dead-code analysis can't see.
#[allow(dead_code)]
pub(crate) struct EventDisplay {
  pub(crate) author_gravatar: Option<String>,
  pub(crate) author_is_agent: bool,
  pub(crate) author_name: Option<String>,
  pub(crate) created_at: DateTime<Utc>,
  pub(crate) created_at_display: String,
  pub(crate) display_text: String,
  pub(crate) id_short: String,
}

/// A display-ready note entry for the activity timeline.
pub(crate) struct NoteItem {
  pub(crate) author_gravatar: Option<String>,
  pub(crate) author_is_agent: bool,
  pub(crate) author_name: Option<String>,
  pub(crate) body_html: String,
  pub(crate) created_at: DateTime<Utc>,
  pub(crate) created_at_display: String,
  pub(crate) id_short: String,
}

/// A single entry in the activity timeline: either a note or a semantic event.
pub(crate) enum TimelineItem {
  Event(EventDisplay),
  Note(NoteItem),
}

impl TimelineItem {
  /// Template helper: return the event variant if present (for askama `if let`).
  pub(crate) fn as_event(&self) -> Option<&EventDisplay> {
    match self {
      Self::Event(e) => Some(e),
      Self::Note(_) => None,
    }
  }

  /// Template helper: return the note variant if present.
  pub(crate) fn as_note(&self) -> Option<&NoteItem> {
    match self {
      Self::Note(n) => Some(n),
      Self::Event(_) => None,
    }
  }

  /// Return the creation timestamp used for chronological sorting.
  fn sort_key(&self) -> DateTime<Utc> {
    match self {
      Self::Event(e) => e.created_at,
      Self::Note(n) => n.created_at,
    }
  }
}

/// Build the unified activity timeline for an entity.
///
/// Merges notes and semantic transaction events (filtering out events whose
/// `semantic_type IS NULL`), resolves author display data, and returns items
/// sorted by `created_at` ascending.
pub(crate) async fn build_timeline(
  conn: &Connection,
  entity_type: EntityType,
  entity_id: &Id,
) -> Result<Vec<TimelineItem>, String> {
  let notes = repo::note::for_entity(conn, entity_type, entity_id)
    .await
    .map_err(|e| e.to_string())?;
  let events = repo::transaction::semantic_events_for_row(conn, entity_table(entity_type), entity_id)
    .await
    .map_err(|e| e.to_string())?;

  let mut items: Vec<TimelineItem> = Vec::with_capacity(notes.len() + events.len());

  for n in notes {
    items.push(TimelineItem::Note(build_note_item(conn, n).await));
  }
  for e in events {
    items.push(TimelineItem::Event(build_event_display(conn, e).await));
  }

  items.sort_by_key(|item| item.sort_key());
  Ok(items)
}

/// Render the human-readable action text for a semantic event.
pub(crate) fn render_event_text(event: &SemanticEvent, author_name: Option<&str>) -> String {
  let who = author_name.unwrap_or("someone");
  let action = match event.semantic_type.as_str() {
    "archived" => "archived this".to_string(),
    "cancelled" => "cancelled this".to_string(),
    "completed" => "completed this".to_string(),
    "created" => "created this".to_string(),
    "phase-change" => match (&event.old_value, &event.new_value) {
      (Some(old), Some(new)) => format!("moved from phase {old} to phase {new}"),
      (None, Some(new)) => format!("moved to phase {new}"),
      _ => "changed phase".to_string(),
    },
    "priority-change" => match (&event.old_value, &event.new_value) {
      (Some(old), Some(new)) => format!("changed priority from P{old} to P{new}"),
      (None, Some(new)) => format!("set priority to P{new}"),
      (Some(old), None) => format!("cleared priority (was P{old})"),
      _ => "changed priority".to_string(),
    },
    "status-change" => match (&event.old_value, &event.new_value) {
      (Some(old), Some(new)) => format!("changed status from {old} to {new}"),
      (None, Some(new)) => format!("set status to {new}"),
      _ => "changed status".to_string(),
    },
    other => format!("performed {other}"),
  };
  format!("{who} {action}")
}

/// Build an [`EventDisplay`] for a single semantic event.
async fn build_event_display(conn: &Connection, event: SemanticEvent) -> EventDisplay {
  let (author_name, author_gravatar, author_is_agent) = resolve_author(conn, event.author_id.as_ref()).await;
  let display_text = render_event_text(&event, author_name.as_deref());

  EventDisplay {
    author_gravatar,
    author_is_agent,
    author_name,
    created_at: event.created_at,
    created_at_display: event.created_at.format("%Y-%m-%d %H:%M UTC").to_string(),
    display_text,
    id_short: event.id.short(),
  }
}

/// Build a [`NoteItem`] for a single note.
async fn build_note_item(conn: &Connection, note: note::Model) -> NoteItem {
  let (author_name, author_gravatar, author_is_agent) = resolve_author(conn, note.author_id().as_ref()).await;

  NoteItem {
    author_gravatar,
    author_is_agent,
    author_name,
    body_html: markdown::render_markdown_to_html(note.body()),
    created_at: *note.created_at(),
    created_at_display: note.created_at().format("%Y-%m-%d %H:%M UTC").to_string(),
    id_short: note.id().short(),
  }
}

/// Map an [`EntityType`] to the transaction_events `table_name` value.
fn entity_table(entity_type: EntityType) -> &'static str {
  match entity_type {
    EntityType::Artifact => "artifacts",
    EntityType::Iteration => "iterations",
    EntityType::Task => "tasks",
  }
}

/// Resolve an author by id into display fields, or empty values when missing.
async fn resolve_author(conn: &Connection, author_id: Option<&Id>) -> (Option<String>, Option<String>, bool) {
  match author_id {
    Some(aid) => match repo::author::find_by_id(conn, aid.clone()).await {
      Ok(Some(author)) => (
        Some(author.name().clone()),
        gravatar::url(author.email().as_deref()),
        author.author_type() == AuthorType::Agent,
      ),
      _ => (None, None, false),
    },
    None => (None, None, false),
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  mod render_event_text_fn {
    use pretty_assertions::assert_eq;

    use super::*;

    fn event(semantic_type: &str, old: Option<&str>, new: Option<&str>) -> SemanticEvent {
      SemanticEvent {
        author_id: None,
        created_at: Utc::now(),
        id: "kkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkk".parse().unwrap(),
        new_value: new.map(str::to_string),
        old_value: old.map(str::to_string),
        row_id: "row".into(),
        semantic_type: semantic_type.into(),
        table_name: "tasks".into(),
      }
    }

    #[test]
    fn it_falls_back_to_someone_when_author_is_missing() {
      let text = render_event_text(&event("completed", None, None), None);

      assert_eq!(text, "someone completed this");
    }

    #[test]
    fn it_renders_archived() {
      let text = render_event_text(&event("archived", None, None), Some("Aaron Allen"));

      assert_eq!(text, "Aaron Allen archived this");
    }

    #[test]
    fn it_renders_created_with_author() {
      let text = render_event_text(&event("created", None, None), Some("Aaron Allen"));

      assert_eq!(text, "Aaron Allen created this");
    }

    #[test]
    fn it_renders_phase_change_between_phases() {
      let text = render_event_text(&event("phase-change", Some("1"), Some("2")), Some("Aaron Allen"));

      assert_eq!(text, "Aaron Allen moved from phase 1 to phase 2");
    }

    #[test]
    fn it_renders_priority_change_when_only_new_value_is_set() {
      let text = render_event_text(&event("priority-change", None, Some("1")), Some("Aaron Allen"));

      assert_eq!(text, "Aaron Allen set priority to P1");
    }

    #[test]
    fn it_renders_priority_change_with_values() {
      let text = render_event_text(&event("priority-change", Some("2"), Some("1")), Some("Aaron Allen"));

      assert_eq!(text, "Aaron Allen changed priority from P2 to P1");
    }

    #[test]
    fn it_renders_status_change_with_old_and_new() {
      let text = render_event_text(
        &event("status-change", Some("open"), Some("in-progress")),
        Some("Aaron Allen"),
      );

      assert_eq!(text, "Aaron Allen changed status from open to in-progress");
    }
  }
}
