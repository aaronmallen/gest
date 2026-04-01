//! Data directory resolution for artifact and task storage.
//!
//! Storage paths are resolved with the following precedence layers:
//! - **data_dir** (global root): `$GEST_DATA_DIR` > config `storage.data_dir` > `$XDG_DATA_HOME/gest`
//! - **project_dir**: `$GEST_PROJECT_DIR` > config `storage.project_dir` > local `.gest` dir > `<data_dir>/<hash>`
//! - **entity dirs**: entity env var > entity config field > `<project_dir>/<entity>`

use std::{
  fmt::Write,
  path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use super::Error;

/// Configuration for the `[storage]` section.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(default)]
pub struct Settings {
  artifact_dir: Option<PathBuf>,
  data_dir: Option<PathBuf>,
  iteration_dir: Option<PathBuf>,
  project_dir: Option<PathBuf>,
  state_dir: Option<PathBuf>,
  task_dir: Option<PathBuf>,
  #[serde(skip)]
  resolved_artifact_dir: PathBuf,
  #[serde(skip)]
  resolved_data_dir: PathBuf,
  #[serde(skip)]
  resolved_iteration_dir: PathBuf,
  #[serde(skip)]
  resolved_project_dir: PathBuf,
  #[serde(skip)]
  resolved_state_dir: PathBuf,
  #[serde(skip)]
  resolved_task_dir: PathBuf,
}

impl Settings {
  /// The resolved artifact storage directory.
  pub fn artifact_dir(&self) -> &Path {
    &self.resolved_artifact_dir
  }

  /// The resolved global data root directory.
  #[allow(dead_code)]
  pub fn data_dir(&self) -> &Path {
    &self.resolved_data_dir
  }

  /// The resolved iteration storage directory.
  pub fn iteration_dir(&self) -> &Path {
    &self.resolved_iteration_dir
  }

  /// The resolved project-specific data directory.
  pub fn project_dir(&self) -> &Path {
    &self.resolved_project_dir
  }

  /// Resolves the global data root directory.
  ///
  /// Checks `$GEST_DATA_DIR`, then the configured `data_dir`, then falls back to
  /// the platform's global data home.
  pub fn resolve_data_dir(&self) -> Result<PathBuf, Error> {
    if let Ok(path) = super::env::GEST_DATA_DIR.value() {
      if path.is_absolute() {
        log::debug!("$GEST_DATA_DIR is set");
        log::trace!("data directory resolved to {}", path.display());
        return Ok(path);
      }
      log::debug!("$GEST_DATA_DIR is set, but is not an absolute path");
      log::warn!("$GEST_DATA_DIR must be an absolute path");
      log::trace!("ignoring $GEST_DATA_DIR: {}", path.display());
    }

    if let Some(path) = &self.data_dir {
      if path.is_absolute() {
        log::debug!("config specifies storage.data_dir");
        log::trace!("data directory resolved to {}", path.display());
        return Ok(path.clone());
      }
      log::debug!("config specifies data_dir, but is not an absolute path");
      log::warn!("storage.data_dir must be an absolute path");
      log::trace!("ignoring storage.data_dir: {}", path.display());
    }

    let global_data_dir = dir_spec::data_home()
      .map(|p| p.join("gest"))
      .ok_or(Error::DirectoryNotFound("data"))?;

    log::debug!("no data directory override found");
    log::trace!("data directory resolved to {}", global_data_dir.display());

    Ok(global_data_dir)
  }

  /// Resolves the project-specific data directory for the given working directory.
  ///
  /// Checks `$GEST_PROJECT_DIR`, then the configured `project_dir`, then walks up from
  /// `cwd` looking for a `.gest` directory, and finally falls back to
  /// `<data_dir>/<path_hash(cwd)>`.
  pub fn resolve_project_dir(&self, cwd: &Path) -> Result<PathBuf, Error> {
    if let Ok(path) = super::env::GEST_PROJECT_DIR.value() {
      if path.is_absolute() && path.is_dir() {
        log::debug!("$GEST_PROJECT_DIR is set");
        log::trace!("project directory resolved to {}", path.display());
        return Ok(path);
      } else if path.is_absolute() && !path.exists() {
        log::debug!("$GEST_PROJECT_DIR is set to a non-existent path");
        log::trace!("project directory resolved to {}", path.display());
        return Ok(path);
      } else if path.is_dir() {
        log::debug!("$GEST_PROJECT_DIR is set, but is not an absolute path");
        log::warn!("$GEST_PROJECT_DIR must be an absolute path");
        log::trace!("ignoring $GEST_PROJECT_DIR: {}", path.display());
      } else if path.is_absolute() {
        return Err(Error::NotADirectory(path));
      }
    }

    if let Some(path) = &self.project_dir {
      if path.is_absolute() && path.is_dir() {
        log::debug!("config specifies storage.project_dir");
        log::trace!("project directory resolved to {}", path.display());
        return Ok(path.clone());
      } else if path.is_absolute() && !path.exists() {
        log::debug!("config specifies storage.project_dir as a non-existent path");
        log::trace!("project directory resolved to {}", path.display());
        return Ok(path.clone());
      } else if path.is_dir() {
        log::debug!("config specifies project_dir, but is not an absolute path");
        log::warn!("storage.project_dir must be an absolute path");
        log::trace!("ignoring storage.project_dir: {}", path.display());
      } else if path.is_absolute() {
        return Err(Error::NotADirectory(path.clone()));
      }
    }

    if let Some(path) = walk_up_dir(cwd, &[".gest"]) {
      log::debug!("found gest directory");
      log::trace!("project directory resolved to {}", path.display());
      return Ok(path);
    }

    let global_project_dir = self.resolved_data_dir.join(path_hash(cwd));
    log::debug!("no gest directory found");
    log::trace!("project directory resolved to {}", global_project_dir.display());

    Ok(global_project_dir)
  }

  /// Resolves the absolute path to the state directory for the given working directory.
  ///
  /// Checks `$GEST_STATE_DIR`, then falls back to the platform's global state home
  /// with a path-derived hash. The state directory is always global (never in-repo).
  pub fn resolve_state_dir(&self, cwd: &Path) -> Result<PathBuf, Error> {
    if let Ok(path) = super::env::GEST_STATE_DIR.value() {
      if path.is_absolute() {
        log::debug!("$GEST_STATE_DIR is set");
        log::trace!("state directory resolved to {}", path.display());
        return Ok(path);
      }
      log::debug!("$GEST_STATE_DIR is set, but is not an absolute path");
      log::warn!("$GEST_STATE_DIR must be an absolute path");
      log::trace!("ignoring $GEST_STATE_DIR: {}", path.display());
    }

    if let Some(path) = &self.state_dir {
      if path.is_absolute() && path.is_dir() {
        log::debug!("config specifies storage.state_dir");
        log::trace!("state directory resolved to {}", path.display());
        return Ok(path.clone());
      } else if path.is_dir() {
        log::debug!("config specifies state_dir, but is not an absolute path");
        log::warn!("storage.state_dir must be an absolute path");
        log::trace!("ignoring storage.state_dir: {}", path.display());
      } else if path.is_absolute() {
        return Err(Error::NotADirectory(path.clone()));
      }
    }

    let global_state_dir = dir_spec::state_home()
      .map(|p| p.join("gest"))
      .ok_or(Error::DirectoryNotFound("state"))?;

    let global_project_state_dir = global_state_dir.join(path_hash(cwd));
    log::debug!("no state directory override found");
    log::trace!("state directory resolved to {}", global_project_state_dir.display());

    Ok(global_project_state_dir)
  }

  /// The resolved state directory (event store, undo log).
  pub fn state_dir(&self) -> &Path {
    &self.resolved_state_dir
  }

  /// The resolved task storage directory.
  pub fn task_dir(&self) -> &Path {
    &self.resolved_task_dir
  }

  /// Resolve all storage paths from the working directory.
  ///
  /// Called during config loading so the settings are ready to use immediately.
  pub(crate) fn resolve(&mut self, cwd: PathBuf) -> Result<(), Error> {
    self.resolved_state_dir = self.resolve_state_dir(&cwd)?;
    self.resolved_data_dir = self.resolve_data_dir()?;
    self.resolved_project_dir = self.resolve_project_dir(&cwd)?;
    let project_dir = self.resolved_project_dir.clone();
    self.resolved_artifact_dir = self.resolve_artifact_dir_from(&project_dir);
    self.resolved_iteration_dir = self.resolve_iteration_dir_from(&project_dir);
    self.resolved_task_dir = self.resolve_task_dir_from(&project_dir);
    Ok(())
  }

  /// Resolve the artifact directory for a given base project directory.
  pub(crate) fn resolve_artifact_dir_from(&self, project_dir: &Path) -> PathBuf {
    resolve_entity_dir(
      &super::env::GEST_ARTIFACT_DIR,
      self.artifact_dir.as_deref(),
      project_dir,
      "artifacts",
    )
  }

  /// Resolve the iteration directory for a given base project directory.
  pub(crate) fn resolve_iteration_dir_from(&self, project_dir: &Path) -> PathBuf {
    resolve_entity_dir(
      &super::env::GEST_ITERATION_DIR,
      self.iteration_dir.as_deref(),
      project_dir,
      "iterations",
    )
  }

  /// Resolve the task directory for a given base project directory.
  pub(crate) fn resolve_task_dir_from(&self, project_dir: &Path) -> PathBuf {
    resolve_entity_dir(
      &super::env::GEST_TASK_DIR,
      self.task_dir.as_deref(),
      project_dir,
      "tasks",
    )
  }

  /// Set storage paths for an already-known project directory, skipping discovery.
  #[cfg(test)]
  pub fn resolve_at(&mut self, project_dir: PathBuf) {
    self.resolved_project_dir = project_dir;
    let base = self.resolved_project_dir.clone();
    self.resolved_artifact_dir = self.resolve_artifact_dir_from(&base);
    self.resolved_iteration_dir = self.resolve_iteration_dir_from(&base);
    self.resolved_task_dir = self.resolve_task_dir_from(&base);
  }

  /// Set the state directory path directly, skipping env-var/fallback discovery.
  #[cfg(test)]
  pub fn resolve_state_at(&mut self, state_dir: PathBuf) {
    self.resolved_state_dir = state_dir;
  }
}

/// Resolve a single entity directory from env var, config, or fallback.
fn resolve_entity_dir(
  env_var: &typed_env::Envar<PathBuf>,
  config_dir: Option<&Path>,
  project_dir: &Path,
  default_subdir: &str,
) -> PathBuf {
  if let Ok(path) = env_var.value() {
    log::debug!("${} is set", env_var.name());
    log::trace!("{default_subdir} directory resolved to {}", path.display());
    return path;
  }

  if let Some(path) = config_dir {
    log::debug!(
      "config specifies storage.{}_dir",
      default_subdir.strip_suffix('s').unwrap_or(default_subdir)
    );
    log::trace!("{default_subdir} directory resolved to {}", path.display());
    return path.to_path_buf();
  }

  project_dir.join(default_subdir)
}

/// Produces a short hex hash of the canonicalized path for use as a directory name.
fn path_hash(path: &Path) -> String {
  let canonical = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
  let mut hasher = Sha256::new();
  hasher.update(canonical.as_os_str().as_encoded_bytes());
  let result = hasher.finalize();
  let mut hash = String::with_capacity(16);
  for b in &result[..8] {
    write!(hash, "{b:02x}").expect("writing to String is infallible");
  }
  hash
}

/// Walks up from `start` looking for a subdirectory matching any of `names`.
fn walk_up_dir(start: &Path, names: &[&str]) -> Option<PathBuf> {
  let mut current = start.to_path_buf();
  loop {
    for name in names {
      let candidate = current.join(name);
      if candidate.is_dir() {
        return Some(candidate);
      }
    }
    if !current.pop() {
      return None;
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  mod settings {
    use super::*;

    mod data_dir {
      use pretty_assertions::assert_eq;
      use temp_env::{with_var, with_var_unset};

      use super::*;

      #[test]
      fn it_falls_back_to_config_when_env_is_set_to_a_non_absolute_path() {
        let tmp_from_config = tempfile::TempDir::new().unwrap();

        with_var("GEST_DATA_DIR", Some("relative/path"), || {
          let settings = Settings {
            data_dir: Some(tmp_from_config.path().to_path_buf()),
            ..Default::default()
          };

          assert_eq!(
            settings.resolve_data_dir().unwrap(),
            tmp_from_config.path().to_path_buf()
          );
        })
      }

      #[test]
      fn it_falls_back_to_global_data_dir_when_no_overrides_exist() {
        with_var_unset("GEST_DATA_DIR", || {
          let settings = Settings::default();
          let result = settings.resolve_data_dir().unwrap();
          let expected = dir_spec::data_home().map(|p| p.join("gest")).unwrap();

          assert_eq!(result, expected);
        })
      }

      #[test]
      fn it_returns_path_from_env_when_set() {
        let tmp = tempfile::TempDir::new().unwrap();
        with_var("GEST_DATA_DIR", Some(tmp.path().to_str().unwrap()), || {
          let settings = Settings::default();

          assert_eq!(settings.resolve_data_dir().unwrap(), tmp.path().to_path_buf());
        })
      }

      #[test]
      fn it_accepts_non_existent_absolute_path_from_env() {
        with_var("GEST_DATA_DIR", Some("/tmp/gest-nonexistent-test-dir"), || {
          let settings = Settings::default();

          assert_eq!(
            settings.resolve_data_dir().unwrap(),
            PathBuf::from("/tmp/gest-nonexistent-test-dir")
          );
        })
      }
    }

    mod project_dir {
      use pretty_assertions::assert_eq;
      use temp_env::{with_var, with_vars, with_vars_unset};
      use tempfile::TempDir;

      use super::*;

      #[test]
      fn it_returns_path_from_env_when_set() {
        let tmp = TempDir::new().unwrap();
        with_var("GEST_PROJECT_DIR", Some(tmp.path().to_str().unwrap()), || {
          let settings = Settings::default();

          assert_eq!(
            settings.resolve_project_dir(&std::env::current_dir().unwrap()).unwrap(),
            tmp.path().to_path_buf()
          );
        })
      }

      #[test]
      fn it_falls_back_to_config_when_env_is_set_to_a_non_absolute_path() {
        let tmp_from_env = TempDir::new().unwrap();
        let tmp_from_config = TempDir::new().unwrap();
        let path = tmp_from_env.path().to_str().unwrap().strip_prefix("/").unwrap();

        with_var("GEST_PROJECT_DIR", Some(path), || {
          let settings = Settings {
            project_dir: Some(tmp_from_config.path().to_path_buf()),
            ..Default::default()
          };

          assert_eq!(
            settings.resolve_project_dir(&std::env::current_dir().unwrap()).unwrap(),
            tmp_from_config.path().to_path_buf()
          );
        })
      }

      #[test]
      fn it_falls_back_to_walk_up_when_no_overrides_are_set() {
        let tmp = TempDir::new().unwrap();
        let gest_path = tmp.path().join(".gest");
        std::fs::create_dir_all(&gest_path).unwrap();

        with_vars_unset(["GEST_PROJECT_DIR", "GEST_DATA_DIR"], || {
          let settings = Settings::default();
          // resolve_data_dir first so resolved_data_dir is populated
          let mut s = settings;
          s.resolved_data_dir = dir_spec::data_home().map(|p| p.join("gest")).unwrap_or_default();

          assert_eq!(
            s.resolve_project_dir(&tmp.path().to_path_buf()).unwrap(),
            gest_path.to_path_buf()
          );
        })
      }

      #[test]
      fn it_falls_back_to_data_dir_with_hash_when_no_local_gest_dir_exists() {
        let tmp = TempDir::new().unwrap();

        with_vars(
          [
            ("GEST_PROJECT_DIR", None),
            ("GEST_DATA_DIR", Some("/tmp/gest-test-root")),
          ],
          || {
            let mut settings = Settings::default();
            settings.resolved_data_dir = PathBuf::from("/tmp/gest-test-root");

            let result = settings.resolve_project_dir(&tmp.path().to_path_buf()).unwrap();
            let expected = PathBuf::from("/tmp/gest-test-root").join(path_hash(tmp.path()));

            assert_eq!(result, expected);
          },
        )
      }

      #[test]
      fn it_returns_an_error_when_env_is_set_to_a_file() {
        let tmp = TempDir::new().unwrap();
        let filepath = tmp.path().join("gest");
        std::fs::write(&filepath, "").unwrap();

        with_var("GEST_PROJECT_DIR", Some(filepath.to_str().unwrap()), || {
          let settings = Settings::default();

          assert!(settings.resolve_project_dir(&std::env::current_dir().unwrap()).is_err());
        })
      }

      #[test]
      fn it_returns_an_error_when_config_is_set_to_a_file() {
        let tmp = TempDir::new().unwrap();
        let filepath = tmp.path().join("gest");
        std::fs::write(&filepath, "").unwrap();

        with_vars_unset(["GEST_PROJECT_DIR"], || {
          let settings = Settings {
            project_dir: Some(filepath.clone()),
            ..Default::default()
          };

          assert!(settings.resolve_project_dir(&std::env::current_dir().unwrap()).is_err());
        })
      }

      #[test]
      fn it_ignores_undotted_gest_directory_during_walk_up() {
        let tmp = TempDir::new().unwrap();
        let undotted = tmp.path().join("gest");
        std::fs::create_dir_all(&undotted).unwrap();

        with_vars(
          [
            ("GEST_PROJECT_DIR", None),
            ("GEST_DATA_DIR", Some("/tmp/gest-test-root")),
          ],
          || {
            let mut settings = Settings::default();
            settings.resolved_data_dir = PathBuf::from("/tmp/gest-test-root");

            let result = settings.resolve_project_dir(&tmp.path().to_path_buf()).unwrap();
            let expected = PathBuf::from("/tmp/gest-test-root").join(path_hash(tmp.path()));

            assert_eq!(result, expected);
          },
        )
      }

      #[test]
      fn it_falls_back_to_local_project_dir_when_config_is_set_to_a_non_absolute_path() {
        let tmp = TempDir::new().unwrap();
        let rel_path = tmp.path().to_str().unwrap().strip_prefix("/").unwrap();
        let gest_path = tmp.path().join(".gest");
        std::fs::create_dir_all(&gest_path).unwrap();

        with_vars_unset(["GEST_PROJECT_DIR"], || {
          let mut settings = Settings {
            project_dir: Some(PathBuf::from(rel_path)),
            ..Default::default()
          };
          settings.resolved_data_dir = dir_spec::data_home().map(|p| p.join("gest")).unwrap_or_default();

          assert_eq!(
            settings.resolve_project_dir(&tmp.path().to_path_buf()).unwrap(),
            gest_path.to_path_buf()
          );
        })
      }
    }
  }

  mod resolve_entity_dirs {
    use pretty_assertions::assert_eq;
    use temp_env::with_vars;
    use tempfile::TempDir;

    use super::*;

    #[test]
    fn it_falls_back_to_project_dir_subdirs_when_no_overrides_are_set() {
      let unset: [(&str, Option<&str>); 3] = [
        ("GEST_ARTIFACT_DIR", None),
        ("GEST_TASK_DIR", None),
        ("GEST_ITERATION_DIR", None),
      ];
      with_vars(unset, || {
        let project_dir = PathBuf::from("/tmp/gest-data");
        let settings = Settings::default();

        assert_eq!(
          settings.resolve_artifact_dir_from(&project_dir),
          Path::new("/tmp/gest-data/artifacts")
        );
        assert_eq!(
          settings.resolve_task_dir_from(&project_dir),
          Path::new("/tmp/gest-data/tasks")
        );
        assert_eq!(
          settings.resolve_iteration_dir_from(&project_dir),
          Path::new("/tmp/gest-data/iterations")
        );
      });
    }

    #[test]
    fn it_mixes_overrides_per_entity() {
      let tmp = TempDir::new().unwrap();
      let env_artifact = tmp.path().join("env-artifacts");

      with_vars(
        [
          ("GEST_ARTIFACT_DIR", Some(env_artifact.to_str().unwrap())),
          ("GEST_TASK_DIR", None),
          ("GEST_ITERATION_DIR", None),
        ],
        || {
          let project_dir = PathBuf::from("/tmp/gest-data");
          let settings = Settings {
            task_dir: Some(PathBuf::from("/config/tasks")),
            ..Default::default()
          };

          assert_eq!(settings.resolve_artifact_dir_from(&project_dir), env_artifact);
          assert_eq!(settings.resolve_task_dir_from(&project_dir), Path::new("/config/tasks"));
          assert_eq!(
            settings.resolve_iteration_dir_from(&project_dir),
            Path::new("/tmp/gest-data/iterations")
          );
        },
      );
    }

    #[test]
    fn it_uses_config_fields_over_project_dir() {
      let unset: [(&str, Option<&str>); 3] = [
        ("GEST_ARTIFACT_DIR", None),
        ("GEST_TASK_DIR", None),
        ("GEST_ITERATION_DIR", None),
      ];
      with_vars(unset, || {
        let project_dir = PathBuf::from("/tmp/gest-data");
        let settings = Settings {
          artifact_dir: Some(PathBuf::from("/custom/docs")),
          task_dir: Some(PathBuf::from("/custom/tasks")),
          iteration_dir: Some(PathBuf::from("/custom/iterations")),
          ..Default::default()
        };

        assert_eq!(
          settings.resolve_artifact_dir_from(&project_dir),
          Path::new("/custom/docs")
        );
        assert_eq!(settings.resolve_task_dir_from(&project_dir), Path::new("/custom/tasks"));
        assert_eq!(
          settings.resolve_iteration_dir_from(&project_dir),
          Path::new("/custom/iterations")
        );
      });
    }

    #[test]
    fn it_uses_env_vars_over_config_fields() {
      let tmp = TempDir::new().unwrap();
      let env_artifact = tmp.path().join("env-artifacts");
      let env_task = tmp.path().join("env-tasks");
      let env_iter = tmp.path().join("env-iterations");

      with_vars(
        [
          ("GEST_ARTIFACT_DIR", Some(env_artifact.to_str().unwrap())),
          ("GEST_TASK_DIR", Some(env_task.to_str().unwrap())),
          ("GEST_ITERATION_DIR", Some(env_iter.to_str().unwrap())),
        ],
        || {
          let project_dir = PathBuf::from("/tmp/gest-data");
          let settings = Settings {
            artifact_dir: Some(PathBuf::from("/config/docs")),
            task_dir: Some(PathBuf::from("/config/tasks")),
            iteration_dir: Some(PathBuf::from("/config/iterations")),
            ..Default::default()
          };

          assert_eq!(settings.resolve_artifact_dir_from(&project_dir), env_artifact);
          assert_eq!(settings.resolve_task_dir_from(&project_dir), env_task);
          assert_eq!(settings.resolve_iteration_dir_from(&project_dir), env_iter);
        },
      );
    }
  }
}
