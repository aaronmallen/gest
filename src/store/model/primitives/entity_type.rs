use std::{
  fmt::{self, Display, Formatter},
  str::FromStr,
};

use serde::{Deserialize, Serialize};

/// The type of entity in the system.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Primitive {
  /// Persistent document such as a spec or ADR.
  Artifact,
  /// Time-boxed collection of tasks.
  Iteration,
  /// Unit of work.
  Task,
}

impl Display for Primitive {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    match self {
      Self::Artifact => f.write_str("artifact"),
      Self::Iteration => f.write_str("iteration"),
      Self::Task => f.write_str("task"),
    }
  }
}

impl FromStr for Primitive {
  type Err = String;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s.to_lowercase().as_str() {
      "artifact" => Ok(Self::Artifact),
      "iteration" => Ok(Self::Iteration),
      "task" => Ok(Self::Task),
      other => Err(format!("invalid entity type: {other}")),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  mod display {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_formats_artifact() {
      assert_eq!(Primitive::Artifact.to_string(), "artifact");
    }

    #[test]
    fn it_formats_iteration() {
      assert_eq!(Primitive::Iteration.to_string(), "iteration");
    }

    #[test]
    fn it_formats_task() {
      assert_eq!(Primitive::Task.to_string(), "task");
    }
  }

  mod from_str {
    use super::*;

    #[test]
    fn it_parses_case_insensitively() {
      assert_eq!("ARTIFACT".parse::<Primitive>().unwrap(), Primitive::Artifact);
      assert_eq!("Task".parse::<Primitive>().unwrap(), Primitive::Task);
    }

    #[test]
    fn it_rejects_invalid() {
      assert!("widget".parse::<Primitive>().is_err());
    }
  }
}
