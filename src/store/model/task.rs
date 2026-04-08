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
  #[get = "pub"]
  assigned_to: Option<Id>,
  #[get = "pub"]
  created_at: DateTime<Utc>,
  #[get = "pub"]
  description: String,
  #[get = "pub"]
  id: Id,
  #[get = "pub"]
  metadata: Value,
  #[getset(get_copy = "pub")]
  priority: Option<u8>,
  #[get = "pub"]
  project_id: Id,
  #[get = "pub"]
  resolved_at: Option<DateTime<Utc>>,
  #[getset(get_copy = "pub")]
  status: TaskStatus,
  #[get = "pub"]
  title: String,
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

impl Filter {
  /// Construct a filter that includes tasks in every status, including terminal ones.
  pub fn all() -> Self {
    Self {
      all: true,
      ..Self::default()
    }
  }
}
