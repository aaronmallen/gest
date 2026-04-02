//! Domain model types for tasks, iterations, artifacts, and their identifiers.

use std::fmt::{self, Formatter};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Deserializer, de, de::value::MapAccessDeserializer};

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

/// The kind of top-level entity stored in a gest project.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EntityType {
  Artifact,
  Iteration,
  Task,
}

impl fmt::Display for EntityType {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    match self {
      Self::Artifact => write!(f, "artifact"),
      Self::Iteration => write!(f, "iteration"),
      Self::Task => write!(f, "task"),
    }
  }
}

/// Backward-compatible deserializer for `Option<DateTime<Utc>>` fields.
///
/// Accepts RFC 3339 strings, legacy empty strings (treated as `None`), and native TOML datetimes.
pub(crate) fn deserialize_optional_datetime<'de, D>(deserializer: D) -> Result<Option<DateTime<Utc>>, D::Error>
where
  D: Deserializer<'de>,
{
  struct OptionalDateTimeVisitor;

  impl<'de> de::Visitor<'de> for OptionalDateTimeVisitor {
    type Value = Option<DateTime<Utc>>;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
      formatter.write_str("an RFC 3339 datetime string, a native TOML datetime, or an empty string")
    }

    fn visit_str<E: de::Error>(self, value: &str) -> Result<Self::Value, E> {
      if value.is_empty() {
        return Ok(None);
      }
      DateTime::parse_from_rfc3339(value)
        .map(|dt| Some(dt.with_timezone(&Utc)))
        .map_err(de::Error::custom)
    }

    fn visit_string<E: de::Error>(self, value: String) -> Result<Self::Value, E> {
      self.visit_str(&value)
    }

    fn visit_map<A: de::MapAccess<'de>>(self, map: A) -> Result<Self::Value, A::Error> {
      let dt = toml::value::Datetime::deserialize(MapAccessDeserializer::new(map))?;
      let s = dt.to_string();
      DateTime::parse_from_rfc3339(&s)
        .map(|dt| Some(dt.with_timezone(&Utc)))
        .map_err(de::Error::custom)
    }
  }

  deserializer.deserialize_any(OptionalDateTimeVisitor)
}
