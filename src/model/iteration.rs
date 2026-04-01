use std::{
  fmt::{self, Display, Formatter},
  str::FromStr,
};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::{event::Event, id::Id, link::Link};

/// A time-boxed planning container that groups related tasks.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Iteration {
  #[serde(
    default,
    skip_serializing_if = "Option::is_none",
    deserialize_with = "super::deserialize_optional_datetime"
  )]
  pub completed_at: Option<DateTime<Utc>>,
  pub created_at: DateTime<Utc>,
  pub description: String,
  #[serde(default)]
  pub events: Vec<Event>,
  pub id: Id,
  #[serde(default)]
  pub links: Vec<Link>,
  #[serde(default)]
  pub metadata: toml::Table,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub phase_count: Option<usize>,
  pub status: Status,
  #[serde(default)]
  pub tags: Vec<String>,
  #[serde(default)]
  pub tasks: Vec<String>,
  pub title: String,
  pub updated_at: DateTime<Utc>,
}

/// Criteria for filtering iterations when listing.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct IterationFilter {
  pub all: bool,
  pub status: Option<Status>,
  pub tag: Option<String>,
}

/// Partial update payload for an existing iteration.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct IterationPatch {
  pub description: Option<String>,
  pub metadata: Option<toml::Table>,
  pub status: Option<Status>,
  pub tags: Option<Vec<String>>,
  pub title: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct NewIteration {
  pub description: String,
  #[serde(default)]
  pub links: Vec<Link>,
  #[serde(default)]
  pub metadata: toml::Table,
  pub status: Status,
  #[serde(default)]
  pub tags: Vec<String>,
  #[serde(default)]
  pub tasks: Vec<String>,
  pub title: String,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub enum Status {
  #[default]
  #[serde(rename = "active")]
  Active,
  #[serde(rename = "completed")]
  Completed,
  #[serde(rename = "failed")]
  Failed,
}

impl Status {
  pub fn as_str(&self) -> &'static str {
    match self {
      Self::Active => "active",
      Self::Completed => "completed",
      Self::Failed => "failed",
    }
  }

  pub fn is_terminal(&self) -> bool {
    matches!(self, Self::Completed | Self::Failed)
  }
}

impl Display for Status {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    f.write_str(self.as_str())
  }
}

impl FromStr for Status {
  type Err = String;

  fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
    match s {
      "active" => Ok(Self::Active),
      "completed" => Ok(Self::Completed),
      "failed" => Ok(Self::Failed),
      other => Err(format!("unknown status: {other}")),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  mod iteration {
    use super::*;

    mod roundtrip {
      use pretty_assertions::assert_eq;

      use super::*;

      #[test]
      fn it_deserializes_without_completed_at_field() {
        let toml_str = r#"
          created_at = "2026-04-01T12:00:00Z"
          description = "An iteration"
          id = "zyxwvutsrqponmlkzyxwvutsrqponmlk"
          status = "active"
          title = "Test"
          updated_at = "2026-04-01T12:00:00Z"
        "#;

        let iteration: Iteration = toml::from_str(toml_str).unwrap();
        assert_eq!(iteration.completed_at, None);
      }

      #[test]
      fn it_deserializes_without_events_field() {
        let toml_str = r#"
          completed_at = ""
          created_at = "2026-04-01T12:00:00Z"
          description = "An iteration"
          id = "zyxwvutsrqponmlkzyxwvutsrqponmlk"
          status = "active"
          title = "Test"
          updated_at = "2026-04-01T12:00:00Z"
        "#;

        let iteration: Iteration = toml::from_str(toml_str).unwrap();
        assert!(iteration.events.is_empty());
      }

      #[test]
      fn it_omits_completed_at_when_none() {
        let now = Utc::now();
        let iteration = Iteration {
          completed_at: None,
          created_at: now,
          description: "An iteration".to_string(),
          events: vec![],
          id: "zyxwvutsrqponmlkzyxwvutsrqponmlk".parse().unwrap(),
          links: vec![],
          metadata: toml::Table::new(),
          phase_count: None,
          status: Status::Active,
          tags: vec![],
          tasks: vec![],
          title: "Test".to_string(),
          updated_at: now,
        };

        let toml_str = toml::to_string(&iteration).unwrap();
        assert!(!toml_str.contains("completed_at"));
      }

      #[test]
      fn it_roundtrips_through_toml() {
        let now = Utc::now();
        let iteration = Iteration {
          completed_at: None,
          created_at: now,
          description: "An iteration description".to_string(),
          events: vec![],
          id: "zyxwvutsrqponmlkzyxwvutsrqponmlk".parse().unwrap(),
          links: vec![Link {
            ref_: "artifacts/zyxwvutsrqponmlkzyxwvutsrqponmlk".to_string(),
            rel: crate::model::link::RelationshipType::ChildOf,
          }],
          metadata: toml::Table::new(),
          phase_count: None,
          status: Status::Active,
          tags: vec!["sprint-1".to_string()],
          tasks: vec!["tasks/kkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkk".to_string()],
          title: "Test Iteration".to_string(),
          updated_at: now,
        };

        let toml_str = toml::to_string(&iteration).unwrap();
        let roundtripped: Iteration = toml::from_str(&toml_str).unwrap();
        assert_eq!(iteration, roundtripped);
      }
    }
  }

  mod status {
    use super::*;

    mod display {
      use pretty_assertions::assert_eq;

      use super::*;

      #[test]
      fn it_formats_as_lowercase() {
        assert_eq!(Status::Active.to_string(), "active");
        assert_eq!(Status::Completed.to_string(), "completed");
        assert_eq!(Status::Failed.to_string(), "failed");
      }
    }

    mod from_str {
      use pretty_assertions::assert_eq;

      use super::*;

      #[test]
      fn it_parses_valid_statuses() {
        assert_eq!("active".parse::<Status>().unwrap(), Status::Active);
        assert_eq!("completed".parse::<Status>().unwrap(), Status::Completed);
        assert_eq!("failed".parse::<Status>().unwrap(), Status::Failed);
      }

      #[test]
      fn it_rejects_unknown() {
        assert!("invalid".parse::<Status>().is_err());
      }
    }
  }
}
