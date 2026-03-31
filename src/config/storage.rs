//! Data directory resolution for artifact and task storage.
//!
//! The data directory is resolved with the following precedence:
//! `$GEST_DATA_DIR` > config `storage.data_dir` > local `.gest`/`gest` dir > global data home.

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
  task_dir: Option<PathBuf>,
}

impl Settings {
  /// Resolve the artifact directory for a given base data directory.
  pub(crate) fn resolve_artifact_dir(&self, data_dir: &Path) -> PathBuf {
    resolve_entity_dir(
      &super::env::GEST_ARTIFACT_DIR,
      self.artifact_dir.as_deref(),
      data_dir,
      "artifacts",
    )
  }

  /// Resolve the iteration directory for a given base data directory.
  pub(crate) fn resolve_iteration_dir(&self, data_dir: &Path) -> PathBuf {
    resolve_entity_dir(
      &super::env::GEST_ITERATION_DIR,
      self.iteration_dir.as_deref(),
      data_dir,
      "iterations",
    )
  }

  /// Resolve the task directory for a given base data directory.
  pub(crate) fn resolve_task_dir(&self, data_dir: &Path) -> PathBuf {
    resolve_entity_dir(&super::env::GEST_TASK_DIR, self.task_dir.as_deref(), data_dir, "tasks")
  }

  /// Resolves the absolute path to the data directory for the given working directory.
  ///
  /// Checks `$GEST_DATA_DIR`, then the configured `data_dir`, then walks up from
  /// `cwd` looking for a `.gest` or `gest` directory, and finally falls back to
  /// the platform's global data home with a path-derived hash.
  pub fn resolve_data_dir(&self, cwd: PathBuf) -> Result<PathBuf, Error> {
    if let Ok(path) = super::env::GEST_DATA_DIR.value() {
      if path.is_absolute() && path.is_dir() {
        log::debug!("$GEST_DATA_DIR is set");
        log::trace!("data directory resolved to {}", path.display());
        return Ok(path);
      } else if path.is_dir() {
        log::debug!("$GEST_DATA_DIR is set, but is not an absolute path");
        log::warn!("$GEST_DATA_DIR must be an absolute path");
        log::trace!("ignoring $GEST_DATA_DIR: {}", path.display());
      } else if path.is_absolute() {
        return Err(Error::NotADirectory(path));
      }
    }

    if let Some(path) = &self.data_dir {
      if path.is_absolute() && path.is_dir() {
        log::debug!("config specifies storage.data_dir");
        log::trace!("data directory resolved to {}", path.display());
        return Ok(path.clone());
      } else if path.is_dir() {
        log::debug!("config specifies data_dir, but is not an absolute path");
        log::warn!("storage.data_dir must be an absolute path");
        log::trace!("ignoring storage.data_dir: {}", path.display());
      } else if path.is_absolute() {
        return Err(Error::NotADirectory(path.clone()));
      }
    }

    if let Some(path) = walk_up_dir(&cwd, &[".gest", "gest"]) {
      log::debug!("found gest directory");
      log::trace!("data directory resolved to {}", path.display());
      return Ok(path);
    }

    let global_data_dir = dir_spec::data_home()
      .map(|p| p.join("gest"))
      .ok_or(Error::DirectoryNotFound("data"))?;

    let global_project_data_dir = global_data_dir.join(path_hash(&cwd));
    log::debug!("no gest directory found");
    log::trace!("data directory resolved to {}", global_project_data_dir.display());

    Ok(global_project_data_dir)
  }
}

/// Resolve a single entity directory from env var, config, or fallback.
fn resolve_entity_dir(
  env_var: &typed_env::Envar<PathBuf>,
  config_dir: Option<&Path>,
  data_dir: &Path,
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

  data_dir.join(default_subdir)
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
      if !current.pop() {
        return None;
      }
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
      use tempfile::TempDir;

      use super::*;

      #[test]
      fn it_falls_back_to_config_when_env_is_set_to_a_non_absolute_path() {
        let tmp_from_env = TempDir::new().unwrap();
        let tmp_from_config = TempDir::new().unwrap();
        let path = tmp_from_env.path().to_str().unwrap().strip_prefix("/").unwrap();

        with_var("GEST_DATA_DIR", Some(path), || {
          let settings = Settings {
            data_dir: Some(tmp_from_config.path().to_path_buf()),
            ..Default::default()
          };

          assert_eq!(
            settings.resolve_data_dir(std::env::current_dir().unwrap()).unwrap(),
            tmp_from_config.path().to_path_buf()
          );
        })
      }

      #[test]
      fn it_falls_back_to_global_data_dir_when_no_local_gest_dir_exists() {
        let tmp = TempDir::new().unwrap();

        with_var_unset("GEST_DATA_DIR", || {
          let settings = Settings::default();
          let result = settings.resolve_data_dir(tmp.path().to_path_buf()).unwrap();
          let expected = dir_spec::data_home()
            .map(|p| p.join("gest").join(super::path_hash(tmp.path())))
            .unwrap();

          assert_eq!(result, expected);
        })
      }

      #[test]
      fn it_falls_back_to_local_data_dir_when_config_is_set_to_a_non_absolute_path() {
        let tmp = TempDir::new().unwrap();
        let rel_path = tmp.path().to_str().unwrap().strip_prefix("/").unwrap();
        let gest_path = tmp.path().join(".gest");
        std::fs::create_dir_all(&gest_path).unwrap();

        with_var_unset("GEST_DATA_DIR", || {
          let settings = Settings {
            data_dir: Some(PathBuf::from(rel_path)),
            ..Default::default()
          };

          assert_eq!(
            settings.resolve_data_dir(tmp.path().to_path_buf()).unwrap(),
            gest_path.to_path_buf()
          );
        })
      }

      #[test]
      fn it_returns_an_error_when_config_is_set_to_a_file_and_env_is_unset() {
        let tmp = TempDir::new().unwrap();
        let filepath = tmp.path().join("gest");
        std::fs::write(&filepath, "").unwrap();

        with_var_unset("GEST_DATA_DIR", || {
          let settings = Settings {
            data_dir: Some(filepath),
            ..Default::default()
          };

          assert!(settings.resolve_data_dir(std::env::current_dir().unwrap()).is_err());
        })
      }

      #[test]
      fn it_returns_an_error_when_env_is_set_to_a_file() {
        let tmp = TempDir::new().unwrap();
        let filepath = tmp.path().join("gest");
        std::fs::write(&filepath, "").unwrap();

        with_var("GEST_DATA_DIR", Some(filepath.to_str().unwrap()), || {
          let settings = Settings::default();

          assert!(settings.resolve_data_dir(std::env::current_dir().unwrap()).is_err());
        })
      }

      #[test]
      fn it_returns_path_from_env_when_set() {
        let tmp = TempDir::new().unwrap();
        with_var("GEST_DATA_DIR", Some(tmp.path().to_str().unwrap()), || {
          let settings = Settings::default();

          assert_eq!(
            settings.resolve_data_dir(std::env::current_dir().unwrap()).unwrap(),
            tmp.path().to_path_buf()
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
    fn it_falls_back_to_data_dir_subdirs_when_no_overrides_are_set() {
      let unset: [(&str, Option<&str>); 3] = [
        ("GEST_ARTIFACT_DIR", None),
        ("GEST_TASK_DIR", None),
        ("GEST_ITERATION_DIR", None),
      ];
      with_vars(unset, || {
        let data_dir = PathBuf::from("/tmp/gest-data");
        let settings = Settings::default();

        assert_eq!(
          settings.resolve_artifact_dir(&data_dir),
          Path::new("/tmp/gest-data/artifacts")
        );
        assert_eq!(settings.resolve_task_dir(&data_dir), Path::new("/tmp/gest-data/tasks"));
        assert_eq!(
          settings.resolve_iteration_dir(&data_dir),
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
          let data_dir = PathBuf::from("/tmp/gest-data");
          let settings = Settings {
            task_dir: Some(PathBuf::from("/config/tasks")),
            ..Default::default()
          };

          assert_eq!(settings.resolve_artifact_dir(&data_dir), env_artifact);
          assert_eq!(settings.resolve_task_dir(&data_dir), Path::new("/config/tasks"));
          assert_eq!(
            settings.resolve_iteration_dir(&data_dir),
            Path::new("/tmp/gest-data/iterations")
          );
        },
      );
    }

    #[test]
    fn it_uses_config_fields_over_data_dir() {
      let unset: [(&str, Option<&str>); 3] = [
        ("GEST_ARTIFACT_DIR", None),
        ("GEST_TASK_DIR", None),
        ("GEST_ITERATION_DIR", None),
      ];
      with_vars(unset, || {
        let data_dir = PathBuf::from("/tmp/gest-data");
        let settings = Settings {
          artifact_dir: Some(PathBuf::from("/custom/docs")),
          task_dir: Some(PathBuf::from("/custom/tasks")),
          iteration_dir: Some(PathBuf::from("/custom/iterations")),
          ..Default::default()
        };

        assert_eq!(settings.resolve_artifact_dir(&data_dir), Path::new("/custom/docs"));
        assert_eq!(settings.resolve_task_dir(&data_dir), Path::new("/custom/tasks"));
        assert_eq!(
          settings.resolve_iteration_dir(&data_dir),
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
          let data_dir = PathBuf::from("/tmp/gest-data");
          let settings = Settings {
            artifact_dir: Some(PathBuf::from("/config/docs")),
            task_dir: Some(PathBuf::from("/config/tasks")),
            iteration_dir: Some(PathBuf::from("/config/iterations")),
            ..Default::default()
          };

          assert_eq!(settings.resolve_artifact_dir(&data_dir), env_artifact);
          assert_eq!(settings.resolve_task_dir(&data_dir), env_task);
          assert_eq!(settings.resolve_iteration_dir(&data_dir), env_iter);
        },
      );
    }
  }
}
