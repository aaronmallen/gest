use clap::Args;

use crate::{
  action,
  cli::{self, AppContext},
  model::task::Task,
};

/// Set a metadata value on a task using a dot-delimited key path.
#[derive(Debug, Args)]
pub struct Command {
  /// Task ID or unique prefix.
  pub id: String,
  /// Dot-delimited key path (e.g. `outer.inner`).
  pub path: String,
  /// Value to set (strings, numbers, and booleans are auto-detected).
  pub value: String,
}

impl Command {
  /// Write the value into the task's metadata table and persist.
  pub fn call(&self, ctx: &AppContext) -> cli::Result<()> {
    action::meta::meta_set::<Task>(ctx, &self.id, &self.path, &self.value)
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{
    store,
    test_helpers::{make_test_context, make_test_task},
  };

  #[test]
  fn it_sets_metadata_value() {
    let dir = tempfile::tempdir().unwrap();
    let ctx = make_test_context(dir.path());
    let task = make_test_task("zyxwvutsrqponmlkzyxwvutsrqponmlk");
    store::write_task(&ctx.settings, &task).unwrap();

    let cmd = Command {
      id: "zyxw".to_string(),
      path: "priority".to_string(),
      value: "high".to_string(),
    };
    cmd.call(&ctx).unwrap();

    let loaded = store::read_task(&ctx.settings, &task.id).unwrap();
    assert_eq!(
      loaded.metadata.get("priority"),
      Some(&toml::Value::String("high".to_string()))
    );
  }

  #[test]
  fn it_sets_nested_metadata_value() {
    let dir = tempfile::tempdir().unwrap();
    let ctx = make_test_context(dir.path());
    let task = make_test_task("zyxwvutsrqponmlkzyxwvutsrqponmlk");
    store::write_task(&ctx.settings, &task).unwrap();

    let cmd = Command {
      id: "zyxw".to_string(),
      path: "config.timeout".to_string(),
      value: "30".to_string(),
    };
    cmd.call(&ctx).unwrap();

    let loaded = store::read_task(&ctx.settings, &task.id).unwrap();
    let config = loaded.metadata.get("config").unwrap().as_table().unwrap();
    assert_eq!(config.get("timeout"), Some(&toml::Value::Integer(30)));
  }
}
