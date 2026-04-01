use std::path::Path;

use clap::Args;

use crate::{
  cli::{self, AppContext},
  config::storage::Settings,
  ui::{theming::theme::Theme, views::system::InitView},
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
      init_at(&base, storage, Some(".gest/config.toml"), &ctx.theme)
    } else {
      init_at(ctx.settings.storage().project_dir(), storage, None, &ctx.theme)
    }
  }
}

/// Create any missing subdirectories and display the result.
///
/// Uses the storage settings to resolve which directories to create,
/// including per-entity overrides from config or environment variables.
fn init_at(base: &Path, storage: &Settings, config_path: Option<&str>, theme: &Theme) -> cli::Result<()> {
  if !base.exists() {
    std::fs::create_dir_all(base)?;
  }
  for (entity_dir, secondary) in [
    (storage.resolve_artifact_dir_from(base), "archive"),
    (storage.resolve_iteration_dir_from(base), "resolved"),
    (storage.resolve_task_dir_from(base), "resolved"),
  ] {
    std::fs::create_dir_all(entity_dir.join(secondary))?;
  }

  let project_dir = base.display().to_string();
  let view = InitView::new(&project_dir, config_path, theme);
  println!("{view}");

  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;

  fn default_storage() -> Settings {
    Settings::default()
  }

  mod init_at {
    use super::*;

    #[test]
    fn it_accepts_optional_config_path() {
      let tmp = tempfile::tempdir().unwrap();
      let base = tmp.path().join(".gest");

      init_at(&base, &default_storage(), Some(".gest/config.toml"), &Theme::default()).unwrap();

      assert!(base.join("tasks").is_dir());
      assert!(base.join("tasks/resolved").is_dir());
    }

    #[test]
    fn it_creates_directory_structure() {
      let tmp = tempfile::tempdir().unwrap();
      let base = tmp.path().join("data");

      init_at(&base, &default_storage(), None, &Theme::default()).unwrap();

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

      init_at(&base, &default_storage(), None, &Theme::default()).unwrap();

      assert!(base.join("artifacts").is_dir());
      assert!(base.join("tasks/resolved").is_dir());
      assert!(base.join("artifacts/archive").is_dir());
    }

    #[test]
    fn it_is_idempotent() {
      let tmp = tempfile::tempdir().unwrap();
      let base = tmp.path().join("data");

      init_at(&base, &default_storage(), None, &Theme::default()).unwrap();
      init_at(&base, &default_storage(), None, &Theme::default()).unwrap();

      assert!(base.join("tasks").is_dir());
      assert!(base.join("artifacts").is_dir());
    }
  }
}
