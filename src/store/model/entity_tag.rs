use chrono::{DateTime, Utc};
use getset::{CopyGetters, Getters};
use serde::{Deserialize, Serialize};

use super::primitives::{EntityType, Id};

/// A join between an entity and a tag.
#[derive(Clone, CopyGetters, Debug, Deserialize, Eq, Getters, PartialEq, Serialize)]
pub struct Model {
  /// When the association was created.
  #[get = "pub"]
  created_at: DateTime<Utc>,
  /// Identifier of the tagged entity.
  #[get = "pub"]
  entity_id: Id,
  /// Which domain type [`entity_id`](Model::entity_id) refers to.
  #[getset(get_copy = "pub")]
  entity_type: EntityType,
  /// Tag being attached.
  #[get = "pub"]
  tag_id: Id,
}

impl Model {
  /// Create a new entity-tag association with `created_at` set to now.
  pub fn new(entity_type: EntityType, entity_id: Id, tag_id: Id) -> Self {
    Self {
      created_at: Utc::now(),
      entity_id,
      entity_type,
      tag_id,
    }
  }
}
