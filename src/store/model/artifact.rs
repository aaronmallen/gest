use chrono::{DateTime, Utc};
use getset::Getters;
use libsql::Row;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{Error, primitives::Id};

/// A persistent document (spec, ADR, design doc, etc.) within a project.
#[derive(Clone, Debug, Deserialize, Eq, Getters, PartialEq, Serialize)]
pub struct Model {
  #[get = "pub"]
  archived_at: Option<DateTime<Utc>>,
  #[serde(skip)]
  #[get = "pub"]
  body: String,
  #[get = "pub"]
  created_at: DateTime<Utc>,
  #[get = "pub"]
  id: Id,
  #[get = "pub"]
  metadata: Value,
  #[get = "pub"]
  project_id: Id,
  #[get = "pub"]
  title: String,
  #[get = "pub"]
  updated_at: DateTime<Utc>,
}

impl Model {
  /// Whether this artifact is archived.
  pub fn is_archived(&self) -> bool {
    self.archived_at.is_some()
  }
}

/// Converts a database row into a [`Model`].
///
/// Expects columns in order: `id`, `project_id`, `title`, `body`, `metadata`,
/// `archived_at`, `created_at`, `updated_at`.
impl TryFrom<Row> for Model {
  type Error = Error;

  fn try_from(row: Row) -> Result<Self, Self::Error> {
    let id: String = row.get(0)?;
    let project_id: String = row.get(1)?;
    let title: String = row.get(2)?;
    let body: String = row.get(3)?;
    let metadata: String = row.get(4)?;
    let archived_at: Option<String> = row.get(5)?;
    let created_at: String = row.get(6)?;
    let updated_at: String = row.get(7)?;

    let archived_at = archived_at
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
    let updated_at = DateTime::parse_from_rfc3339(&updated_at)
      .map(|dt| dt.with_timezone(&Utc))
      .map_err(|e| Error::InvalidValue(e.to_string()))?;

    Ok(Self {
      archived_at,
      body,
      created_at,
      id,
      metadata,
      project_id,
      title,
      updated_at,
    })
  }
}

/// Parameters for creating a new artifact.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct New {
  pub body: String,
  pub metadata: Option<Value>,
  pub title: String,
}

/// Optional fields for updating an existing artifact.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Patch {
  pub body: Option<String>,
  pub metadata: Option<Value>,
  pub title: Option<String>,
}

/// Criteria for filtering artifacts.
#[derive(Clone, Debug, Default)]
pub struct Filter {
  pub all: bool,
  pub only_archived: bool,
  pub tag: Option<String>,
}
