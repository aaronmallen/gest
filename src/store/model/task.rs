use chrono::{DateTime, Utc};
use getset::{CopyGetters, Getters};
use libsql::Row;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{
  Error,
  primitives::{Id, TaskStatus},
};

/// A unit of work within a project.
#[derive(Clone, CopyGetters, Debug, Deserialize, Eq, Getters, PartialEq, Serialize)]
pub struct Model {
  /// Author the task is assigned to, if any.
  #[get = "pub"]
  assigned_to: Option<Id>,
  /// When the task was first created.
  #[get = "pub"]
  created_at: DateTime<Utc>,
  /// Longer-form description of the work required.
  #[get = "pub"]
  description: String,
  /// Stable identifier assigned at creation.
  #[get = "pub"]
  id: Id,
  /// Free-form JSON object for user-defined metadata.
  #[get = "pub"]
  metadata: Value,
  /// Optional priority score; lower numbers rank first.
  #[getset(get_copy = "pub")]
  priority: Option<u8>,
  /// Project this task belongs to.
  #[get = "pub"]
  project_id: Id,
  /// When the task entered a terminal status, or `None` if still open.
  #[get = "pub"]
  resolved_at: Option<DateTime<Utc>>,
  /// Current lifecycle status.
  #[getset(get_copy = "pub")]
  status: TaskStatus,
  /// Short human-readable title.
  #[get = "pub"]
  title: String,
  /// When the task was last modified.
  #[get = "pub"]
  updated_at: DateTime<Utc>,
}

/// Converts a database row into a [`Model`].
///
/// Expects columns in order: `id`, `project_id`, `title`, `priority`, `status`,
/// `description`, `assigned_to`, `metadata`, `resolved_at`, `created_at`, `updated_at`.
impl TryFrom<Row> for Model {
  type Error = Error;

  fn try_from(row: Row) -> Result<Self, Self::Error> {
    let id: String = row.get(0)?;
    let project_id: String = row.get(1)?;
    let title: String = row.get(2)?;
    let priority: Option<i64> = row.get(3)?;
    let status: String = row.get(4)?;
    let description: String = row.get(5)?;
    let assigned_to: Option<String> = row.get(6)?;
    let metadata: String = row.get(7)?;
    let resolved_at: Option<String> = row.get(8)?;
    let created_at: String = row.get(9)?;
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
  /// Author the new task should be assigned to.
  pub assigned_to: Option<Id>,
  /// Longer-form description of the work required.
  pub description: String,
  /// Optional user-defined metadata merged into the new row.
  pub metadata: Option<Value>,
  /// Optional priority score; lower numbers rank first.
  pub priority: Option<u8>,
  /// Initial lifecycle status; defaults to [`TaskStatus::default`] when unset.
  pub status: Option<TaskStatus>,
  /// Short human-readable title.
  pub title: String,
}

/// Optional fields for updating an existing task.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Patch {
  /// Replacement assignee; `Some(None)` explicitly clears it.
  pub assigned_to: Option<Option<Id>>,
  /// Replacement description.
  pub description: Option<String>,
  /// Replacement metadata object; overwrites the stored value when set.
  pub metadata: Option<Value>,
  /// Replacement priority; `Some(None)` explicitly clears it.
  pub priority: Option<Option<u8>>,
  /// New lifecycle status; transitions to terminal states also set `resolved_at`.
  pub status: Option<TaskStatus>,
  /// Replacement title.
  pub title: Option<String>,
}

/// Criteria for filtering tasks.
#[derive(Clone, Debug, Default)]
pub struct Filter {
  /// Include tasks in every status, even terminal ones.
  pub all: bool,
  /// Restrict to tasks assigned to the author matching this name or id prefix.
  pub assigned_to: Option<String>,
  /// Restrict to tasks in this status.
  pub status: Option<TaskStatus>,
  /// Restrict to tasks carrying this tag label.
  pub tag: Option<String>,
}

impl Filter {
  /// Construct a filter that includes tasks in every status, including terminal ones.
  pub fn all() -> Self {
    Self {
      all: true,
      ..Self::default()
    }
  }
}
