use clap::Args;

use crate::{
  cli::{self, AppContext},
  ui::views::system::ConfigView,
};

/// Display the merged configuration and discovered config file sources.
#[derive(Debug, Args)]
pub struct Command;

impl Command {
  /// Render active settings, data directory, log level, and config file locations.
  pub fn call(&self, ctx: &AppContext) -> cli::Result<()> {
    let data_dir = ctx.settings.storage().data_dir().display().to_string();
    let log_level = ctx.settings.log().level().unwrap_or("warn");

    let mut view =
      ConfigView::new(&data_dir, log_level, &ctx.theme).has_color_overrides(!ctx.settings.colors().is_empty());

    if let Some(config_home) = dir_spec::config_home() {
      let global = config_home.join("gest/config.toml");
      if global.exists() {
        let global_str = global.display().to_string();
        view = view.global_config(Box::leak(global_str.into_boxed_str()));
      }
    }

    if let Ok(cwd) = std::env::current_dir() {
      for name in &[".gest/config.toml", ".gest.toml"] {
        let path = cwd.join(name);
        if path.exists() {
          let path_str = path.display().to_string();
          view = view.project_config(Box::leak(path_str.into_boxed_str()));
          break;
        }
      }
    }

    println!("{view}");
    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  mod call {
    use super::*;
    use crate::test_helpers::make_test_context;

    #[test]
    fn it_succeeds_with_default_config() {
      let dir = tempfile::tempdir().unwrap();
      let ctx = make_test_context(dir.path());
      let cmd = Command;
      cmd.call(&ctx).unwrap();
    }
  }
}
