use std::{
  fmt::{self, Display, Formatter},
  str::FromStr,
};

use serde::{Deserialize, Serialize};

/// The type of author that performed an action.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum Primitive {
  /// An automated agent such as an LLM or CI bot.
  Agent,
  /// A human author (the default).
  #[default]
  Human,
}

impl Display for Primitive {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    match self {
      Self::Agent => f.write_str("agent"),
      Self::Human => f.write_str("human"),
    }
  }
}

impl FromStr for Primitive {
  type Err = String;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s.to_lowercase().as_str() {
      "agent" => Ok(Self::Agent),
      "human" => Ok(Self::Human),
      other => Err(format!("invalid author type: {other}")),
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
    fn it_formats_agent() {
      assert_eq!(Primitive::Agent.to_string(), "agent");
    }

    #[test]
    fn it_formats_human() {
      assert_eq!(Primitive::Human.to_string(), "human");
    }
  }

  mod from_str {
    use super::*;

    #[test]
    fn it_parses_case_insensitively() {
      assert_eq!("AGENT".parse::<Primitive>().unwrap(), Primitive::Agent);
      assert_eq!("Human".parse::<Primitive>().unwrap(), Primitive::Human);
    }

    #[test]
    fn it_rejects_invalid() {
      assert!("robot".parse::<Primitive>().is_err());
    }
  }
}
