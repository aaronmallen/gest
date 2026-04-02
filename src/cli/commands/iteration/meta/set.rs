use clap::Args;

use crate::{
  action,
  cli::{self, AppContext},
  model::iteration::Iteration,
};

/// Set a metadata value on an iteration.
#[derive(Debug, Args)]
pub struct Command {
  /// Iteration ID or unique prefix.
  pub id: String,
  /// Dot-delimited key path (e.g. `outer.inner`).
  pub path: String,
  /// Value to set (strings, numbers, and booleans are auto-detected).
  pub value: String,
}

impl Command {
  /// Write a metadata key-value pair into the iteration, creating nested tables as needed.
  pub fn call(&self, ctx: &AppContext) -> cli::Result<()> {
    action::meta::meta_set::<Iteration>(ctx, &self.id, &self.path, &self.value)
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{
    store,
    test_helpers::{make_test_context, make_test_iteration},
  };

  #[test]
  fn it_sets_metadata_value() {
    let dir = tempfile::tempdir().unwrap();
    let ctx = make_test_context(dir.path());
    let iteration = make_test_iteration("zyxwvutsrqponmlkzyxwvutsrqponmlk");
    store::write_iteration(&ctx.settings, &iteration).unwrap();

    let cmd = Command {
      id: "zyxw".to_string(),
      path: "priority".to_string(),
      value: "high".to_string(),
    };
    cmd.call(&ctx).unwrap();

    let loaded = store::read_iteration(&ctx.settings, &iteration.id).unwrap();
    assert_eq!(
      loaded.metadata.get("priority"),
      Some(&toml::Value::String("high".to_string()))
    );
  }

  #[test]
  fn it_sets_nested_metadata_value() {
    let dir = tempfile::tempdir().unwrap();
    let ctx = make_test_context(dir.path());
    let iteration = make_test_iteration("zyxwvutsrqponmlkzyxwvutsrqponmlk");
    store::write_iteration(&ctx.settings, &iteration).unwrap();

    let cmd = Command {
      id: "zyxw".to_string(),
      path: "config.timeout".to_string(),
      value: "30".to_string(),
    };
    cmd.call(&ctx).unwrap();

    let loaded = store::read_iteration(&ctx.settings, &iteration.id).unwrap();
    let config = loaded.metadata.get("config").unwrap().as_table().unwrap();
    assert_eq!(config.get("timeout"), Some(&toml::Value::Integer(30)));
  }
}
