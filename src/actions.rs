pub mod meta;
pub mod note;
pub mod tag;

use libsql::Connection;
use serde::Serialize;
use serde_json::Value;

use crate::store::{
  Error,
  model::{
    artifact, iteration,
    primitives::{EntityType, Id},
    task,
  },
  repo,
  repo::resolve::Table,
};

/// An entity that can be resolved from an ID prefix and fetched by full ID.
pub trait Findable {
  /// The domain model returned by `find_by_id`.
  type Model: Send + Serialize;

  /// The entity type for envelope/sidecar loading.
  fn entity_type() -> EntityType;

  /// Fetch the entity by its full resolved ID.
  fn find_by_id(conn: &Connection, id: impl Into<Id> + Send)
  -> impl Future<Output = Result<Self::Model, Error>> + Send;

  /// The `resolve` table used for prefix resolution.
  fn table() -> Table;
}

/// An entity that carries user-defined JSON metadata and can be patched.
pub trait HasMetadata: Findable + Prefixable {
  /// The patch type used to update the entity.
  type Patch: Default + Send;

  /// The table name used when recording transaction events (e.g., "tasks").
  fn event_table() -> &'static str;

  /// Read the metadata value from the model.
  fn metadata(model: &Self::Model) -> &Value;

  /// Produce a patch that sets only the metadata field.
  fn metadata_patch(metadata: Value) -> Self::Patch;

  /// Persist the patch and return the updated model.
  fn update(conn: &Connection, id: &Id, patch: &Self::Patch)
  -> impl Future<Output = Result<Self::Model, Error>> + Send;
}

/// An entity that supports notes.
pub trait HasNotes: Findable + Prefixable {}

/// An entity whose display ID can be shortened to a unique prefix.
pub trait Prefixable: Findable {
  /// Compute the minimum unique prefix length for a single entity.
  fn prefix_length(conn: &Connection, project_id: &Id, id: &str) -> impl Future<Output = Result<usize, Error>> + Send;
}

/// An entity that supports tag attachment/detachment.
pub trait Taggable: Findable + Prefixable {}

/// Artifact entity binding for action traits.
pub struct Artifact;

impl Findable for Artifact {
  type Model = artifact::Model;

  fn entity_type() -> EntityType {
    EntityType::Artifact
  }

  async fn find_by_id(conn: &Connection, id: impl Into<Id> + Send) -> Result<artifact::Model, Error> {
    repo::artifact::find_required_by_id(conn, id).await
  }

  fn table() -> Table {
    Table::Artifacts
  }
}

impl HasMetadata for Artifact {
  type Patch = artifact::Patch;

  fn event_table() -> &'static str {
    "artifacts"
  }

  fn metadata(model: &artifact::Model) -> &Value {
    model.metadata()
  }

  fn metadata_patch(metadata: Value) -> artifact::Patch {
    artifact::Patch {
      metadata: Some(metadata),
      ..Default::default()
    }
  }

  async fn update(conn: &Connection, id: &Id, patch: &artifact::Patch) -> Result<artifact::Model, Error> {
    repo::artifact::update(conn, id, patch).await
  }
}

impl HasNotes for Artifact {}

impl Prefixable for Artifact {
  async fn prefix_length(conn: &Connection, project_id: &Id, id: &str) -> Result<usize, Error> {
    let lengths = repo::artifact::prefix_lengths(conn, project_id, &[id]).await?;
    Ok(lengths.into_iter().next().unwrap_or(1))
  }
}

impl Taggable for Artifact {}

/// Iteration entity binding for action traits.
pub struct Iteration;

impl Findable for Iteration {
  type Model = iteration::Model;

  fn entity_type() -> EntityType {
    EntityType::Iteration
  }

  async fn find_by_id(conn: &Connection, id: impl Into<Id> + Send) -> Result<iteration::Model, Error> {
    repo::iteration::find_required_by_id(conn, id).await
  }

  fn table() -> Table {
    Table::Iterations
  }
}

impl HasMetadata for Iteration {
  type Patch = iteration::Patch;

  fn event_table() -> &'static str {
    "iterations"
  }

  fn metadata(model: &iteration::Model) -> &Value {
    model.metadata()
  }

  fn metadata_patch(metadata: Value) -> iteration::Patch {
    iteration::Patch {
      metadata: Some(metadata),
      ..Default::default()
    }
  }

  async fn update(conn: &Connection, id: &Id, patch: &iteration::Patch) -> Result<iteration::Model, Error> {
    repo::iteration::update(conn, id, patch).await
  }
}

impl Prefixable for Iteration {
  async fn prefix_length(conn: &Connection, project_id: &Id, id: &str) -> Result<usize, Error> {
    let lengths = repo::iteration::prefix_lengths_for_project(conn, project_id, &[id]).await?;
    Ok(lengths.into_iter().next().unwrap_or(1))
  }
}

impl Taggable for Iteration {}

/// Task entity binding for action traits.
pub struct Task;

impl Findable for Task {
  type Model = task::Model;

  fn entity_type() -> EntityType {
    EntityType::Task
  }

  async fn find_by_id(conn: &Connection, id: impl Into<Id> + Send) -> Result<task::Model, Error> {
    repo::task::find_required_by_id(conn, id).await
  }

  fn table() -> Table {
    Table::Tasks
  }
}

impl HasMetadata for Task {
  type Patch = task::Patch;

  fn event_table() -> &'static str {
    "tasks"
  }

  fn metadata(model: &task::Model) -> &Value {
    model.metadata()
  }

  fn metadata_patch(metadata: Value) -> task::Patch {
    task::Patch {
      metadata: Some(metadata),
      ..Default::default()
    }
  }

  async fn update(conn: &Connection, id: &Id, patch: &task::Patch) -> Result<task::Model, Error> {
    repo::task::update(conn, id, patch).await
  }
}

impl HasNotes for Task {}

impl Prefixable for Task {
  async fn prefix_length(conn: &Connection, project_id: &Id, id: &str) -> Result<usize, Error> {
    let map = repo::task::per_id_prefix_lengths(conn, project_id).await?;
    Ok(map.get(id).copied().unwrap_or(1))
  }
}

impl Taggable for Task {}
