//! Soft-delete tombstone primitives shared across per-entity sync adapters.
//!
//! ADR-0016 dictates that deletion is signaled by a top-level
//! `deleted_at: Option<DateTime<Utc>>` field on every entity wrapper struct.
//! Files with `deleted_at` set are kept on disk so that two collaborators
//! deleting the same entity merge cleanly. The reader treats them as
//! deletions and removes the corresponding SQLite row on import.
//!
//! [`Tombstone`] is the read-side trait the orchestrator uses to ask "should
//! I treat this file as deleted?" without knowing the wrapper's concrete type.
//! [`Tombstoned<T>`] is an optional generic helper for adapters that want to
//! attach a `deleted_at` field to an existing payload type without writing a
//! new wrapper struct by hand.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Read-side view of a tombstoneable entity wrapper.
///
/// Adapters implement this on their wrapper struct so the orchestrator can
/// branch on tombstone status uniformly. Implementations are usually a single
/// accessor returning the wrapper's `deleted_at` field.
pub trait Tombstone {
  /// The instant the entity was soft-deleted, or `None` if it is live.
  fn deleted_at(&self) -> Option<DateTime<Utc>>;

  /// Convenience: `true` when the entity carries a tombstone.
  fn is_deleted(&self) -> bool {
    self.deleted_at().is_some()
  }
}

/// Generic wrapper that attaches a tombstone to an arbitrary payload `T`.
///
/// The `deleted_at` field is declared first so it appears at the top of the
/// serialized YAML. `inner` uses `#[serde(flatten)]` so the payload's fields
/// land at the same nesting level as `deleted_at`, producing files that read
/// as one structured entity rather than `inner: { … }`.
///
/// Adapters that need finer control over field ordering or non-flatten
/// composition can write their own wrapper struct and implement [`Tombstone`]
/// directly. This helper exists for adapters where the default layout is fine.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Tombstoned<T> {
  /// Instant the entity was soft-deleted; absent for live entities.
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub deleted_at: Option<DateTime<Utc>>,
  /// The wrapped payload, flattened into the parent document.
  #[serde(flatten)]
  pub inner: T,
}

impl<T> Tombstoned<T> {
  /// Wrap a live (non-deleted) payload.
  pub fn live(inner: T) -> Self {
    Self {
      deleted_at: None,
      inner,
    }
  }

  /// Wrap a payload with the given tombstone instant.
  pub fn deleted(inner: T, deleted_at: DateTime<Utc>) -> Self {
    Self {
      deleted_at: Some(deleted_at),
      inner,
    }
  }
}

impl<T> Tombstone for Tombstoned<T> {
  fn deleted_at(&self) -> Option<DateTime<Utc>> {
    self.deleted_at
  }
}

#[cfg(test)]
mod tests {
  use chrono::TimeZone;
  use serde::{Deserialize, Serialize};

  use super::*;

  #[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
  struct Payload {
    name: String,
    count: u32,
  }

  fn payload() -> Payload {
    Payload {
      name: "demo".into(),
      count: 3,
    }
  }

  fn instant() -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 4, 8, 12, 0, 0).unwrap()
  }

  mod is_deleted {
    use super::*;

    #[test]
    fn it_is_false_when_deleted_at_is_none() {
      let wrapper = Tombstoned::live(payload());

      assert!(!wrapper.is_deleted());
    }

    #[test]
    fn it_is_true_when_deleted_at_is_set() {
      let wrapper = Tombstoned::deleted(payload(), instant());

      assert!(wrapper.is_deleted());
    }
  }

  mod serialize {
    use super::*;

    #[test]
    fn it_omits_deleted_at_for_live_entities() {
      let wrapper = Tombstoned::live(payload());

      let yaml = yaml_serde::to_string(&wrapper).unwrap();

      assert!(!yaml.contains("deleted_at"));
    }

    #[test]
    fn it_emits_deleted_at_when_set() {
      let wrapper = Tombstoned::deleted(payload(), instant());

      let yaml = yaml_serde::to_string(&wrapper).unwrap();

      assert!(yaml.contains("deleted_at"));
    }

    #[test]
    fn it_flattens_inner_fields_into_the_top_level_document() {
      let wrapper = Tombstoned::live(payload());

      let yaml = yaml_serde::to_string(&wrapper).unwrap();

      assert!(yaml.contains("name:"));
      assert!(yaml.contains("count:"));
      assert!(!yaml.contains("inner:"));
    }
  }

  mod deserialize {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_roundtrips_a_live_payload() {
      let wrapper = Tombstoned::live(payload());

      let yaml = yaml_serde::to_string(&wrapper).unwrap();
      let parsed: Tombstoned<Payload> = yaml_serde::from_str(&yaml).unwrap();

      assert_eq!(parsed, wrapper);
    }

    #[test]
    fn it_roundtrips_a_deleted_payload() {
      let wrapper = Tombstoned::deleted(payload(), instant());

      let yaml = yaml_serde::to_string(&wrapper).unwrap();
      let parsed: Tombstoned<Payload> = yaml_serde::from_str(&yaml).unwrap();

      assert_eq!(parsed, wrapper);
    }
  }
}
