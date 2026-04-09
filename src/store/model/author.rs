use chrono::{DateTime, Utc};
use getset::{CopyGetters, Getters};
use libsql::Row;
use serde::{Deserialize, Serialize};

use super::primitives::{AuthorType, Id};
use crate::store::Error;

/// An author who can create notes, events, and other actions.
#[derive(Clone, CopyGetters, Debug, Deserialize, Eq, Getters, PartialEq, Serialize)]
pub struct Model {
  /// Whether the author is a human or an automated agent.
  #[getset(get_copy = "pub")]
  author_type: AuthorType,
  /// When the author was first recorded.
  #[get = "pub"]
  created_at: DateTime<Utc>,
  /// Optional contact email address.
  #[get = "pub"]
  email: Option<String>,
  /// Stable identifier assigned at creation.
  #[get = "pub"]
  id: Id,
  /// Display name.
  #[get = "pub"]
  name: String,
  /// When the author record was last modified.
  #[get = "pub"]
  updated_at: DateTime<Utc>,
}

impl Model {
  /// Create a new author with a fresh [`Id`] and timestamps set to now.
  pub fn new(name: impl Into<String>, author_type: AuthorType) -> Self {
    let now = Utc::now();
    Self {
      author_type,
      created_at: now,
      email: None,
      id: Id::new(),
      name: name.into(),
      updated_at: now,
    }
  }

  /// Set the email address.
  pub fn with_email(mut self, email: impl Into<String>) -> Self {
    self.email = Some(email.into());
    self
  }
}

/// Converts a database row into a [`Model`].
///
/// Expects columns in order: `id`, `author_type`, `name`, `email`, `created_at`, `updated_at`.
impl TryFrom<Row> for Model {
  type Error = Error;

  fn try_from(row: Row) -> Result<Self, Self::Error> {
    let id: String = row.get(0)?;
    let author_type: String = row.get(1)?;
    let name: String = row.get(2)?;
    let email: Option<String> = row.get(3)?;
    let created_at: String = row.get(4)?;
    let updated_at: String = row.get(5)?;

    let author_type: AuthorType = author_type.parse().map_err(Error::InvalidValue)?;
    let id: Id = id.parse().map_err(Error::InvalidValue)?;
    let created_at = DateTime::parse_from_rfc3339(&created_at)
      .map(|dt| dt.with_timezone(&Utc))
      .map_err(|e| Error::InvalidValue(e.to_string()))?;
    let updated_at = DateTime::parse_from_rfc3339(&updated_at)
      .map(|dt| dt.with_timezone(&Utc))
      .map_err(|e| Error::InvalidValue(e.to_string()))?;

    Ok(Self {
      author_type,
      created_at,
      email,
      id,
      name,
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
    fn it_creates_a_human_author() {
      let author = Model::new("Alice", AuthorType::Human);

      assert_eq!(author.name(), "Alice");
      assert_eq!(author.author_type(), AuthorType::Human);
      assert_eq!(author.email(), &None);
    }

    #[test]
    fn it_creates_an_agent_author() {
      let author = Model::new("Claude", AuthorType::Agent);

      assert_eq!(author.author_type(), AuthorType::Agent);
    }
  }

  mod with_email {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_sets_the_email() {
      let author = Model::new("Alice", AuthorType::Human).with_email("alice@example.com");

      assert_eq!(author.email().as_deref(), Some("alice@example.com"));
    }
  }
}
