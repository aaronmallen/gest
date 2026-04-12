use std::path::PathBuf;

use chrono::{DateTime, Utc};
use getset::Getters;
use libsql::Row;
use serde::{Deserialize, Serialize};

use super::primitives::Id;
use crate::store::Error;

/// A project represents a tracked repository or directory root.
///
/// Each project is uniquely identified by its [`root`](Model::root) path and
/// is assigned a stable [`Id`] at creation time.
#[derive(Clone, Debug, Deserialize, Eq, Getters, PartialEq, Serialize)]
pub struct Model {
  /// When the project was soft-archived, or `None` if it is active.
  #[get = "pub"]
  archived_at: Option<DateTime<Utc>>,
  /// When the project was first registered.
  #[get = "pub"]
  created_at: DateTime<Utc>,
  /// Stable identifier shared with collaborators via synced `.gest/project.yaml`.
  #[get = "pub"]
  id: Id,
  /// Absolute path to the local checkout root.
  #[get = "pub"]
  root: PathBuf,
  /// When the project record was last modified.
  #[get = "pub"]
  updated_at: DateTime<Utc>,
}

impl Model {
  /// Reconstruct a project from a synced `.gest/project.yaml` file.
  ///
  /// The id and timestamps come from the synced file (so collaborators share
  /// a stable identity), while `root` is the local checkout path.
  pub fn from_synced_parts(
    id: Id,
    root: PathBuf,
    archived_at: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
  ) -> Self {
    Self {
      archived_at,
      created_at,
      id,
      root,
      updated_at,
    }
  }

  /// Create a new project with a fresh [`Id`] and timestamps set to now.
  pub fn new(root: PathBuf) -> Self {
    let now = Utc::now();
    Self {
      archived_at: None,
      created_at: now,
      id: Id::new(),
      root,
      updated_at: now,
    }
  }
}

/// Converts a database row into a [`Model`].
///
/// Expects columns in order: `id`, `root`, `archived_at`, `created_at`, `updated_at`.
impl TryFrom<Row> for Model {
  type Error = Error;

  fn try_from(row: Row) -> Result<Self, Self::Error> {
    let id: String = row.get(0)?;
    let root: String = row.get(1)?;
    let archived_at: Option<String> = row.get(2)?;
    let created_at: String = row.get(3)?;
    let updated_at: String = row.get(4)?;

    let id: Id = id.parse().map_err(Error::InvalidValue)?;
    let archived_at = archived_at
      .map(|s| {
        DateTime::parse_from_rfc3339(&s)
          .map(|dt| dt.with_timezone(&Utc))
          .map_err(|e| Error::InvalidValue(e.to_string()))
      })
      .transpose()?;
    let created_at = DateTime::parse_from_rfc3339(&created_at)
      .map(|dt| dt.with_timezone(&Utc))
      .map_err(|e| Error::InvalidValue(e.to_string()))?;
    let updated_at = DateTime::parse_from_rfc3339(&updated_at)
      .map(|dt| dt.with_timezone(&Utc))
      .map_err(|e| Error::InvalidValue(e.to_string()))?;

    Ok(Self {
      archived_at,
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
