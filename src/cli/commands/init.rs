use std::path::Path;

use clap::Args;

use crate::{
  cli::{self, AppContext},
  config::storage::DataLayout,
  ui::{theme::Theme, views::system::InitView},
};

/// Initialize gest for the current project.
#[derive(Debug, Args)]
pub struct Command {
  /// Initialize a `.gest` directory in the current project instead of the global data directory.
  #[arg(long)]
  local: bool,
}

impl Command {
  /// Initialize the store directory tree, either locally or globally.
  pub fn call(&self, ctx: &AppContext) -> cli::Result<()> {
    let storage = ctx.settings.storage();
    if self.local {
      let cwd = std::env::current_dir()?;
      let base = cwd.join(".gest");
      let layout = DataLayout::new(storage, &base);
      init_at(&base, &layout, Some(".gest/config.toml"), &ctx.theme)
    } else {
      let layout = DataLayout::new(storage, &ctx.data_dir);
      init_at(&ctx.data_dir, &layout, None, &ctx.theme)
    }
  }
}

/// Create any missing subdirectories and display the result.
///
/// Uses the resolved `DataLayout` to determine which directories to create,
/// including per-entity overrides from config or environment variables.
fn init_at(base: &Path, layout: &DataLayout, config_path: Option<&str>, theme: &Theme) -> cli::Result<()> {
  if !base.exists() {
    std::fs::create_dir_all(base)?;
  }
  for (entity_dir, secondary) in [
    (layout.artifact_dir(), "archive"),
    (layout.iteration_dir(), "resolved"),
    (layout.task_dir(), "resolved"),
  ] {
    std::fs::create_dir_all(entity_dir.join(secondary))?;
  }

  let data_dir = base.display().to_string();
  let view = InitView::new(&data_dir, config_path, theme);
  println!("{view}");

  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;

  fn default_layout(base: &std::path::Path) -> DataLayout {
    DataLayout::new(&crate::config::storage::Settings::default(), base)
  }

  mod init_at {
    use super::*;

    #[test]
    fn it_creates_directory_structure() {
      let tmp = tempfile::tempdir().unwrap();
      let base = tmp.path().join("data");

      init_at(&base, &default_layout(&base), None, &Theme::default()).unwrap();

      assert!(base.join("tasks").is_dir());
      assert!(base.join("tasks/resolved").is_dir());
      assert!(base.join("artifacts").is_dir());
      assert!(base.join("artifacts/archive").is_dir());
      assert!(base.join("iterations").is_dir());
      assert!(base.join("iterations/resolved").is_dir());
    }

    #[test]
    fn it_creates_missing_subdirs_when_partially_initialized() {
      let tmp = tempfile::tempdir().unwrap();
      let base = tmp.path().join("data");

      std::fs::create_dir_all(base.join("tasks")).unwrap();

      init_at(&base, &default_layout(&base), None, &Theme::default()).unwrap();

      assert!(base.join("artifacts").is_dir());
      assert!(base.join("tasks/resolved").is_dir());
      assert!(base.join("artifacts/archive").is_dir());
    }

    #[test]
    fn it_is_idempotent() {
      let tmp = tempfile::tempdir().unwrap();
      let base = tmp.path().join("data");

      init_at(&base, &default_layout(&base), None, &Theme::default()).unwrap();
      init_at(&base, &default_layout(&base), None, &Theme::default()).unwrap();

      assert!(base.join("tasks").is_dir());
      assert!(base.join("artifacts").is_dir());
    }

    #[test]
    fn it_accepts_optional_config_path() {
      let tmp = tempfile::tempdir().unwrap();
      let base = tmp.path().join(".gest");

      init_at(
        &base,
        &default_layout(&base),
        Some(".gest/config.toml"),
        &Theme::default(),
      )
      .unwrap();

      assert!(base.join("tasks").is_dir());
      assert!(base.join("tasks/resolved").is_dir());
    }
  }
}
