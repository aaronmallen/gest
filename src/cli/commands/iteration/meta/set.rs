use clap::Args;

use crate::{
  action,
  cli::{self, AppContext},
  model::iteration::Iteration,
  ui::composites::success_message::SuccessMessage,
};

/// Set a metadata value on an iteration.
#[derive(Debug, Args)]
pub struct Command {
  /// Iteration ID or unique prefix.
  pub id: String,
  /// Output as JSON.
  #[arg(short, long, conflicts_with = "quiet")]
  pub json: bool,
  /// Dot-delimited key path (e.g. `outer.inner`).
  pub path: String,
  /// Print only the entity ID.
  #[arg(short, long, conflicts_with = "json")]
  pub quiet: bool,
  /// Value to set (strings, numbers, and booleans are auto-detected).
  pub value: String,
}

impl Command {
  /// Write a metadata key-value pair into the iteration, creating nested tables as needed.
  pub fn call(&self, ctx: &AppContext) -> cli::Result<()> {
    let iteration = action::meta::meta_set::<Iteration>(ctx, &self.id, &self.path, &self.value)?;

    if self.json {
      println!("{}", serde_json::to_string_pretty(&iteration)?);
    } else if self.quiet {
      println!("{}", iteration.id);
    } else {
      let msg = format!("Set {}.{} = {}", iteration.id, self.path, self.value);
      println!("{}", SuccessMessage::new(&msg, &ctx.theme));
    }

    Ok(())
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
      json: false,
      path: "priority".to_string(),
      quiet: false,
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
      json: false,
      path: "config.timeout".to_string(),
      quiet: false,
      value: "30".to_string(),
    };
    cmd.call(&ctx).unwrap();

    let loaded = store::read_iteration(&ctx.settings, &iteration.id).unwrap();
    let config = loaded.metadata.get("config").unwrap().as_table().unwrap();
    assert_eq!(config.get("timeout"), Some(&toml::Value::Integer(30)));
  }
}
