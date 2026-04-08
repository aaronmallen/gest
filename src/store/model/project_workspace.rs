use std::path::PathBuf;

use chrono::{DateTime, Utc};
use getset::Getters;
use libsql::Row;
use serde::{Deserialize, Serialize};

use super::{Error, primitives::Id};

/// A workspace directory that belongs to a [`super::Project`].
///
/// Workspaces represent distinct working directories within a project. Each
/// workspace is tied to exactly one project via [`project_id`](Model::project_id)
/// and is uniquely constrained on the `(path, project_id)` pair.
#[derive(Clone, Debug, Deserialize, Eq, Getters, PartialEq, Serialize)]
pub struct Model {
  #[get = "pub"]
  created_at: DateTime<Utc>,
  #[get = "pub"]
  id: Id,
  #[get = "pub"]
  path: PathBuf,
  #[get = "pub"]
  project_id: Id,
  #[get = "pub"]
  updated_at: DateTime<Utc>,
}

impl Model {
  /// Create a new project workspace with a fresh [`Id`] and timestamps set to now.
  pub fn new(path: PathBuf, project_id: Id) -> Self {
    let now = Utc::now();
    Self {
      created_at: now,
      id: Id::new(),
      path,
      project_id,
      updated_at: now,
    }
  }
}

/// Converts a database row into a [`Model`].
///
/// Expects columns in order: `id`, `project_id`, `path`, `created_at`, `updated_at`.
impl TryFrom<Row> for Model {
  type Error = Error;

  fn try_from(row: Row) -> Result<Self, Self::Error> {
    let id: String = row.get(0)?;
    let project_id: String = row.get(1)?;
    let path: String = row.get(2)?;
    let created_at: String = row.get(3)?;
    let updated_at: String = row.get(4)?;

    let id: Id = id.parse().map_err(Error::InvalidValue)?;
    let project_id: Id = project_id.parse().map_err(Error::InvalidValue)?;
    let created_at = DateTime::parse_from_rfc3339(&created_at)
      .map(|dt| dt.with_timezone(&Utc))
      .map_err(|e| Error::InvalidValue(e.to_string()))?;
    let updated_at = DateTime::parse_from_rfc3339(&updated_at)
      .map(|dt| dt.with_timezone(&Utc))
      .map_err(|e| Error::InvalidValue(e.to_string()))?;

    Ok(Self {
      created_at,
      id,
      path: PathBuf::from(path),
      project_id,
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
      let workspace = Model::new(PathBuf::from("/tmp/test"), Id::new());
      let after = Utc::now();

      assert!(*workspace.created_at() >= before && *workspace.created_at() <= after);
      assert_eq!(workspace.created_at(), workspace.updated_at());
    }
  }
}
