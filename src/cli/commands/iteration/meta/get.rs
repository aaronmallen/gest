use clap::Args;

use crate::{
  cli::{self, AppContext},
  store,
};

/// Retrieve a single metadata value from an iteration.
#[derive(Debug, Args)]
pub struct Command {
  /// Iteration ID or unique prefix.
  pub id: String,
  /// Dot-delimited key path (e.g. `outer.inner`).
  pub path: String,
}

impl Command {
  /// Resolve the iteration, walk the metadata table by dot-path, and print the value.
  pub fn call(&self, ctx: &AppContext) -> cli::Result<()> {
    let data_dir = &ctx.data_dir;
    let id = store::resolve_iteration_id(data_dir, &self.id, false)?;
    let iteration = store::read_iteration(data_dir, &id)?;

    let value = store::meta::resolve_dot_path(&toml::Value::Table(iteration.metadata), &self.path)
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
    use crate::test_helpers::{make_test_context, make_test_iteration};

    #[test]
    fn it_errors_on_missing_path() {
      let dir = tempfile::tempdir().unwrap();
      let ctx = make_test_context(dir.path());
      let iteration = make_test_iteration("zyxwvutsrqponmlkzyxwvutsrqponmlk");
      store::write_iteration(&ctx.data_dir, &iteration).unwrap();

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
      let mut iteration = make_test_iteration("zyxwvutsrqponmlkzyxwvutsrqponmlk");
      iteration
        .metadata
        .insert("priority".to_string(), toml::Value::String("high".to_string()));
      store::write_iteration(&ctx.data_dir, &iteration).unwrap();

      let cmd = Command {
        id: "zyxw".to_string(),
        path: "priority".to_string(),
      };
      cmd.call(&ctx).unwrap();
    }
  }
}
