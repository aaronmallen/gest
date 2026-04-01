use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::{id::Id, note::AuthorType};

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(tag = "type")]
#[allow(clippy::enum_variant_names)]
pub enum EventKind {
  #[serde(rename = "phase-change")]
  PhaseChange { from: Option<u16>, to: Option<u16> },
  #[serde(rename = "priority-change")]
  PriorityChange { from: Option<u8>, to: Option<u8> },
  #[serde(rename = "status-change")]
  StatusChange { from: String, to: String },
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Event {
  pub author: String,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub author_email: Option<String>,
  #[serde(default)]
  pub author_type: AuthorType,
  pub created_at: DateTime<Utc>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub description: Option<String>,
  pub id: Id,
  pub kind: EventKind,
}

#[cfg(test)]
mod tests {
  use super::*;

  mod event_kind {
    use super::*;

    mod roundtrip {
      use pretty_assertions::assert_eq;

      use super::*;

      #[test]
      fn it_roundtrips_phase_change_through_toml() {
        let kind = EventKind::PhaseChange {
          from: Some(1),
          to: Some(2),
        };

        let toml_str = toml::to_string(&kind).unwrap();
        let roundtripped: EventKind = toml::from_str(&toml_str).unwrap();
        assert_eq!(kind, roundtripped);
      }

      #[test]
      fn it_roundtrips_phase_change_with_none_through_toml() {
        let kind = EventKind::PhaseChange {
          from: None,
          to: Some(1),
        };

        let toml_str = toml::to_string(&kind).unwrap();
        let roundtripped: EventKind = toml::from_str(&toml_str).unwrap();
        assert_eq!(kind, roundtripped);
      }

      #[test]
      fn it_roundtrips_priority_change_through_toml() {
        let kind = EventKind::PriorityChange {
          from: Some(0),
          to: Some(1),
        };

        let toml_str = toml::to_string(&kind).unwrap();
        let roundtripped: EventKind = toml::from_str(&toml_str).unwrap();
        assert_eq!(kind, roundtripped);
      }

      #[test]
      fn it_roundtrips_priority_change_with_none_through_toml() {
        let kind = EventKind::PriorityChange {
          from: None,
          to: Some(0),
        };

        let toml_str = toml::to_string(&kind).unwrap();
        let roundtripped: EventKind = toml::from_str(&toml_str).unwrap();
        assert_eq!(kind, roundtripped);
      }

      #[test]
      fn it_roundtrips_status_change_through_toml() {
        let kind = EventKind::StatusChange {
          from: "open".to_string(),
          to: "in-progress".to_string(),
        };

        let toml_str = toml::to_string(&kind).unwrap();
        let roundtripped: EventKind = toml::from_str(&toml_str).unwrap();
        assert_eq!(kind, roundtripped);
      }
    }
  }

  mod event {
    use super::*;

    mod roundtrip {
      use pretty_assertions::assert_eq;

      use super::*;

      #[test]
      fn it_omits_optional_fields_when_none() {
        let now = Utc::now();
        let event = Event {
          author: "claude".to_string(),
          author_email: None,
          author_type: AuthorType::Agent,
          created_at: now,
          description: None,
          id: "zyxwvutsrqponmlkzyxwvutsrqponmlk".parse().unwrap(),
          kind: EventKind::StatusChange {
            from: "open".to_string(),
            to: "done".to_string(),
          },
        };

        let toml_str = toml::to_string(&event).unwrap();
        assert!(!toml_str.contains("author_email"));
        assert!(!toml_str.contains("description"));
      }

      #[test]
      fn it_roundtrips_agent_event_through_toml() {
        let now = Utc::now();
        let event = Event {
          author: "claude".to_string(),
          author_email: None,
          author_type: AuthorType::Agent,
          created_at: now,
          description: None,
          id: "zyxwvutsrqponmlkzyxwvutsrqponmlk".parse().unwrap(),
          kind: EventKind::PriorityChange {
            from: Some(0),
            to: Some(1),
          },
        };

        let toml_str = toml::to_string(&event).unwrap();
        let roundtripped: Event = toml::from_str(&toml_str).unwrap();
        assert_eq!(event, roundtripped);
      }

      #[test]
      fn it_roundtrips_through_toml() {
        let now = Utc::now();
        let event = Event {
          author: "alice".to_string(),
          author_email: Some("alice@example.com".to_string()),
          author_type: AuthorType::Human,
          created_at: now,
          description: Some("Status changed from open to in-progress".to_string()),
          id: "zyxwvutsrqponmlkzyxwvutsrqponmlk".parse().unwrap(),
          kind: EventKind::StatusChange {
            from: "open".to_string(),
            to: "in-progress".to_string(),
          },
        };

        let toml_str = toml::to_string(&event).unwrap();
        let roundtripped: Event = toml::from_str(&toml_str).unwrap();
        assert_eq!(event, roundtripped);
      }
    }
  }
}
