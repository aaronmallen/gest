use clap::Args;

use crate::{
  config,
  config::Config,
  store,
  ui::{components::TaskDetail, theme::Theme},
};

/// Display a task's full details, description, and links
#[derive(Debug, Args)]
pub struct Command {
  /// Task ID or unique prefix
  pub id: String,
  /// Output task details as JSON
  #[arg(short, long)]
  pub json: bool,
}

impl Command {
  pub fn call(&self, config: &Config, theme: &Theme) -> crate::Result<()> {
    let data_dir = config::data_dir(config)?;
    let id = store::resolve_task_id(&data_dir, &self.id, true)?;
    let task = store::read_task(&data_dir, &id)?;

    if self.json {
      let json = serde_json::to_string_pretty(&task)?;
      println!("{json}");
      return Ok(());
    }

    TaskDetail::new(&task).write_to(&mut std::io::stdout(), theme)?;

    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{
    model::{Link, RelationshipType},
    store,
    test_helpers::{make_test_config, make_test_task},
  };

  mod call {
    use super::*;

    #[test]
    fn it_resolves_resolved_task() {
      let dir = tempfile::tempdir().unwrap();
      let config = make_test_config(dir.path());
      let mut task = make_test_task("zyxwvutsrqponmlkzyxwvutsrqponmlk");
      task.description = "A test description".to_string();
      task.tags = vec!["rust".to_string()];
      task.links = vec![Link {
        ref_: "https://example.com".to_string(),
        rel: RelationshipType::RelatesTo,
      }];
      task.title = "Test Task".to_string();
      store::write_task(dir.path(), &task).unwrap();
      store::resolve_task(dir.path(), &task.id).unwrap();

      let cmd = Command {
        id: "zyxw".to_string(),
        json: false,
      };

      cmd.call(&config, &Theme::default()).unwrap();
    }

    #[test]
    fn it_shows_task_as_json() {
      let dir = tempfile::tempdir().unwrap();
      let config = make_test_config(dir.path());
      let task = make_test_task("zyxwvutsrqponmlkzyxwvutsrqponmlk");
      store::write_task(dir.path(), &task).unwrap();

      let cmd = Command {
        id: "zyxw".to_string(),
        json: true,
      };

      cmd.call(&config, &Theme::default()).unwrap();
    }

    #[test]
    fn it_shows_task_detail() {
      let dir = tempfile::tempdir().unwrap();
      let config = make_test_config(dir.path());
      let task = make_test_task("zyxwvutsrqponmlkzyxwvutsrqponmlk");
      store::write_task(dir.path(), &task).unwrap();

      let cmd = Command {
        id: "zyxw".to_string(),
        json: false,
      };

      cmd.call(&config, &Theme::default()).unwrap();
    }
  }
}
