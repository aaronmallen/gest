use libsql::{Connection, Error as DbError, Value};

use crate::store::model::{
  Author, Error as ModelError,
  primitives::{AuthorType, Id},
};

/// Errors that can occur in author repository operations.
#[derive(Debug, thiserror::Error)]
pub enum Error {
  /// The underlying database driver returned an error.
  #[error(transparent)]
  Database(#[from] DbError),
  /// A row could not be converted into a domain model.
  #[error(transparent)]
  Model(#[from] ModelError),
}

const SELECT_COLUMNS: &str = "id, author_type, name, email, created_at, updated_at";

/// Return all authors ordered by name.
#[cfg(test)]
pub async fn all(conn: &Connection) -> Result<Vec<Author>, Error> {
  log::debug!("repo::author::all");
  let mut rows = conn
    .query(&format!("SELECT {SELECT_COLUMNS} FROM authors ORDER BY name"), ())
    .await?;

  let mut authors = Vec::new();
  while let Some(row) = rows.next().await? {
    authors.push(Author::try_from(row)?);
  }
  Ok(authors)
}

/// Create a new author and persist it.
pub async fn create(conn: &Connection, author: &Author) -> Result<Author, Error> {
  log::debug!("repo::author::create");
  let email: Value = match author.email() {
    Some(e) => Value::from(e.clone()),
    None => Value::Null,
  };
  conn
    .execute(
      &format!("INSERT INTO authors ({SELECT_COLUMNS}) VALUES (?1, ?2, ?3, ?4, ?5, ?6)"),
      libsql::params![
        author.id().to_string(),
        author.author_type().to_string(),
        author.name().clone(),
        email,
        author.created_at().to_rfc3339(),
        author.updated_at().to_rfc3339(),
      ],
    )
    .await?;

  find_by_id(conn, author.id().clone())
    .await?
    .ok_or_else(|| Error::Model(ModelError::InvalidValue("author not found after insert".into())))
}

/// Find an author by their [`Id`].
pub async fn find_by_id(conn: &Connection, id: impl Into<Id>) -> Result<Option<Author>, Error> {
  log::debug!("repo::author::find_by_id");
  let id = id.into();
  let mut rows = conn
    .query(
      &format!("SELECT {SELECT_COLUMNS} FROM authors WHERE id = ?1"),
      [id.to_string()],
    )
    .await?;

  match rows.next().await? {
    Some(row) => Ok(Some(Author::try_from(row)?)),
    None => Ok(None),
  }
}

/// Find an author by name, optionally filtering by email.
pub async fn find_by_name(conn: &Connection, name: &str, email: Option<&str>) -> Result<Option<Author>, Error> {
  log::debug!("repo::author::find_by_name");
  let mut rows = if let Some(email) = email {
    conn
      .query(
        &format!("SELECT {SELECT_COLUMNS} FROM authors WHERE name = ?1 AND email = ?2"),
        [name.to_string(), email.to_string()],
      )
      .await?
  } else {
    conn
      .query(
        &format!("SELECT {SELECT_COLUMNS} FROM authors WHERE name = ?1 AND email IS NULL"),
        [name.to_string()],
      )
      .await?
  };

  match rows.next().await? {
    Some(row) => Ok(Some(Author::try_from(row)?)),
    None => Ok(None),
  }
}

/// Find an existing author by name/email or create a new one.
pub async fn find_or_create(
  conn: &Connection,
  name: &str,
  email: Option<&str>,
  author_type: AuthorType,
) -> Result<Author, Error> {
  log::debug!("repo::author::find_or_create");
  if let Some(existing) = find_by_name(conn, name, email).await? {
    return Ok(existing);
  }

  let mut author = Author::new(name, author_type);
  if let Some(email) = email {
    author = author.with_email(email);
  }
  create(conn, &author).await
}

#[cfg(test)]
mod tests {
  use std::sync::Arc;

  use tempfile::TempDir;

  use super::*;
  use crate::store::{self, Db};

  async fn setup() -> (Arc<Db>, Connection, TempDir) {
    let (store, tmp) = store::open_temp().await.unwrap();
    let conn = store.connect().await.unwrap();
    (store, conn, tmp)
  }

  mod all {
    use pretty_assertions::assert_eq;

    use super::*;

    #[tokio::test]
    async fn it_returns_authors_sorted_by_name() {
      let (_store, conn, _tmp) = setup().await;

      let b = Author::new("Bravo", AuthorType::Human);
      let a = Author::new("Alpha", AuthorType::Agent);
      create(&conn, &b).await.unwrap();
      create(&conn, &a).await.unwrap();

      let authors = all(&conn).await.unwrap();
      assert_eq!(authors.len(), 2);
      assert_eq!(authors[0].name(), "Alpha");
      assert_eq!(authors[1].name(), "Bravo");
    }
  }

  mod create_fn {
    use pretty_assertions::assert_eq;

    use super::*;

    #[tokio::test]
    async fn it_persists_the_author() {
      let (_store, conn, _tmp) = setup().await;

      let author = Author::new("Alice", AuthorType::Human).with_email("alice@example.com");
      let created = create(&conn, &author).await.unwrap();

      assert_eq!(created.name(), "Alice");
      assert_eq!(created.email().as_deref(), Some("alice@example.com"));
      assert_eq!(created.author_type(), AuthorType::Human);
    }
  }

  mod find_by_name {
    use pretty_assertions::assert_eq;

    use super::*;

    #[tokio::test]
    async fn it_finds_by_name_without_email() {
      let (_store, conn, _tmp) = setup().await;

      let author = Author::new("Bob", AuthorType::Agent);
      create(&conn, &author).await.unwrap();

      let found = find_by_name(&conn, "Bob", None).await.unwrap();

      assert_eq!(found.as_ref().map(|a| a.name().as_str()), Some("Bob"));
    }

    #[tokio::test]
    async fn it_returns_none_when_not_found() {
      let (_store, conn, _tmp) = setup().await;

      let found = find_by_name(&conn, "Nobody", None).await.unwrap();

      assert_eq!(found, None);
    }
  }

  mod find_or_create_fn {
    use pretty_assertions::assert_eq;

    use super::*;

    #[tokio::test]
    async fn it_creates_when_not_found() {
      let (_store, conn, _tmp) = setup().await;

      let author = find_or_create(&conn, "New Author", None, AuthorType::Human)
        .await
        .unwrap();

      assert_eq!(author.name(), "New Author");
    }

    #[tokio::test]
    async fn it_returns_existing_when_found() {
      let (_store, conn, _tmp) = setup().await;

      let first = find_or_create(&conn, "Same Author", None, AuthorType::Human)
        .await
        .unwrap();
      let second = find_or_create(&conn, "Same Author", None, AuthorType::Human)
        .await
        .unwrap();

      assert_eq!(first.id(), second.id());
    }
  }
}
