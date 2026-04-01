use clap::Args;

use crate::{
  cli::{self, AppContext},
  model::EntityType,
  store,
  ui::composites::empty_list::EmptyList,
};

/// List all unique tags, optionally filtered by entity type.
#[derive(Debug, Args)]
pub struct Command {
  /// Show only tags from tasks.
  #[arg(long)]
  task: bool,
  /// Show only tags from artifacts.
  #[arg(long)]
  artifact: bool,
  /// Show only tags from iterations.
  #[arg(long)]
  iteration: bool,
}

impl Command {
  pub fn call(&self, ctx: &AppContext) -> cli::Result<()> {
    let filter = self.entity_type_filter();
    let tags = store::list_tags(&ctx.settings, filter.as_deref())?;

    if tags.is_empty() {
      println!("{}", EmptyList::new("tags", &ctx.theme));
      return Ok(());
    }

    for tag in &tags {
      println!("{tag}");
    }

    Ok(())
  }

  /// Build an entity-type filter from the CLI flags.
  ///
  /// Returns `None` when no flags are set (meaning all types).
  fn entity_type_filter(&self) -> Option<Vec<EntityType>> {
    if !self.task && !self.artifact && !self.iteration {
      return None;
    }

    let mut types = Vec::new();
    if self.task {
      types.push(EntityType::Task);
    }
    if self.artifact {
      types.push(EntityType::Artifact);
    }
    if self.iteration {
      types.push(EntityType::Iteration);
    }
    Some(types)
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::test_helpers::{make_test_artifact, make_test_context, make_test_task};

  mod call {
    use super::*;

    #[test]
    fn it_lists_all_tags_without_filter() {
      let dir = tempfile::tempdir().unwrap();
      let ctx = make_test_context(dir.path());

      let mut task = make_test_task("zyxwvutsrqponmlkzyxwvutsrqponmlk");
      task.tags = vec!["task-tag".to_string()];
      store::write_task(&ctx.settings, &task).unwrap();

      let mut artifact = make_test_artifact("klmnopqrstuvwxyzklmnopqrstuvwxyz");
      artifact.tags = vec!["artifact-tag".to_string()];
      store::write_artifact(&ctx.settings, &artifact).unwrap();

      let cmd = Command {
        task: false,
        artifact: false,
        iteration: false,
      };
      // Just verify it doesn't error — output goes to stdout.
      cmd.call(&ctx).unwrap();
    }

    #[test]
    fn it_filters_to_task_tags_only() {
      let dir = tempfile::tempdir().unwrap();
      let ctx = make_test_context(dir.path());

      let mut task = make_test_task("zyxwvutsrqponmlkzyxwvutsrqponmlk");
      task.tags = vec!["task-tag".to_string()];
      store::write_task(&ctx.settings, &task).unwrap();

      let mut artifact = make_test_artifact("klmnopqrstuvwxyzklmnopqrstuvwxyz");
      artifact.tags = vec!["artifact-tag".to_string()];
      store::write_artifact(&ctx.settings, &artifact).unwrap();

      let cmd = Command {
        task: true,
        artifact: false,
        iteration: false,
      };
      cmd.call(&ctx).unwrap();
    }
  }

  mod entity_type_filter {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_returns_none_when_no_flags() {
      let cmd = Command {
        task: false,
        artifact: false,
        iteration: false,
      };
      assert_eq!(cmd.entity_type_filter(), None);
    }

    #[test]
    fn it_returns_task_when_task_flag() {
      let cmd = Command {
        task: true,
        artifact: false,
        iteration: false,
      };
      assert_eq!(cmd.entity_type_filter(), Some(vec![EntityType::Task]));
    }

    #[test]
    fn it_combines_multiple_flags() {
      let cmd = Command {
        task: true,
        artifact: true,
        iteration: false,
      };
      assert_eq!(
        cmd.entity_type_filter(),
        Some(vec![EntityType::Task, EntityType::Artifact])
      );
    }
  }
}
