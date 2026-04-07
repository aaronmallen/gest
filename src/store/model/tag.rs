use libsql::Row;
use serde::{Deserialize, Serialize};

use super::{Error, primitives::Id};

/// A deduplicated label that can be attached to any entity.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Model {
  id: Id,
  label: String,
}

impl Model {
  /// Create a new tag with a fresh [`Id`].
  pub fn new(label: impl Into<String>) -> Self {
    Self {
      id: Id::new(),
      label: label.into(),
    }
  }

  /// The unique identifier for this tag.
  pub fn id(&self) -> &Id {
    &self.id
  }

  /// The tag's display label.
  pub fn label(&self) -> &str {
    &self.label
  }
}

/// Expects columns in order: `id`, `label`.
impl TryFrom<Row> for Model {
  type Error = Error;

  fn try_from(row: Row) -> Result<Self, Self::Error> {
    let id: String = row.get(0)?;
    let label: String = row.get(1)?;

    let id: Id = id.parse().map_err(Error::InvalidValue)?;

    Ok(Self {
      id,
      label,
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
