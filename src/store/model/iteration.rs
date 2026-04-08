use chrono::{DateTime, Utc};
use getset::{CopyGetters, Getters};
use libsql::Row;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{
  Error,
  primitives::{Id, IterationStatus},
};

/// A time-boxed collection of tasks within a project.
#[derive(Clone, CopyGetters, Debug, Deserialize, Eq, Getters, PartialEq, Serialize)]
pub struct Model {
  #[get = "pub"]
  completed_at: Option<DateTime<Utc>>,
  #[get = "pub"]
  created_at: DateTime<Utc>,
  #[get = "pub"]
  description: String,
  #[get = "pub"]
  id: Id,
  #[get = "pub"]
  metadata: Value,
  #[get = "pub"]
  project_id: Id,
  #[getset(get_copy = "pub")]
  status: IterationStatus,
  #[get = "pub"]
  title: String,
  #[get = "pub"]
  updated_at: DateTime<Utc>,
}

/// Converts a database row into a [`Model`].
///
/// Expects columns in order: `id`, `project_id`, `title`, `status`, `description`,
/// `metadata`, `completed_at`, `created_at`, `updated_at`.
impl TryFrom<Row> for Model {
  type Error = Error;

  fn try_from(row: Row) -> Result<Self, Self::Error> {
    let id: String = row.get(0)?;
    let project_id: String = row.get(1)?;
    let title: String = row.get(2)?;
    let status: String = row.get(3)?;
    let description: String = row.get(4)?;
    let metadata: String = row.get(5)?;
    let completed_at: Option<String> = row.get(6)?;
    let created_at: String = row.get(7)?;
    let updated_at: String = row.get(8)?;

    let completed_at = completed_at
      .map(|s| {
        DateTime::parse_from_rfc3339(&s)
          .map(|dt| dt.with_timezone(&Utc))
          .map_err(|e| Error::InvalidValue(e.to_string()))
      })
      .transpose()?;
    let created_at = DateTime::parse_from_rfc3339(&created_at)
      .map(|dt| dt.with_timezone(&Utc))
      .map_err(|e| Error::InvalidValue(e.to_string()))?;
    let id: Id = id.parse().map_err(Error::InvalidValue)?;
    let metadata: Value = serde_json::from_str(&metadata).map_err(|e| Error::InvalidValue(e.to_string()))?;
    let project_id: Id = project_id.parse().map_err(Error::InvalidValue)?;
    let status: IterationStatus = status.parse().map_err(Error::InvalidValue)?;
    let updated_at = DateTime::parse_from_rfc3339(&updated_at)
      .map(|dt| dt.with_timezone(&Utc))
      .map_err(|e| Error::InvalidValue(e.to_string()))?;

    Ok(Self {
      completed_at,
      created_at,
      description,
      id,
      metadata,
      project_id,
      status,
      title,
      updated_at,
    })
  }
}

/// Parameters for creating a new iteration.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct New {
  pub description: String,
  pub metadata: Option<Value>,
  pub title: String,
}

/// Optional fields for updating an existing iteration.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Patch {
  pub description: Option<String>,
  pub metadata: Option<Value>,
  pub status: Option<IterationStatus>,
  pub title: Option<String>,
}

/// Criteria for filtering iterations.
#[derive(Clone, Debug, Default)]
pub struct Filter {
  pub all: bool,
  pub has_available: bool,
  pub status: Option<IterationStatus>,
  pub tag: Option<String>,
}

impl Filter {
  /// Construct a filter that includes iterations in every status, including terminal ones.
  pub fn all() -> Self {
    Self {
      all: true,
      ..Self::default()
    }
  }
}
