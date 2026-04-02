use clap::Args;

use crate::{
  action,
  cli::{self, AppContext},
  model::iteration::Iteration,
  ui::views::meta::MetaValueView,
};

/// Retrieve a single metadata value from an iteration.
#[derive(Debug, Args)]
pub struct Command {
  /// Iteration ID or unique prefix.
  pub id: String,
  /// Output as a JSON object.
  #[arg(long, conflicts_with = "raw")]
  pub json: bool,
  /// Dot-delimited key path (e.g. `outer.inner`).
  pub path: String,
  /// Output the bare value with no styling.
  #[arg(long, conflicts_with = "json")]
  pub raw: bool,
}

impl Command {
  /// Resolve the iteration, walk the metadata table by dot-path, and print the value.
  pub fn call(&self, ctx: &AppContext) -> cli::Result<()> {
    let formatted = action::meta::meta_get::<Iteration>(ctx, &self.id, &self.path)?;
    let value = formatted.trim_end_matches('\n');

    if self.json {
      let obj = serde_json::json!({ &self.path: value });
      println!("{}", serde_json::to_string_pretty(&obj).unwrap());
    } else if self.raw {
      println!("{value}");
    } else {
      println!("{}", MetaValueView::new(formatted, ctx.theme.iteration_detail_value));
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
  fn it_errors_on_missing_path() {
    let dir = tempfile::tempdir().unwrap();
    let ctx = make_test_context(dir.path());
    let iteration = make_test_iteration("zyxwvutsrqponmlkzyxwvutsrqponmlk");
    store::write_iteration(&ctx.settings, &iteration).unwrap();

    let cmd = Command {
      id: "zyxw".to_string(),
      json: false,
      path: "nonexistent".to_string(),
      raw: false,
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
    store::write_iteration(&ctx.settings, &iteration).unwrap();

    let cmd = Command {
      id: "zyxw".to_string(),
      json: false,
      path: "priority".to_string(),
      raw: false,
    };
    cmd.call(&ctx).unwrap();
  }
}
