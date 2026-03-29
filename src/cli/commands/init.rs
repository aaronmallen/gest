use std::path::Path;

use clap::Args;

use crate::{
  config::Config,
  ui::{
    components::{AlreadyInitialized, InitCreated},
    theme::Theme,
  },
};

const SUBDIRS: &[&str] = &[
  "artifacts",
  "artifacts/archive",
  "iterations",
  "iterations/resolved",
  "tasks",
  "tasks/archive",
];

/// Initialize a .gest directory in the current project
#[derive(Debug, Args)]
pub struct Command;

impl Command {
  pub fn call(&self, _config: &Config, theme: &Theme) -> crate::Result<()> {
    let cwd = std::env::current_dir()?;
    let base = cwd.join(".gest");
    init_at(&base, theme)
  }
}

fn init_at(base: &Path, theme: &Theme) -> crate::Result<()> {
  let mut created_subdirs = Vec::new();
  for subdir in SUBDIRS {
    let path = base.join(subdir);
    if !path.exists() {
      std::fs::create_dir_all(&path)?;
      created_subdirs.push((*subdir).to_string());
    }
  }

  if created_subdirs.is_empty() {
    AlreadyInitialized.write_to(&mut std::io::stdout(), theme)?;
  } else {
    InitCreated::new(created_subdirs).write_to(&mut std::io::stdout(), theme)?;
  }

  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;

  mod init_at {
    use super::*;

    #[test]
    fn it_creates_gest_directory_structure() {
      let tmp = tempfile::tempdir().unwrap();
      let base = tmp.path().join(".gest");

      init_at(&base, &Theme::default()).unwrap();

      assert!(base.join("tasks").is_dir());
      assert!(base.join("tasks/archive").is_dir());
      assert!(base.join("artifacts").is_dir());
      assert!(base.join("artifacts/archive").is_dir());
      assert!(base.join("iterations").is_dir());
      assert!(base.join("iterations/resolved").is_dir());
    }

    #[test]
    fn it_creates_missing_subdirs_when_partially_initialized() {
      let tmp = tempfile::tempdir().unwrap();
      let base = tmp.path().join(".gest");

      std::fs::create_dir_all(base.join("tasks")).unwrap();

      init_at(&base, &Theme::default()).unwrap();

      assert!(base.join("artifacts").is_dir());
      assert!(base.join("tasks/archive").is_dir());
      assert!(base.join("artifacts/archive").is_dir());
    }

    #[test]
    fn it_is_idempotent() {
      let tmp = tempfile::tempdir().unwrap();
      let base = tmp.path().join(".gest");

      init_at(&base, &Theme::default()).unwrap();
      init_at(&base, &Theme::default()).unwrap();

      assert!(base.join("tasks").is_dir());
      assert!(base.join("artifacts").is_dir());
    }
  }
}
