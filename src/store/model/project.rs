use std::path::PathBuf;

use chrono::{DateTime, Utc};
use libsql::Row;
use serde::{Deserialize, Serialize};

use super::{Error, primitives::Id};

/// A project represents a tracked repository or directory root.
///
/// Each project is uniquely identified by its [`root`](Model::root) path and
/// is assigned a stable [`Id`] at creation time.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Model {
  created_at: DateTime<Utc>,
  id: Id,
  root: PathBuf,
  updated_at: DateTime<Utc>,
}

impl Model {
  /// Create a new project with a fresh [`Id`] and timestamps set to now.
  pub fn new(root: PathBuf) -> Self {
    let now = Utc::now();
    Self {
      created_at: now,
      id: Id::new(),
      root,
      updated_at: now,
    }
  }

  /// Reconstruct a project from a synced `.gest/project.yaml` file.
  ///
  /// The id and timestamps come from the synced file (so collaborators share
  /// a stable identity), while `root` is the local checkout path.
  pub fn from_synced_parts(id: Id, root: PathBuf, created_at: DateTime<Utc>, updated_at: DateTime<Utc>) -> Self {
    Self {
      created_at,
      id,
      root,
      updated_at,
    }
  }

  /// When this project was first created.
  pub fn created_at(&self) -> &DateTime<Utc> {
    &self.created_at
  }

  /// The unique identifier for this project.
  pub fn id(&self) -> &Id {
    &self.id
  }

  /// The absolute path to the project root directory.
  pub fn root(&self) -> &PathBuf {
    &self.root
  }

  /// When this project was last modified.
  pub fn updated_at(&self) -> &DateTime<Utc> {
    &self.updated_at
  }
}

/// Converts a database row into a [`Model`].
///
/// Expects columns in order: `id`, `root`, `created_at`, `updated_at`.
impl TryFrom<Row> for Model {
  type Error = Error;

  fn try_from(row: Row) -> Result<Self, Self::Error> {
    let id: String = row.get(0)?;
    let root: String = row.get(1)?;
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
      root: PathBuf::from(root),
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
    fn it_sets_timestamps_to_now() {
      let before = Utc::now();
      let project = Model::new(PathBuf::from("/tmp/test"));
      let after = Utc::now();

      assert!(*project.created_at() >= before && *project.created_at() <= after);
      assert_eq!(project.created_at(), project.updated_at());
    }
  }
}
