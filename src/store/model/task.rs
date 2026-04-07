use chrono::{DateTime, Utc};
use libsql::Row;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{
  Error,
  primitives::{Id, TaskStatus},
};

/// A unit of work within a project.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Model {
  assigned_to: Option<Id>,
  created_at: DateTime<Utc>,
  description: String,
  id: Id,
  metadata: Value,
  priority: Option<u8>,
  project_id: Id,
  resolved_at: Option<DateTime<Utc>>,
  status: TaskStatus,
  title: String,
  updated_at: DateTime<Utc>,
}

impl Model {
  /// The author assigned to this task.
  pub fn assigned_to(&self) -> Option<&Id> {
    self.assigned_to.as_ref()
  }

  /// When this task was first created.
  pub fn created_at(&self) -> &DateTime<Utc> {
    &self.created_at
  }

  /// The task's description.
  pub fn description(&self) -> &str {
    &self.description
  }

  /// The unique identifier for this task.
  pub fn id(&self) -> &Id {
    &self.id
  }

  /// Custom metadata stored as JSON.
  pub fn metadata(&self) -> &Value {
    &self.metadata
  }

  /// The task's priority (0-4, lower is higher priority).
  pub fn priority(&self) -> Option<u8> {
    self.priority
  }

  /// The project this task belongs to.
  pub fn project_id(&self) -> &Id {
    &self.project_id
  }

  /// When this task was resolved (completed or cancelled).
  pub fn resolved_at(&self) -> Option<&DateTime<Utc>> {
    self.resolved_at.as_ref()
  }

  /// The task's current lifecycle status.
  pub fn status(&self) -> TaskStatus {
    self.status
  }

  /// The task's title.
  pub fn title(&self) -> &str {
    &self.title
  }

  /// When this task was last modified.
  pub fn updated_at(&self) -> &DateTime<Utc> {
    &self.updated_at
  }
}

/// Expects columns in order: `id`, `project_id`, `assigned_to`, `created_at`,
/// `description`, `metadata`, `priority`, `resolved_at`, `status`, `title`, `updated_at`.
impl TryFrom<Row> for Model {
  type Error = Error;

  fn try_from(row: Row) -> Result<Self, Self::Error> {
    let id: String = row.get(0)?;
    let project_id: String = row.get(1)?;
    let assigned_to: Option<String> = row.get(2)?;
    let created_at: String = row.get(3)?;
    let description: String = row.get(4)?;
    let metadata: String = row.get(5)?;
    let priority: Option<i64> = row.get(6)?;
    let resolved_at: Option<String> = row.get(7)?;
    let status: String = row.get(8)?;
    let title: String = row.get(9)?;
    let updated_at: String = row.get(10)?;

    let assigned_to = assigned_to
      .map(|s| s.parse::<Id>())
      .transpose()
      .map_err(Error::InvalidValue)?;
    let created_at = DateTime::parse_from_rfc3339(&created_at)
      .map(|dt| dt.with_timezone(&Utc))
      .map_err(|e| Error::InvalidValue(e.to_string()))?;
    let id: Id = id.parse().map_err(Error::InvalidValue)?;
    let metadata: Value = serde_json::from_str(&metadata).map_err(|e| Error::InvalidValue(e.to_string()))?;
    let project_id: Id = project_id.parse().map_err(Error::InvalidValue)?;
    let resolved_at = resolved_at
      .map(|s| {
        DateTime::parse_from_rfc3339(&s)
          .map(|dt| dt.with_timezone(&Utc))
          .map_err(|e| Error::InvalidValue(e.to_string()))
      })
      .transpose()?;
    let status: TaskStatus = status.parse().map_err(Error::InvalidValue)?;
    let updated_at = DateTime::parse_from_rfc3339(&updated_at)
      .map(|dt| dt.with_timezone(&Utc))
      .map_err(|e| Error::InvalidValue(e.to_string()))?;

    Ok(Self {
      assigned_to,
      created_at,
      description,
      id,
      metadata,
      priority: priority.map(|p| p as u8),
      project_id,
      resolved_at,
      status,
      title,
      updated_at,
    })
  }
}

/// Parameters for creating a new task.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct New {
  pub assigned_to: Option<Id>,
  pub description: String,
  pub metadata: Option<Value>,
  pub priority: Option<u8>,
  pub status: Option<TaskStatus>,
  pub title: String,
}

/// Optional fields for updating an existing task.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Patch {
  pub assigned_to: Option<Option<Id>>,
  pub description: Option<String>,
  pub metadata: Option<Value>,
  pub priority: Option<Option<u8>>,
  pub status: Option<TaskStatus>,
  pub title: Option<String>,
}

/// Criteria for filtering tasks.
#[derive(Clone, Debug, Default)]
pub struct Filter {
  pub all: bool,
  pub assigned_to: Option<String>,
  pub status: Option<TaskStatus>,
  pub tag: Option<String>,
}
