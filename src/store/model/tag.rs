use chrono::{DateTime, Utc};
use getset::Getters;
use libsql::Row;
use serde::{Deserialize, Serialize};

use super::{Error, primitives::Id};

/// A deduplicated label that can be attached to any entity.
#[derive(Clone, Debug, Deserialize, Eq, Getters, PartialEq, Serialize)]
pub struct Model {
  #[get = "pub"]
  created_at: DateTime<Utc>,
  #[get = "pub"]
  id: Id,
  #[get = "pub"]
  label: String,
  #[get = "pub"]
  updated_at: DateTime<Utc>,
}

impl Model {
  /// Create a new tag with a fresh [`Id`] and timestamps set to now.
  pub fn new(label: impl Into<String>) -> Self {
    let now = Utc::now();
    Self {
      created_at: now,
      id: Id::new(),
      label: label.into(),
      updated_at: now,
    }
  }
}

/// Converts a database row into a [`Model`].
///
/// Expects columns in order: `id`, `label`, `created_at`, `updated_at`.
impl TryFrom<Row> for Model {
  type Error = Error;

  fn try_from(row: Row) -> Result<Self, Self::Error> {
    let id: String = row.get(0)?;
    let label: String = row.get(1)?;
    let created_at: String = row.get(2)?;
    let updated_at: String = row.get(3)?;

    let id: Id = id.parse().map_err(Error::InvalidValue)?;
    let created_at = DateTime::parse_from_rfc3339(&created_at)
      .map(|dt| dt.with_timezone(&Utc))
      .map_err(|e| Error::InvalidValue(e.to_string()))?;
    let updated_at = DateTime::parse_from_rfc3339(&updated_at)
      .map(|dt| dt.with_timezone(&Utc))
      .map_err(|e| Error::InvalidValue(e.to_string()))?;

    Ok(Self {
      created_at,
      id,
      label,
      updated_at,
    })
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  mod new {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_creates_a_tag_with_label() {
      let tag = Model::new("blocked");

      assert_eq!(tag.label(), "blocked");
    }
  }
}
