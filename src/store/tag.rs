use chrono::Utc;

use crate::{
  config::Settings,
  model::{EntityType, Id},
};

/// Parameters for a tag or untag operation.
pub struct TagParams<'a> {
  /// The entity type to tag.
  pub entity_type: EntityType,
  /// The ID prefix of the entity to tag.
  pub id_prefix: &'a str,
  /// The tags to add or remove.
  pub tags: &'a [String],
}

/// The result of a successful tag/untag operation.
pub struct TagResult {
  /// The resolved full ID of the entity that was tagged/untagged.
  pub id: Id,
}

/// Append tags from `to_add` into `tags`, skipping duplicates.
pub fn apply_tags(tags: &mut Vec<String>, to_add: &[String]) {
  for tag in to_add {
    if !tags.contains(tag) {
      tags.push(tag.clone());
    }
  }
}

/// Remove all entries in `to_remove` from `tags`.
pub fn remove_tags(tags: &mut Vec<String>, to_remove: &[String]) {
  tags.retain(|t| !to_remove.contains(t));
}

/// Add tags to the entity identified by `params`.
pub fn tag_entity(config: &Settings, params: &TagParams) -> super::Result<TagResult> {
  match params.entity_type {
    EntityType::Artifact => {
      let id = super::resolve_artifact_id(config, params.id_prefix, false)?;
      let mut artifact = super::read_artifact(config, &id)?;
      apply_tags(&mut artifact.tags, params.tags);
      artifact.updated_at = Utc::now();
      super::write_artifact(config, &artifact)?;
      Ok(TagResult {
        id,
      })
    }
    EntityType::Iteration => {
      let id = super::resolve_iteration_id(config, params.id_prefix, false)?;
      let mut iteration = super::read_iteration(config, &id)?;
      apply_tags(&mut iteration.tags, params.tags);
      iteration.updated_at = Utc::now();
      super::write_iteration(config, &iteration)?;
      Ok(TagResult {
        id,
      })
    }
    EntityType::Task => {
      let id = super::resolve_task_id(config, params.id_prefix, false)?;
      let mut task = super::read_task(config, &id)?;
      apply_tags(&mut task.tags, params.tags);
      task.updated_at = Utc::now();
      super::write_task(config, &task)?;
      Ok(TagResult {
        id,
      })
    }
  }
}

/// Remove tags from the entity identified by `params`.
pub fn untag_entity(config: &Settings, params: &TagParams) -> super::Result<TagResult> {
  match params.entity_type {
    EntityType::Artifact => {
      let id = super::resolve_artifact_id(config, params.id_prefix, false)?;
      let mut artifact = super::read_artifact(config, &id)?;
      remove_tags(&mut artifact.tags, params.tags);
      artifact.updated_at = Utc::now();
      super::write_artifact(config, &artifact)?;
      Ok(TagResult {
        id,
      })
    }
    EntityType::Iteration => {
      let id = super::resolve_iteration_id(config, params.id_prefix, false)?;
      let mut iteration = super::read_iteration(config, &id)?;
      remove_tags(&mut iteration.tags, params.tags);
      iteration.updated_at = Utc::now();
      super::write_iteration(config, &iteration)?;
      Ok(TagResult {
        id,
      })
    }
    EntityType::Task => {
      let id = super::resolve_task_id(config, params.id_prefix, false)?;
      let mut task = super::read_task(config, &id)?;
      remove_tags(&mut task.tags, params.tags);
      task.updated_at = Utc::now();
      super::write_task(config, &task)?;
      Ok(TagResult {
        id,
      })
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  mod apply_tags {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_adds_new_tags() {
      let mut tags = vec!["a".to_string()];
      apply_tags(&mut tags, &["b".to_string(), "c".to_string()]);

      assert_eq!(tags, vec!["a", "b", "c"]);
    }

    #[test]
    fn it_skips_duplicates() {
      let mut tags = vec!["a".to_string(), "b".to_string()];
      apply_tags(&mut tags, &["b".to_string(), "c".to_string()]);

      assert_eq!(tags, vec!["a", "b", "c"]);
    }
  }

  mod remove_tags {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_filters_matching_tags() {
      let mut tags = vec!["a".to_string(), "b".to_string(), "c".to_string()];
      remove_tags(&mut tags, &["b".to_string()]);

      assert_eq!(tags, vec!["a", "c"]);
    }

    #[test]
    fn it_ignores_absent_tags() {
      let mut tags = vec!["a".to_string()];
      remove_tags(&mut tags, &["z".to_string()]);

      assert_eq!(tags, vec!["a"]);
    }
  }

  mod tag_entity {
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::test_helpers::{make_test_artifact, make_test_config, make_test_task};

    #[test]
    fn it_tags_a_task() {
      let dir = tempfile::tempdir().unwrap();
      let config = make_test_config(dir.path().to_path_buf());
      let task = make_test_task("zyxwvutsrqponmlkzyxwvutsrqponmlk");
      crate::store::write_task(&config, &task).unwrap();

      let params = TagParams {
        entity_type: EntityType::Task,
        id_prefix: "zyxw",
        tags: &["rust".to_string(), "cli".to_string()],
      };
      let result = tag_entity(&config, &params).unwrap();

      assert_eq!(result.id.to_string(), "zyxwvutsrqponmlkzyxwvutsrqponmlk");
      let loaded = crate::store::read_task(&config, &task.id).unwrap();
      assert_eq!(loaded.tags, vec!["rust".to_string(), "cli".to_string()]);
    }

    #[test]
    fn it_tags_an_artifact() {
      let dir = tempfile::tempdir().unwrap();
      let config = make_test_config(dir.path().to_path_buf());
      let artifact = make_test_artifact("zyxwvutsrqponmlkzyxwvutsrqponmlk");
      crate::store::write_artifact(&config, &artifact).unwrap();

      let params = TagParams {
        entity_type: EntityType::Artifact,
        id_prefix: "zyxw",
        tags: &["spec".to_string()],
      };
      let result = tag_entity(&config, &params).unwrap();

      assert_eq!(result.id.to_string(), "zyxwvutsrqponmlkzyxwvutsrqponmlk");
      let loaded = crate::store::read_artifact(&config, &artifact.id).unwrap();
      assert_eq!(loaded.tags, vec!["spec".to_string()]);
    }
  }

  mod untag_entity {
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::test_helpers::{make_test_config, make_test_task};

    #[test]
    fn it_removes_tags_from_a_task() {
      let dir = tempfile::tempdir().unwrap();
      let config = make_test_config(dir.path().to_path_buf());
      let mut task = make_test_task("zyxwvutsrqponmlkzyxwvutsrqponmlk");
      task.tags = vec!["rust".to_string(), "cli".to_string(), "keep".to_string()];
      crate::store::write_task(&config, &task).unwrap();

      let params = TagParams {
        entity_type: EntityType::Task,
        id_prefix: "zyxw",
        tags: &["rust".to_string(), "cli".to_string()],
      };
      let result = untag_entity(&config, &params).unwrap();

      assert_eq!(result.id.to_string(), "zyxwvutsrqponmlkzyxwvutsrqponmlk");
      let loaded = crate::store::read_task(&config, &task.id).unwrap();
      assert_eq!(loaded.tags, vec!["keep".to_string()]);
    }
  }
}
