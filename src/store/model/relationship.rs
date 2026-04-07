use libsql::Row;
use serde::{Deserialize, Serialize};

use super::{
  Error,
  primitives::{EntityType, Id, RelationshipType},
};

/// A directed relationship between two entities.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Model {
  id: Id,
  rel_type: RelationshipType,
  source_id: Id,
  source_type: EntityType,
  target_id: Id,
  target_type: EntityType,
}

impl Model {
  /// The unique identifier for this relationship.
  pub fn id(&self) -> &Id {
    &self.id
  }

  /// The type of relationship.
  pub fn rel_type(&self) -> RelationshipType {
    self.rel_type
  }

  /// The source entity's ID.
  pub fn source_id(&self) -> &Id {
    &self.source_id
  }

  /// The source entity's type.
  pub fn source_type(&self) -> EntityType {
    self.source_type
  }

  /// The target entity's ID.
  pub fn target_id(&self) -> &Id {
    &self.target_id
  }

  /// The target entity's type.
  pub fn target_type(&self) -> EntityType {
    self.target_type
  }
}

/// Expects columns in order: `id`, `rel_type`, `source_id`, `source_type`,
/// `target_id`, `target_type`.
impl TryFrom<Row> for Model {
  type Error = Error;

  fn try_from(row: Row) -> Result<Self, Self::Error> {
    let id: String = row.get(0)?;
    let rel_type: String = row.get(1)?;
    let source_id: String = row.get(2)?;
    let source_type: String = row.get(3)?;
    let target_id: String = row.get(4)?;
    let target_type: String = row.get(5)?;

    let id: Id = id.parse().map_err(Error::InvalidValue)?;
    let rel_type: RelationshipType = rel_type.parse().map_err(Error::InvalidValue)?;
    let source_id: Id = source_id.parse().map_err(Error::InvalidValue)?;
    let source_type: EntityType = source_type.parse().map_err(Error::InvalidValue)?;
    let target_id: Id = target_id.parse().map_err(Error::InvalidValue)?;
    let target_type: EntityType = target_type.parse().map_err(Error::InvalidValue)?;

    Ok(Self {
      id,
      rel_type,
      source_id,
      source_type,
      target_id,
      target_type,
    })
  }
}
