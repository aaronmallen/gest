use chrono::{DateTime, Utc};

use crate::{
  config::Settings,
  model::{Id, event::AuthorInfo, link::Link},
  store,
};

/// Resolve an ID prefix to a full entity ID via the store.
pub trait Resolvable: Sized {
  /// The directory prefix used in ref-paths (e.g. `"tasks"`, `"iterations"`, `"artifacts"`).
  fn entity_prefix() -> &'static str;

  /// Resolve a short ID prefix to the full [`Id`] for this entity type.
  fn resolve_id(config: &Settings, prefix: &str) -> store::Result<Id>;
}

/// Read and write an entity via the store.
pub trait Storable: Sized {
  /// Read an entity from the store by its full ID.
  fn read(config: &Settings, id: &Id) -> store::Result<Self>;

  /// Write an entity to the store, persisting its current state.
  fn write(config: &Settings, entity: &Self) -> store::Result<()>;
}

/// Access to an entity's tags and updated-at timestamp for mutation.
pub trait Taggable {
  fn tags_mut(&mut self) -> &mut Vec<String>;

  fn set_updated_at(&mut self, time: DateTime<Utc>);
}

/// Access to an entity's links and updated-at timestamp for mutation.
pub trait Linkable {
  fn links_mut(&mut self) -> &mut Vec<Link>;

  fn set_updated_at(&mut self, time: DateTime<Utc>);
}

/// Access to an entity's TOML metadata table for mutation.
pub trait HasMetadata {
  fn metadata(&self) -> &toml::Table;

  fn metadata_mut(&mut self) -> &mut toml::Table;

  fn set_updated_at(&mut self, time: DateTime<Utc>);
}

/// Marker for entity types that carry a status field.
///
/// Only implemented on entity types that have a status (Task, Iteration).
/// Artifact does not have a status field.
pub trait HasStatus: Sized {
  type Status;

  /// Apply a status change via the entity's store update function.
  ///
  /// Each implementation delegates to its specific store update (e.g. `store::update_task`)
  /// which handles lifecycle management such as file movement, timestamps, and event recording.
  fn update_status(
    config: &Settings,
    id: &Id,
    status: Self::Status,
    author: Option<&AuthorInfo>,
  ) -> store::Result<Self>;
}
