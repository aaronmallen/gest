use std::fmt::{self, Display, Formatter};

use yansi::{Paint, Style};

use super::super::atoms::{Badge, Id, Separator};
use crate::ui::style;

/// A single search hit showing entity type, ID, title, and status/kind.
pub struct Component {
  body: Option<String>,
  entity_type: EntityType,
  expanded: bool,
  id: String,
  status_or_kind: Option<String>,
  title: String,
}

enum EntityType {
  Artifact,
  Iteration,
  Task,
}

impl EntityType {
  fn label(&self) -> &'static str {
    match self {
      Self::Artifact => "artifact",
      Self::Iteration => "iteration",
      Self::Task => "task",
    }
  }
}

impl Component {
  /// Create a search result for an artifact.
  pub fn artifact(id: String, title: String) -> Self {
    Self {
      body: None,
      entity_type: EntityType::Artifact,
      expanded: false,
      id,
      status_or_kind: None,
      title,
    }
  }

  /// Create a search result for an iteration.
  pub fn iteration(id: String, title: String, status: String) -> Self {
    Self {
      body: None,
      entity_type: EntityType::Iteration,
      expanded: false,
      id,
      status_or_kind: Some(status),
      title,
    }
  }

  /// Create a search result for a task.
  pub fn task(id: String, title: String, status: String) -> Self {
    Self {
      body: None,
      entity_type: EntityType::Task,
      expanded: false,
      id,
      status_or_kind: Some(status),
      title,
    }
  }

  /// Set the body/description content shown in expanded mode.
  pub fn body(mut self, body: Option<String>) -> Self {
    self.body = body;
    self
  }

  /// Enable expanded rendering with full description and section separator.
  pub fn expanded(mut self, expanded: bool) -> Self {
    self.expanded = expanded;
    self
  }
}

impl Display for Component {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    let theme = style::global();

    if self.expanded {
      // Dashed separator header: "╌╌╌ entity_type id ╌╌╌"
      let label = format!("{} {}", self.entity_type.label(), self.id);
      write!(f, "  {}", Separator::dashed(label, *theme.search_expand_separator()),)?;
      writeln!(f)?;
    }

    // Entity type badge with color based on type
    let type_style = match self.entity_type {
      EntityType::Task => theme.status_in_progress(),
      EntityType::Artifact => theme.status_open(),
      EntityType::Iteration => theme.status_done(),
    };
    write!(f, "  {}", self.entity_type.label().paint(*type_style))?;

    // ID
    write!(f, "  {}", Id::new(&self.id))?;

    // Title
    write!(f, "  {}", self.title)?;

    // Status badge or kind label
    if let Some(ref value) = self.status_or_kind {
      match self.entity_type {
        EntityType::Artifact => {
          write!(f, "  {}", value.paint(*theme.search_type_label()))?;
        }
        _ => {
          let status_style = status_to_style(value);
          write!(f, "  {}", Badge::new(value, status_style))?;
        }
      }
    }

    // Body content in expanded mode — full content, no truncation
    if self.expanded
      && let Some(ref body) = self.body
    {
      writeln!(f)?;
      writeln!(f)?;
      for line in body.lines() {
        writeln!(f, "    {}", line.paint(*theme.muted()))?;
      }
    }

    Ok(())
  }
}

fn status_to_style(status: &str) -> Style {
  let theme = style::global();
  match status {
    "cancelled" => *theme.status_cancelled(),
    "done" => *theme.status_done(),
    "in_progress" | "in-progress" => *theme.status_in_progress(),
    _ => *theme.status_open(),
  }
}
