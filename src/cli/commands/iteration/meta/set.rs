use chrono::Utc;
use clap::Args;

use crate::{
  cli::{self, AppContext},
  store,
  ui::composites::success_message::SuccessMessage,
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
    let layout = &ctx.layout;
    let theme = &ctx.theme;
    let id = store::resolve_iteration_id(layout, &self.id, false)?;
    let mut iteration = store::read_iteration(layout, &id)?;

    store::meta::set_dot_path(&mut iteration.metadata, &self.path, &self.value);

    iteration.updated_at = Utc::now();
    store::write_iteration(layout, &iteration)?;

    let msg = format!("Set {}.{} = {}", id, self.path, self.value);
    println!("{}", SuccessMessage::new(&msg, theme));
    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  mod call {
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::test_helpers::{make_test_context, make_test_iteration};

    #[test]
    fn it_sets_metadata_value() {
      let dir = tempfile::tempdir().unwrap();
      let ctx = make_test_context(dir.path());
      let iteration = make_test_iteration("zyxwvutsrqponmlkzyxwvutsrqponmlk");
      store::write_iteration(&ctx.layout, &iteration).unwrap();

      let cmd = Command {
        id: "zyxw".to_string(),
        path: "priority".to_string(),
        value: "high".to_string(),
      };
      cmd.call(&ctx).unwrap();

      let loaded = store::read_iteration(&ctx.layout, &iteration.id).unwrap();
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
      store::write_iteration(&ctx.layout, &iteration).unwrap();

      let cmd = Command {
        id: "zyxw".to_string(),
        path: "config.timeout".to_string(),
        value: "30".to_string(),
      };
      cmd.call(&ctx).unwrap();

      let loaded = store::read_iteration(&ctx.layout, &iteration.id).unwrap();
      let config = loaded.metadata.get("config").unwrap().as_table().unwrap();
      assert_eq!(config.get("timeout"), Some(&toml::Value::Integer(30)));
    }
  }
}
