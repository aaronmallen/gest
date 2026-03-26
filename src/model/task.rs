use std::{fmt, str::FromStr};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::{id::Id, link::Link};

/// Canonical display ordering for task statuses.
pub const STATUS_ORDER: &[Status] = &[Status::Open, Status::InProgress, Status::Done, Status::Cancelled];

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct NewTask {
  pub description: String,
  pub links: Vec<Link>,
  pub metadata: toml::Table,
  pub status: Status,
  pub tags: Vec<String>,
  pub title: String,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub enum Status {
  #[serde(rename = "cancelled")]
  Cancelled,
  #[serde(rename = "done")]
  Done,
  #[serde(rename = "in-progress")]
  InProgress,
  #[default]
  #[serde(rename = "open")]
  Open,
}

impl Status {
  pub fn is_terminal(&self) -> bool {
    matches!(self, Self::Done | Self::Cancelled)
  }
}

impl fmt::Display for Status {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let s = match self {
      Self::Cancelled => "cancelled",
      Self::Done => "done",
      Self::InProgress => "in-progress",
      Self::Open => "open",
    };
    f.write_str(s)
  }
}

impl FromStr for Status {
  type Err = String;

  fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
    match s {
      "cancelled" => Ok(Self::Cancelled),
      "done" => Ok(Self::Done),
      "in-progress" => Ok(Self::InProgress),
      "open" => Ok(Self::Open),
      other => Err(format!("unknown status: {other}")),
    }
  }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Task {
  pub created_at: DateTime<Utc>,
  pub description: String,
  pub id: Id,
  #[serde(default)]
  pub links: Vec<Link>,
  #[serde(default)]
  pub metadata: toml::Table,
  #[serde(alias = "archived_at", with = "resolved_at_serde")]
  pub resolved_at: Option<DateTime<Utc>>,
  pub status: Status,
  #[serde(default)]
  pub tags: Vec<String>,
  pub title: String,
  pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct TaskFilter {
  pub all: bool,
  pub status: Option<Status>,
  pub tag: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct TaskPatch {
  pub description: Option<String>,
  pub metadata: Option<toml::Table>,
  pub status: Option<Status>,
  pub tags: Option<Vec<String>>,
  pub title: Option<String>,
}

mod resolved_at_serde {
  use chrono::{DateTime, Utc};
  use serde::{self, Deserialize, Deserializer, Serializer};

  pub fn serialize<S>(value: &Option<DateTime<Utc>>, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    match value {
      Some(dt) => serializer.serialize_str(&dt.to_rfc3339()),
      None => serializer.serialize_str(""),
    }
  }

  pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<DateTime<Utc>>, D::Error>
  where
    D: Deserializer<'de>,
  {
    let s = String::deserialize(deserializer)?;
    if s.is_empty() {
      Ok(None)
    } else {
      DateTime::parse_from_rfc3339(&s)
        .map(|dt| Some(dt.with_timezone(&Utc)))
        .map_err(serde::de::Error::custom)
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  mod status {
    use super::*;

    mod display {
      use pretty_assertions::assert_eq;

      use super::*;

      #[test]
      fn it_formats_as_kebab_case() {
        assert_eq!(Status::Cancelled.to_string(), "cancelled");
        assert_eq!(Status::Done.to_string(), "done");
        assert_eq!(Status::InProgress.to_string(), "in-progress");
        assert_eq!(Status::Open.to_string(), "open");
      }
    }

    mod from_str {
      use pretty_assertions::assert_eq;

      use super::*;

      #[test]
      fn it_parses_valid_statuses() {
        assert_eq!("cancelled".parse::<Status>().unwrap(), Status::Cancelled);
        assert_eq!("done".parse::<Status>().unwrap(), Status::Done);
        assert_eq!("in-progress".parse::<Status>().unwrap(), Status::InProgress);
        assert_eq!("open".parse::<Status>().unwrap(), Status::Open);
      }

      #[test]
      fn it_rejects_unknown() {
        assert!("invalid".parse::<Status>().is_err());
      }
    }
  }

  mod task {
    use super::*;

    mod roundtrip {
      use pretty_assertions::assert_eq;

      use super::*;

      #[test]
      fn it_roundtrips_through_toml() {
        let now = Utc::now();
        let task = Task {
          resolved_at: None,
          created_at: now,
          description: "A test task description".to_string(),
          id: "zyxwvutsrqponmlkzyxwvutsrqponmlk".parse().unwrap(),
          links: vec![Link {
            ref_: "https://example.com".to_string(),
            rel: crate::model::link::RelationshipType::RelatesTo,
          }],
          metadata: {
            let mut table = toml::Table::new();
            table.insert("priority".to_string(), toml::Value::String("high".to_string()));
            table
          },
          status: Status::Open,
          tags: vec!["test".to_string(), "example".to_string()],
          title: "Test Task".to_string(),
          updated_at: now,
        };

        let toml_str = toml::to_string(&task).unwrap();
        let roundtripped: Task = toml::from_str(&toml_str).unwrap();
        assert_eq!(task, roundtripped);
      }
    }
  }
}
