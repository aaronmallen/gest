use chrono::Utc;
use clap::Args;

use crate::{
  config,
  config::Config,
  model::{Link, RelationshipType},
  store,
  ui::{components::LinkAdded, theme::Theme},
};

/// Create a relationship between a task and another task or artifact
#[derive(Debug, Args)]
pub struct Command {
  /// Source task ID or unique prefix
  pub id: String,
  /// Relationship type: blocks, blocked-by, child-of, parent-of, or relates-to
  #[arg(value_enum)]
  pub rel: RelationshipType,
  /// Target task or artifact ID or unique prefix
  pub target_id: String,
  /// Target is an artifact instead of a task (no reciprocal link is created)
  #[arg(long)]
  pub artifact: bool,
}

impl Command {
  pub fn call(&self, config: &Config, _theme: &Theme) -> crate::Result<()> {
    let data_dir = config::data_dir(config)?;
    let id = store::resolve_task_id(&data_dir, &self.id, false)?;

    let target_id = if self.artifact {
      store::resolve_artifact_id(&data_dir, &self.target_id, true)?
    } else {
      store::resolve_task_id(&data_dir, &self.target_id, true)?
    };

    let ref_path = if self.artifact {
      format!("artifacts/{target_id}")
    } else {
      format!("tasks/{target_id}")
    };

    // Add link on source task
    let mut task = store::read_task(&data_dir, &id)?;
    task.links.push(Link {
      ref_: ref_path,
      rel: self.rel.clone(),
    });
    task.updated_at = Utc::now();
    store::write_task(&data_dir, &task)?;

    // Add reciprocal link on target (task-to-task only)
    if !self.artifact {
      let mut target_task = store::read_task(&data_dir, &target_id)?;
      target_task.links.push(Link {
        ref_: format!("tasks/{id}"),
        rel: self.rel.inverse(),
      });
      target_task.updated_at = Utc::now();
      store::write_task(&data_dir, &target_task)?;
    }

    LinkAdded::new(&id, &self.rel.to_string(), &target_id).write_to(&mut std::io::stdout())?;
    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use tempfile::TempDir;

  use super::*;
  use crate::{
    model::RelationshipType,
    test_helpers::{make_test_artifact, make_test_config, make_test_task},
  };

  mod call {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_appends_multiple_links() {
      let (_dir, config) = setup();
      let source = make_test_task("zyxwvutsrqponmlkzyxwvutsrqponmlk");
      let target1 = make_test_task("kkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkk");
      let target2 = make_test_task("nnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnn");
      store::write_task(_dir.path(), &source).unwrap();
      store::write_task(_dir.path(), &target1).unwrap();
      store::write_task(_dir.path(), &target2).unwrap();

      let cmd1 = Command {
        id: "zyxw".to_string(),
        rel: RelationshipType::Blocks,
        target_id: "kkkk".to_string(),
        artifact: false,
      };
      cmd1.call(&config, &Theme::default()).unwrap();

      let cmd2 = Command {
        id: "zyxw".to_string(),
        rel: RelationshipType::RelatesTo,
        target_id: "nnnn".to_string(),
        artifact: false,
      };
      cmd2.call(&config, &Theme::default()).unwrap();

      let loaded = store::read_task(_dir.path(), &source.id).unwrap();
      assert_eq!(loaded.links.len(), 2);
    }

    #[test]
    fn it_errors_when_target_not_found() {
      let (_dir, config) = setup();
      let source = make_test_task("zyxwvutsrqponmlkzyxwvutsrqponmlk");
      store::write_task(_dir.path(), &source).unwrap();

      let cmd = Command {
        id: "zyxw".to_string(),
        rel: RelationshipType::Blocks,
        target_id: "zzzz".to_string(),
        artifact: false,
      };
      let result = cmd.call(&config, &Theme::default());
      assert!(result.is_err());
    }

    #[test]
    fn it_links_task_to_artifact() {
      let (_dir, config) = setup();
      let source = make_test_task("zyxwvutsrqponmlkzyxwvutsrqponmlk");
      let target = make_test_artifact("kkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkk");
      store::write_task(_dir.path(), &source).unwrap();
      store::write_artifact(_dir.path(), &target).unwrap();

      let cmd = Command {
        id: "zyxw".to_string(),
        rel: RelationshipType::RelatesTo,
        target_id: "kkkk".to_string(),
        artifact: true,
      };
      cmd.call(&config, &Theme::default()).unwrap();

      let loaded = store::read_task(_dir.path(), &source.id).unwrap();
      assert_eq!(loaded.links.len(), 1);
      assert_eq!(loaded.links[0].rel, RelationshipType::RelatesTo);
      assert_eq!(loaded.links[0].ref_, "artifacts/kkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkk");
    }

    #[test]
    fn it_links_task_to_task() {
      let (_dir, config) = setup();
      let source = make_test_task("zyxwvutsrqponmlkzyxwvutsrqponmlk");
      let target = make_test_task("kkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkk");
      store::write_task(_dir.path(), &source).unwrap();
      store::write_task(_dir.path(), &target).unwrap();

      let cmd = Command {
        id: "zyxw".to_string(),
        rel: RelationshipType::Blocks,
        target_id: "kkkk".to_string(),
        artifact: false,
      };
      cmd.call(&config, &Theme::default()).unwrap();

      let loaded = store::read_task(_dir.path(), &source.id).unwrap();
      assert_eq!(loaded.links.len(), 1);
      assert_eq!(loaded.links[0].rel, RelationshipType::Blocks);
      assert_eq!(loaded.links[0].ref_, "tasks/kkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkk");

      // Verify reciprocal link on target
      let loaded_target = store::read_task(_dir.path(), &target.id).unwrap();
      assert_eq!(loaded_target.links.len(), 1);
      assert_eq!(loaded_target.links[0].rel, RelationshipType::BlockedBy);
      assert_eq!(loaded_target.links[0].ref_, "tasks/zyxwvutsrqponmlkzyxwvutsrqponmlk");
    }

    #[test]
    fn it_creates_reciprocal_relates_to_on_both() {
      let (_dir, config) = setup();
      let source = make_test_task("zyxwvutsrqponmlkzyxwvutsrqponmlk");
      let target = make_test_task("kkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkk");
      store::write_task(_dir.path(), &source).unwrap();
      store::write_task(_dir.path(), &target).unwrap();

      let cmd = Command {
        id: "zyxw".to_string(),
        rel: RelationshipType::RelatesTo,
        target_id: "kkkk".to_string(),
        artifact: false,
      };
      cmd.call(&config, &Theme::default()).unwrap();

      let loaded_source = store::read_task(_dir.path(), &source.id).unwrap();
      assert_eq!(loaded_source.links[0].rel, RelationshipType::RelatesTo);

      let loaded_target = store::read_task(_dir.path(), &target.id).unwrap();
      assert_eq!(loaded_target.links[0].rel, RelationshipType::RelatesTo);
    }

    #[test]
    fn it_does_not_create_reciprocal_for_artifact_links() {
      let (_dir, config) = setup();
      let source = make_test_task("zyxwvutsrqponmlkzyxwvutsrqponmlk");
      let target = make_test_artifact("kkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkk");
      store::write_task(_dir.path(), &source).unwrap();
      store::write_artifact(_dir.path(), &target).unwrap();

      let cmd = Command {
        id: "zyxw".to_string(),
        rel: RelationshipType::RelatesTo,
        target_id: "kkkk".to_string(),
        artifact: true,
      };
      cmd.call(&config, &Theme::default()).unwrap();

      // Source should have the link
      let loaded = store::read_task(_dir.path(), &source.id).unwrap();
      assert_eq!(loaded.links.len(), 1);
      // No reciprocal on artifacts (they don't have links)
    }
  }

  fn setup() -> (TempDir, crate::config::Config) {
    let dir = TempDir::new().unwrap();
    let config = make_test_config(dir.path());
    (dir, config)
  }
}
