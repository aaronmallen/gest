use chrono::{DateTime, Utc};
use getset::{CopyGetters, Getters};
use libsql::Row;
use serde::{Deserialize, Serialize};

use super::{
  Error,
  primitives::{EntityType, Id, RelationshipType},
};

/// A directed relationship between two entities.
#[derive(Clone, CopyGetters, Debug, Deserialize, Eq, Getters, PartialEq, Serialize)]
pub struct Model {
  #[get = "pub"]
  created_at: DateTime<Utc>,
  #[get = "pub"]
  id: Id,
  #[getset(get_copy = "pub")]
  rel_type: RelationshipType,
  #[get = "pub"]
  source_id: Id,
  #[getset(get_copy = "pub")]
  source_type: EntityType,
  #[get = "pub"]
  target_id: Id,
  #[getset(get_copy = "pub")]
  target_type: EntityType,
  #[get = "pub"]
  updated_at: DateTime<Utc>,
}

/// Converts a database row into a [`Model`].
///
/// Expects columns in order: `id`, `rel_type`, `source_id`, `source_type`,
/// `target_id`, `target_type`, `created_at`, `updated_at`.
impl TryFrom<Row> for Model {
  type Error = Error;

  fn try_from(row: Row) -> Result<Self, Self::Error> {
    let id: String = row.get(0)?;
    let rel_type: String = row.get(1)?;
    let source_id: String = row.get(2)?;
    let source_type: String = row.get(3)?;
    let target_id: String = row.get(4)?;
    let target_type: String = row.get(5)?;
    let created_at: String = row.get(6)?;
    let updated_at: String = row.get(7)?;

    let id: Id = id.parse().map_err(Error::InvalidValue)?;
    let rel_type: RelationshipType = rel_type.parse().map_err(Error::InvalidValue)?;
    let source_id: Id = source_id.parse().map_err(Error::InvalidValue)?;
    let source_type: EntityType = source_type.parse().map_err(Error::InvalidValue)?;
    let target_id: Id = target_id.parse().map_err(Error::InvalidValue)?;
    let target_type: EntityType = target_type.parse().map_err(Error::InvalidValue)?;
    let created_at = DateTime::parse_from_rfc3339(&created_at)
      .map(|dt| dt.with_timezone(&Utc))
      .map_err(|e| Error::InvalidValue(e.to_string()))?;
    let updated_at = DateTime::parse_from_rfc3339(&updated_at)
      .map(|dt| dt.with_timezone(&Utc))
      .map_err(|e| Error::InvalidValue(e.to_string()))?;

    Ok(Self {
      created_at,
      id,
      rel_type,
      source_id,
      source_type,
      target_id,
      target_type,
      updated_at,
    })
  }
}
