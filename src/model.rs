//! Domain model types for tasks, iterations, artifacts, and their identifiers.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Deserializer, de};

pub mod artifact;
pub mod event;
pub mod id;
pub mod iteration;
pub mod link;
pub mod note;
pub mod task;

pub use artifact::{Artifact, ArtifactFilter, ArtifactPatch, NewArtifact};
pub use id::Id;
pub use iteration::{Iteration, IterationFilter, IterationPatch, NewIteration};
pub use note::{NewNote, Note, NotePatch};
pub use task::{NewTask, Task, TaskFilter, TaskPatch};

/// Backward-compatible deserializer for `Option<DateTime<Utc>>` fields.
///
/// Accepts both RFC 3339 timestamps and legacy empty strings (treated as `None`).
pub(crate) fn deserialize_optional_datetime<'de, D>(deserializer: D) -> Result<Option<DateTime<Utc>>, D::Error>
where
  D: Deserializer<'de>,
{
  let s = String::deserialize(deserializer)?;
  if s.is_empty() {
    Ok(None)
  } else {
    DateTime::parse_from_rfc3339(&s)
      .map(|dt| Some(dt.with_timezone(&Utc)))
      .map_err(de::Error::custom)
  }
}
