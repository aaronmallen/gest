use clap::Args;

use crate::{
  action,
  cli::{self, AppContext},
  model::task::Task,
};

/// Get a metadata value from a task by dot-delimited key path.
#[derive(Debug, Args)]
pub struct Command {
  /// Task ID or unique prefix.
  pub id: String,
  /// Dot-delimited key path (e.g. `outer.inner`).
  pub path: String,
}

impl Command {
  /// Resolve the task, look up the metadata key, and print the value.
  pub fn call(&self, ctx: &AppContext) -> cli::Result<()> {
    action::meta::meta_get::<Task>(ctx, &self.id, &self.path, ctx.theme.task_detail_value)
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
  fn it_errors_on_missing_path() {
    let dir = tempfile::tempdir().unwrap();
    let ctx = make_test_context(dir.path());
    let task = make_test_task("zyxwvutsrqponmlkzyxwvutsrqponmlk");
    store::write_task(&ctx.settings, &task).unwrap();

    let cmd = Command {
      id: "zyxw".to_string(),
      path: "nonexistent".to_string(),
    };
    let result = cmd.call(&ctx);

    assert!(result.is_err());
  }

  #[test]
  fn it_reads_metadata_value() {
    let dir = tempfile::tempdir().unwrap();
    let ctx = make_test_context(dir.path());
    let mut task = make_test_task("zyxwvutsrqponmlkzyxwvutsrqponmlk");
    task
      .metadata
      .insert("priority".to_string(), toml::Value::String("high".to_string()));
    store::write_task(&ctx.settings, &task).unwrap();

    let cmd = Command {
      id: "zyxw".to_string(),
      path: "priority".to_string(),
    };
    cmd.call(&ctx).unwrap();
  }
}
