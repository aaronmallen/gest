use chrono::Utc;
use clap::Args;

use crate::{
  config,
  config::Config,
  store,
  ui::{components::TagChange, theme::Theme},
};

/// Remove tags from a task
#[derive(Debug, Args)]
pub struct Command {
  /// Task ID or unique prefix
  pub id: String,
  /// Tags to remove (space-separated)
  pub tags: Vec<String>,
}

impl Command {
  pub fn call(&self, config: &Config, _theme: &Theme) -> crate::Result<()> {
    let data_dir = config::data_dir(config)?;
    let id = store::resolve_task_id(&data_dir, &self.id, false)?;
    let mut task = store::read_task(&data_dir, &id)?;

    task.tags.retain(|t| !self.tags.contains(t));

    task.updated_at = Utc::now();
    store::write_task(&data_dir, &task)?;

    TagChange::new("Untagged", "task", &id, &self.tags).write_to(&mut std::io::stdout())?;
    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::test_helpers::{make_test_config, make_test_task};

  mod call {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_can_remove_all_tags() {
      let dir = tempfile::tempdir().unwrap();
      let config = make_test_config(dir.path());
      let mut task = make_test_task("zyxwvutsrqponmlkzyxwvutsrqponmlk");
      task.tags = vec!["rust".to_string(), "cli".to_string()];
      store::write_task(dir.path(), &task).unwrap();

      let cmd = Command {
        id: "zyxw".to_string(),
        tags: vec!["rust".to_string(), "cli".to_string()],
      };
      cmd.call(&config, &Theme::default()).unwrap();

      let loaded = store::read_task(dir.path(), &task.id).unwrap();
      assert!(loaded.tags.is_empty());
    }

    #[test]
    fn it_handles_nonexistent_tags_gracefully() {
      let dir = tempfile::tempdir().unwrap();
      let config = make_test_config(dir.path());
      let mut task = make_test_task("zyxwvutsrqponmlkzyxwvutsrqponmlk");
      task.tags = vec!["rust".to_string()];
      store::write_task(dir.path(), &task).unwrap();

      let cmd = Command {
        id: "zyxw".to_string(),
        tags: vec!["nonexistent".to_string()],
      };
      cmd.call(&config, &Theme::default()).unwrap();

      let loaded = store::read_task(dir.path(), &task.id).unwrap();
      assert_eq!(loaded.tags, vec!["rust".to_string()]);
    }

    #[test]
    fn it_removes_tags() {
      let dir = tempfile::tempdir().unwrap();
      let config = make_test_config(dir.path());
      let mut task = make_test_task("zyxwvutsrqponmlkzyxwvutsrqponmlk");
      task.tags = vec!["rust".to_string(), "cli".to_string(), "keep".to_string()];
      store::write_task(dir.path(), &task).unwrap();

      let cmd = Command {
        id: "zyxw".to_string(),
        tags: vec!["rust".to_string(), "cli".to_string()],
      };
      cmd.call(&config, &Theme::default()).unwrap();

      let loaded = store::read_task(dir.path(), &task.id).unwrap();
      assert_eq!(loaded.tags, vec!["keep".to_string()]);
    }
  }
}
