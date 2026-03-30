use clap::Args;

use crate::{
  cli::{self, AppContext},
  store,
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
    let data_dir = &ctx.data_dir;
    let id = store::resolve_task_id(data_dir, &self.id, false)?;
    let task = store::read_task(data_dir, &id)?;

    let value = store::meta::resolve_dot_path(&toml::Value::Table(task.metadata), &self.path)
      .ok_or_else(|| cli::Error::generic(format!("Metadata key not found: '{}'", self.path)))?;

    store::meta::print_toml_value(&value);
    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  mod call {
    use super::*;
    use crate::test_helpers::{make_test_context, make_test_task};

    #[test]
    fn it_errors_on_missing_path() {
      let dir = tempfile::tempdir().unwrap();
      let ctx = make_test_context(dir.path());
      let task = make_test_task("zyxwvutsrqponmlkzyxwvutsrqponmlk");
      store::write_task(&ctx.data_dir, &task).unwrap();

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
      store::write_task(&ctx.data_dir, &task).unwrap();

      let cmd = Command {
        id: "zyxw".to_string(),
        path: "priority".to_string(),
      };
      cmd.call(&ctx).unwrap();
    }
  }
}
