use chrono::{DateTime, Utc};
use getset::{CopyGetters, Getters};
use libsql::Row;
use serde::{Deserialize, Serialize};

use super::{
  Error,
  primitives::{EntityType, Id},
};

/// A note attached to an entity.
#[derive(Clone, CopyGetters, Debug, Deserialize, Eq, Getters, PartialEq, Serialize)]
pub struct Model {
  #[get = "pub"]
  author_id: Option<Id>,
  #[get = "pub"]
  body: String,
  #[get = "pub"]
  created_at: DateTime<Utc>,
  #[get = "pub"]
  entity_id: Id,
  #[getset(get_copy = "pub")]
  entity_type: EntityType,
  #[get = "pub"]
  id: Id,
  #[get = "pub"]
  updated_at: DateTime<Utc>,
}

/// Converts a database row into a [`Model`].
///
/// Expects columns in order: `id`, `entity_id`, `entity_type`, `author_id`, `body`,
/// `created_at`, `updated_at`.
impl TryFrom<Row> for Model {
  type Error = Error;

  fn try_from(row: Row) -> Result<Self, Self::Error> {
    let id: String = row.get(0)?;
    let entity_id: String = row.get(1)?;
    let entity_type: String = row.get(2)?;
    let author_id: Option<String> = row.get(3)?;
    let body: String = row.get(4)?;
    let created_at: String = row.get(5)?;
    let updated_at: String = row.get(6)?;

    let author_id = author_id
      .map(|s| s.parse::<Id>())
      .transpose()
      .map_err(Error::InvalidValue)?;
    let created_at = DateTime::parse_from_rfc3339(&created_at)
      .map(|dt| dt.with_timezone(&Utc))
      .map_err(|e| Error::InvalidValue(e.to_string()))?;
    let entity_id: Id = entity_id.parse().map_err(Error::InvalidValue)?;
    let entity_type: EntityType = entity_type.parse().map_err(Error::InvalidValue)?;
    let id: Id = id.parse().map_err(Error::InvalidValue)?;
    let updated_at = DateTime::parse_from_rfc3339(&updated_at)
      .map(|dt| dt.with_timezone(&Utc))
      .map_err(|e| Error::InvalidValue(e.to_string()))?;

    Ok(Self {
      author_id,
      body,
      created_at,
      entity_id,
      entity_type,
      id,
      updated_at,
    })
  }
}

/// Parameters for creating a new note.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct New {
  pub author_id: Option<Id>,
  pub body: String,
}

/// Optional fields for updating an existing note.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Patch {
  pub body: Option<String>,
}
