use chrono::Utc;
use clap::Args;

use crate::{
  config,
  config::Config,
  store,
  ui::{components::TagChange, theme::Theme},
};

/// Add tags to a task
#[derive(Debug, Args)]
pub struct Command {
  /// Task ID or unique prefix
  pub id: String,
  /// Tags to add (space-separated)
  pub tags: Vec<String>,
}

impl Command {
  pub fn call(&self, config: &Config, _theme: &Theme) -> crate::Result<()> {
    let data_dir = config::data_dir(config)?;
    let id = store::resolve_task_id(&data_dir, &self.id, false)?;
    let mut task = store::read_task(&data_dir, &id)?;

    for tag in &self.tags {
      if !task.tags.contains(tag) {
        task.tags.push(tag.clone());
      }
    }

    task.updated_at = Utc::now();
    store::write_task(&data_dir, &task)?;

    TagChange::new("Tagged", "task", &id, &self.tags).write_to(&mut std::io::stdout())?;
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
    fn it_adds_tags() {
      let dir = tempfile::tempdir().unwrap();
      let config = make_test_config(dir.path());
      let task = make_test_task("zyxwvutsrqponmlkzyxwvutsrqponmlk");
      store::write_task(dir.path(), &task).unwrap();

      let cmd = Command {
        id: "zyxw".to_string(),
        tags: vec!["rust".to_string(), "cli".to_string()],
      };
      cmd.call(&config, &Theme::default()).unwrap();

      let loaded = store::read_task(dir.path(), &task.id).unwrap();
      assert_eq!(loaded.tags, vec!["rust".to_string(), "cli".to_string()]);
    }

    #[test]
    fn it_deduplicates_tags() {
      let dir = tempfile::tempdir().unwrap();
      let config = make_test_config(dir.path());
      let mut task = make_test_task("zyxwvutsrqponmlkzyxwvutsrqponmlk");
      task.tags = vec!["rust".to_string()];
      store::write_task(dir.path(), &task).unwrap();

      let cmd = Command {
        id: "zyxw".to_string(),
        tags: vec!["rust".to_string(), "cli".to_string()],
      };
      cmd.call(&config, &Theme::default()).unwrap();

      let loaded = store::read_task(dir.path(), &task.id).unwrap();
      assert_eq!(loaded.tags, vec!["rust".to_string(), "cli".to_string()]);
    }

    #[test]
    fn it_preserves_existing_tags() {
      let dir = tempfile::tempdir().unwrap();
      let config = make_test_config(dir.path());
      let mut task = make_test_task("zyxwvutsrqponmlkzyxwvutsrqponmlk");
      task.tags = vec!["existing".to_string()];
      store::write_task(dir.path(), &task).unwrap();

      let cmd = Command {
        id: "zyxw".to_string(),
        tags: vec!["new".to_string()],
      };
      cmd.call(&config, &Theme::default()).unwrap();

      let loaded = store::read_task(dir.path(), &task.id).unwrap();
      assert_eq!(loaded.tags, vec!["existing".to_string(), "new".to_string()]);
    }
  }
}
