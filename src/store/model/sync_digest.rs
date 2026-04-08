use chrono::{DateTime, Utc};
use libsql::Row;
use serde::{Deserialize, Serialize};

use super::{Error, primitives::Id};

/// Tracks content digests for the per-entity `.gest/` sync mirror.
///
/// Digests are keyed by `(project_id, relative_path)` where `relative_path`
/// is the path relative to the project's `.gest/` directory using forward
/// slashes (e.g. `task/abc.yaml`). This makes the cache portable across
/// checkouts and host operating systems.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Model {
  digest: String,
  project_id: Id,
  relative_path: String,
  synced_at: DateTime<Utc>,
}

impl Model {
  /// The SHA-256 content digest of the file.
  pub fn digest(&self) -> &str {
    &self.digest
  }

  /// The project this digest belongs to.
  pub fn project_id(&self) -> &Id {
    &self.project_id
  }

  /// The repo-relative path of the synced file (forward-slash, no leading
  /// slash, no `.gest/` prefix).
  pub fn relative_path(&self) -> &str {
    &self.relative_path
  }

  /// When this digest was last recorded.
  pub fn synced_at(&self) -> &DateTime<Utc> {
    &self.synced_at
  }
}

/// Expects columns in order: `project_id`, `relative_path`, `digest`, `synced_at`.
impl TryFrom<Row> for Model {
  type Error = Error;

  fn try_from(row: Row) -> Result<Self, Self::Error> {
    let project_id: String = row.get(0)?;
    let relative_path: String = row.get(1)?;
    let digest: String = row.get(2)?;
    let synced_at: String = row.get(3)?;

    let project_id: Id = project_id.parse().map_err(Error::InvalidValue)?;
    let synced_at = DateTime::parse_from_rfc3339(&synced_at)
      .map(|dt| dt.with_timezone(&Utc))
      .map_err(|e| Error::InvalidValue(e.to_string()))?;

    Ok(Self {
      digest,
      project_id,
      relative_path,
      synced_at,
    })
  }
}
