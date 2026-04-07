use chrono::{DateTime, Utc};
use libsql::Row;
use serde::{Deserialize, Serialize};

use super::{
  Error,
  primitives::{AuthorType, Id},
};

/// An author who can create notes, events, and other actions.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Model {
  author_type: AuthorType,
  created_at: DateTime<Utc>,
  email: Option<String>,
  id: Id,
  name: String,
}

impl Model {
  /// Create a new author with a fresh [`Id`] and timestamp set to now.
  pub fn new(name: impl Into<String>, author_type: AuthorType) -> Self {
    Self {
      author_type,
      created_at: Utc::now(),
      email: None,
      id: Id::new(),
      name: name.into(),
    }
  }

  /// The type of this author (human or agent).
  pub fn author_type(&self) -> AuthorType {
    self.author_type
  }

  /// When this author was first created.
  pub fn created_at(&self) -> &DateTime<Utc> {
    &self.created_at
  }

  /// The author's email address, if provided.
  pub fn email(&self) -> Option<&str> {
    self.email.as_deref()
  }

  /// The unique identifier for this author.
  pub fn id(&self) -> &Id {
    &self.id
  }

  /// The author's display name.
  pub fn name(&self) -> &str {
    &self.name
  }

  /// Set the email address.
  pub fn with_email(mut self, email: impl Into<String>) -> Self {
    self.email = Some(email.into());
    self
  }
}

/// Expects columns in order: `id`, `author_type`, `created_at`, `email`, `name`.
impl TryFrom<Row> for Model {
  type Error = Error;

  fn try_from(row: Row) -> Result<Self, Self::Error> {
    let id: String = row.get(0)?;
    let author_type: String = row.get(1)?;
    let created_at: String = row.get(2)?;
    let email: Option<String> = row.get(3)?;
    let name: String = row.get(4)?;

    let author_type: AuthorType = author_type.parse().map_err(Error::InvalidValue)?;
    let id: Id = id.parse().map_err(Error::InvalidValue)?;
    let created_at = DateTime::parse_from_rfc3339(&created_at)
      .map(|dt| dt.with_timezone(&Utc))
      .map_err(|e| Error::InvalidValue(e.to_string()))?;

    Ok(Self {
      author_type,
      created_at,
      email,
      id,
      name,
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
      assert_eq!(author.email(), None);
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

      assert_eq!(author.email(), Some("alice@example.com"));
    }
  }
}
