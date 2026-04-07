use chrono::{DateTime, Utc};
use libsql::Row;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{
  Error,
  primitives::{EntityType, EventKind, Id},
};

/// An audit trail entry recording a change to an entity.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Model {
  author_id: Option<Id>,
  created_at: DateTime<Utc>,
  data: Value,
  description: Option<String>,
  entity_id: Id,
  entity_type: EntityType,
  event_type: EventKind,
  id: Id,
}

impl Model {
  /// The author who caused this event.
  pub fn author_id(&self) -> Option<&Id> {
    self.author_id.as_ref()
  }

  /// When this event occurred.
  pub fn created_at(&self) -> &DateTime<Utc> {
    &self.created_at
  }

  /// Structured data associated with this event (e.g. `{from, to}` for changes).
  pub fn data(&self) -> &Value {
    &self.data
  }

  /// Optional human-readable description of the event.
  pub fn description(&self) -> Option<&str> {
    self.description.as_deref()
  }

  /// The entity this event is associated with.
  pub fn entity_id(&self) -> &Id {
    &self.entity_id
  }

  /// The type of entity this event is associated with.
  pub fn entity_type(&self) -> EntityType {
    self.entity_type
  }

  /// The kind of event.
  pub fn event_type(&self) -> EventKind {
    self.event_type
  }

  /// The unique identifier for this event.
  pub fn id(&self) -> &Id {
    &self.id
  }
}

/// Expects columns in order: `id`, `entity_id`, `entity_type`, `author_id`,
/// `created_at`, `data`, `description`, `event_type`.
impl TryFrom<Row> for Model {
  type Error = Error;

  fn try_from(row: Row) -> Result<Self, Self::Error> {
    let id: String = row.get(0)?;
    let entity_id: String = row.get(1)?;
    let entity_type: String = row.get(2)?;
    let author_id: Option<String> = row.get(3)?;
    let created_at: String = row.get(4)?;
    let data: String = row.get(5)?;
    let description: Option<String> = row.get(6)?;
    let event_type: String = row.get(7)?;

    let author_id = author_id
      .map(|s| s.parse::<Id>())
      .transpose()
      .map_err(Error::InvalidValue)?;
    let created_at = DateTime::parse_from_rfc3339(&created_at)
      .map(|dt| dt.with_timezone(&Utc))
      .map_err(|e| Error::InvalidValue(e.to_string()))?;
    let data: Value = serde_json::from_str(&data).map_err(|e| Error::InvalidValue(e.to_string()))?;
    let entity_id: Id = entity_id.parse().map_err(Error::InvalidValue)?;
    let entity_type: EntityType = entity_type.parse().map_err(Error::InvalidValue)?;
    let event_type: EventKind = event_type.parse().map_err(Error::InvalidValue)?;
    let id: Id = id.parse().map_err(Error::InvalidValue)?;

    Ok(Self {
      author_id,
      created_at,
      data,
      description,
      entity_id,
      entity_type,
      event_type,
      id,
    })
  }
}
