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
  /// When the relationship was first recorded.
  #[get = "pub"]
  created_at: DateTime<Utc>,
  /// Stable identifier assigned at creation.
  #[get = "pub"]
  id: Id,
  /// Semantic kind of relationship (e.g. blocks, parent-of).
  #[getset(get_copy = "pub")]
  rel_type: RelationshipType,
  /// Identifier of the source (from-side) entity.
  #[get = "pub"]
  source_id: Id,
  /// Which domain type [`source_id`](Model::source_id) refers to.
  #[getset(get_copy = "pub")]
  source_type: EntityType,
  /// Identifier of the target (to-side) entity.
  #[get = "pub"]
  target_id: Id,
  /// Which domain type [`target_id`](Model::target_id) refers to.
  #[getset(get_copy = "pub")]
  target_type: EntityType,
  /// When the relationship row was last modified.
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
