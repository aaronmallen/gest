use clap::Args;

use crate::{
  cli::{self, AppContext},
  model::EntityType,
  store,
};

/// Remove tags from any entity (task, artifact, or iteration) by ID prefix.
#[derive(Debug, Args)]
pub struct Command {
  /// Entity ID or unique prefix.
  pub id: String,
  /// Tags to remove (space-separated).
  pub tags: Vec<String>,
}

impl Command {
  pub fn call(&self, ctx: &AppContext) -> cli::Result<()> {
    let resolved = store::resolve_any_id(&ctx.settings, &self.id)?;

    match resolved.entity_type {
      EntityType::Task => super::super::tags::untag_entity(
        ctx,
        &self.id,
        &self.tags,
        "task",
        store::resolve_task_id,
        store::read_task,
        |t| &mut t.tags,
        |t, ts| t.updated_at = ts,
        store::write_task,
      ),
      EntityType::Artifact => super::super::tags::untag_entity(
        ctx,
        &self.id,
        &self.tags,
        "artifact",
        store::resolve_artifact_id,
        store::read_artifact,
        |a| &mut a.tags,
        |a, ts| a.updated_at = ts,
        store::write_artifact,
      ),
      EntityType::Iteration => super::super::tags::untag_entity(
        ctx,
        &self.id,
        &self.tags,
        "iteration",
        store::resolve_iteration_id,
        store::read_iteration,
        |i| &mut i.tags,
        |i, ts| i.updated_at = ts,
        store::write_iteration,
      ),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::test_helpers::{make_test_context, make_test_task};

  mod call {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_removes_tags_from_a_task() {
      let dir = tempfile::tempdir().unwrap();
      let ctx = make_test_context(dir.path());
      let mut task = make_test_task("zyxwvutsrqponmlkzyxwvutsrqponmlk");
      task.tags = vec!["rust".to_string(), "cli".to_string(), "keep".to_string()];
      store::write_task(&ctx.settings, &task).unwrap();

      let cmd = Command {
        id: "zyxw".to_string(),
        tags: vec!["rust".to_string(), "cli".to_string()],
      };
      cmd.call(&ctx).unwrap();

      let loaded = store::read_task(&ctx.settings, &task.id).unwrap();
      assert_eq!(loaded.tags, vec!["keep".to_string()]);
    }
  }
}
