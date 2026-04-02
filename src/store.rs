//! Persistence layer for reading, writing, and querying entities on disk.

mod artifact;
pub mod artifact_meta;
mod fs;
mod helpers;
mod iteration;
pub mod meta;
pub(crate) mod meta_value;
pub mod note;
mod orchestration;
mod search;
pub mod search_query;
mod tag;
mod task;

use std::io;

use crate::{
  config::Settings,
  model::{EntityType, Id},
};

/// A successfully resolved entity ID together with its type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedEntity {
  pub entity_type: EntityType,
  pub id: Id,
}

/// Errors that can occur during store operations.
#[derive(Debug, thiserror::Error)]
pub enum Error {
  #[error("{0}")]
  AmbiguousId(String),
  #[error("failed to generate a unique ID after {0} attempts")]
  IdExhausted(usize),
  #[error("{0}")]
  InvalidFormat(String),
  #[error("{0}")]
  InvalidId(String),
  #[error(transparent)]
  Io(#[from] io::Error),
  #[error("{0}")]
  NotFound(String),
  #[error("{0}")]
  PhaseAdvance(String),
  #[error(transparent)]
  TomlDe(#[from] toml::de::Error),
  #[error(transparent)]
  TomlSer(#[from] toml::ser::Error),
  #[error(transparent)]
  Yaml(#[from] yaml_serde::Error),
}

/// Convenience alias for store operations.
pub type Result<T> = std::result::Result<T, Error>;

pub use artifact::{
  archive_artifact, create_artifact, list_artifacts, read_artifact, resolve_artifact_id, update_artifact,
  write_artifact,
};
#[allow(unused_imports)] // used in tests via crate::store::ensure_dirs
pub use fs::ensure_dirs;
#[allow(unused_imports)] // is_iteration_resolved, resolve_iteration used in tests
pub use iteration::{
  add_task as add_iteration_task, create_iteration, is_iteration_resolved, list_iterations, read_iteration,
  read_iteration_tasks, remove_task as remove_iteration_task, resolve_iteration, resolve_iteration_id,
  update_iteration, write_iteration,
};
#[allow(unused_imports)] // AdvanceSummary, OverallProgress, PhaseProgress used in tests
pub use orchestration::{
  AdvanceSummary, IterationProgress, OverallProgress, PhaseProgress, advance_phase, claim_task, iteration_status,
  next_available_task,
};
pub use search::{SearchResults, search};
#[allow(unused_imports)] // TagResult used by callers of tag_entity/untag_entity
pub use tag::{TagParams, TagResult, tag_entity, untag_entity};
#[allow(unused_imports)] // is_task_resolved, resolve_task used in tests
pub use task::{
  ResolvedBlocking, create_task, is_task_resolved, list_tasks, read_task, resolve_blocking, resolve_blocking_batch,
  resolve_task, resolve_task_id, update_task, write_task,
};

/// Collect every unique tag used across tasks, artifacts, and iterations, sorted alphabetically.
///
/// When `entity_types` is `None` (or an empty slice), tags are collected from all entity types.
/// Otherwise only the specified types are queried.
pub fn list_tags(
  config: &crate::config::Settings,
  entity_types: Option<&[crate::model::EntityType]>,
) -> Result<Vec<String>> {
  use std::collections::BTreeSet;

  use crate::model::EntityType;

  let include_all = entity_types.is_none_or(|t| t.is_empty());

  let mut tags: BTreeSet<String> = BTreeSet::new();

  if include_all || entity_types.is_some_and(|t| t.contains(&EntityType::Task)) {
    let task_filter = crate::model::TaskFilter {
      all: true,
      ..Default::default()
    };
    for task in list_tasks(config, &task_filter)? {
      tags.extend(task.tags);
    }
  }

  if include_all || entity_types.is_some_and(|t| t.contains(&EntityType::Artifact)) {
    let artifact_filter = crate::model::ArtifactFilter {
      all: true,
      ..Default::default()
    };
    for artifact in list_artifacts(config, &artifact_filter)? {
      tags.extend(artifact.tags);
    }
  }

  if include_all || entity_types.is_some_and(|t| t.contains(&EntityType::Iteration)) {
    let iteration_filter = crate::model::IterationFilter {
      all: true,
      ..Default::default()
    };
    for iteration in list_iterations(config, &iteration_filter)? {
      tags.extend(iteration.tags);
    }
  }

  Ok(tags.into_iter().collect())
}

/// Resolve an ID prefix across tasks, artifacts, and iterations.
///
/// Returns the full [`Id`] and [`EntityType`] when the prefix matches exactly one
/// entity type. Returns an error with disambiguation info when the prefix matches
/// multiple entity types, or a "not found" error when it matches none.
///
/// Resolved/archived entities are always included in the search.
pub fn resolve_any_id(config: &Settings, prefix: &str) -> Result<ResolvedEntity> {
  Id::validate_prefix(prefix).map_err(Error::InvalidId)?;

  let mut matches: Vec<(EntityType, Id)> = Vec::new();

  if let Ok(id) = resolve_task_id(config, prefix, true) {
    matches.push((EntityType::Task, id));
  }
  if let Ok(id) = resolve_artifact_id(config, prefix, true) {
    matches.push((EntityType::Artifact, id));
  }
  if let Ok(id) = resolve_iteration_id(config, prefix, true) {
    matches.push((EntityType::Iteration, id));
  }

  match matches.len() {
    0 => Err(Error::NotFound(format!("No entity found matching '{prefix}'"))),
    1 => {
      let (entity_type, id) = matches.remove(0);
      Ok(ResolvedEntity {
        entity_type,
        id,
      })
    }
    _ => {
      let types: Vec<String> = matches.iter().map(|(et, id)| format!("{et} ({id})")).collect();
      Err(Error::AmbiguousId(format!(
        "Ambiguous ID prefix '{prefix}' matches multiple entity types: {}",
        types.join(", ")
      )))
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::config::Settings;

  fn make_config(base: &std::path::Path) -> Settings {
    crate::test_helpers::make_test_config(base.to_path_buf())
  }

  mod resolve_any_id {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_errors_when_no_entity_matches() {
      let dir = tempfile::tempdir().unwrap();
      let config = make_config(dir.path());
      ensure_dirs(&config).unwrap();

      let result = resolve_any_id(&config, "zyxw");

      assert!(result.is_err());
      let err = result.unwrap_err().to_string();
      assert!(err.contains("No entity found"), "Expected not-found error, got: {err}");
    }

    #[test]
    fn it_errors_with_disambiguation_when_multiple_types_match() {
      let dir = tempfile::tempdir().unwrap();
      let config = make_config(dir.path());
      let task = crate::test_helpers::make_test_task("zyxwvutsrqponmlkzyxwvutsrqponmlk");
      let artifact = crate::test_helpers::make_test_artifact("zyxwvutsrqponmlkzyxwvutsrqponmlk");
      write_task(&config, &task).unwrap();
      write_artifact(&config, &artifact).unwrap();

      let result = resolve_any_id(&config, "zyxw");

      assert!(result.is_err());
      let err = result.unwrap_err().to_string();
      assert!(err.contains("Ambiguous"), "Expected ambiguity error, got: {err}");
      assert!(err.contains("task"), "Expected task in disambiguation, got: {err}");
      assert!(
        err.contains("artifact"),
        "Expected artifact in disambiguation, got: {err}"
      );
    }

    #[test]
    fn it_includes_archived_artifacts() {
      let dir = tempfile::tempdir().unwrap();
      let config = make_config(dir.path());
      let artifact = crate::test_helpers::make_test_artifact("zyxwvutsrqponmlkzyxwvutsrqponmlk");
      write_artifact(&config, &artifact).unwrap();
      archive_artifact(&config, &artifact.id).unwrap();

      let resolved = resolve_any_id(&config, "zyxw").unwrap();

      assert_eq!(resolved.entity_type, EntityType::Artifact);
    }

    #[test]
    fn it_includes_resolved_iterations() {
      let dir = tempfile::tempdir().unwrap();
      let config = make_config(dir.path());
      let iteration = crate::test_helpers::make_test_iteration("zyxwvutsrqponmlkzyxwvutsrqponmlk");
      write_iteration(&config, &iteration).unwrap();
      resolve_iteration(&config, &iteration.id).unwrap();

      let resolved = resolve_any_id(&config, "zyxw").unwrap();

      assert_eq!(resolved.entity_type, EntityType::Iteration);
    }

    #[test]
    fn it_includes_resolved_tasks() {
      let dir = tempfile::tempdir().unwrap();
      let config = make_config(dir.path());
      let task = crate::test_helpers::make_test_task("zyxwvutsrqponmlkzyxwvutsrqponmlk");
      write_task(&config, &task).unwrap();
      resolve_task(&config, &task.id).unwrap();

      let resolved = resolve_any_id(&config, "zyxw").unwrap();

      assert_eq!(resolved.entity_type, EntityType::Task);
    }

    #[test]
    fn it_resolves_a_task_prefix() {
      let dir = tempfile::tempdir().unwrap();
      let config = make_config(dir.path());
      let task = crate::test_helpers::make_test_task("zyxwvutsrqponmlkzyxwvutsrqponmlk");
      write_task(&config, &task).unwrap();

      let resolved = resolve_any_id(&config, "zyxw").unwrap();

      assert_eq!(resolved.entity_type, EntityType::Task);
      assert_eq!(resolved.id.to_string(), "zyxwvutsrqponmlkzyxwvutsrqponmlk");
    }

    #[test]
    fn it_resolves_an_artifact_prefix() {
      let dir = tempfile::tempdir().unwrap();
      let config = make_config(dir.path());
      let artifact = crate::test_helpers::make_test_artifact("zyxwvutsrqponmlkzyxwvutsrqponmlk");
      write_artifact(&config, &artifact).unwrap();

      let resolved = resolve_any_id(&config, "zyxw").unwrap();

      assert_eq!(resolved.entity_type, EntityType::Artifact);
      assert_eq!(resolved.id.to_string(), "zyxwvutsrqponmlkzyxwvutsrqponmlk");
    }

    #[test]
    fn it_resolves_an_iteration_prefix() {
      let dir = tempfile::tempdir().unwrap();
      let config = make_config(dir.path());
      let iteration = crate::test_helpers::make_test_iteration("zyxwvutsrqponmlkzyxwvutsrqponmlk");
      write_iteration(&config, &iteration).unwrap();

      let resolved = resolve_any_id(&config, "zyxw").unwrap();

      assert_eq!(resolved.entity_type, EntityType::Iteration);
      assert_eq!(resolved.id.to_string(), "zyxwvutsrqponmlkzyxwvutsrqponmlk");
    }
  }
}
