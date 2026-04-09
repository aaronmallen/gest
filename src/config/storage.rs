use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use super::{
  Error,
  env::{GEST_STORAGE__CACHE_DIR, GEST_STORAGE__DATA_DIR, GEST_STORAGE__SYNC},
};

/// Storage-related configuration settings.
///
/// Controls where gest persists data on disk. The `[storage]` TOML table maps to this struct.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(default)]
pub struct Settings {
  /// Explicit cache directory override from the config file.
  cache_dir: Option<PathBuf>,
  /// Explicit data directory override from the config file.
  data_dir: Option<PathBuf>,
  /// Whether automatic file sync with `.gest/` directories is enabled.
  /// Defaults to `true` when absent.
  sync: Option<bool>,
}

impl Settings {
  /// Resolves the cache directory using the following precedence:
  ///
  /// 1. `GEST_STORAGE__CACHE_DIR` environment variable (must be absolute)
  /// 2. `storage.cache_dir` from the config file (must be absolute)
  /// 3. XDG cache home (`$XDG_CACHE_HOME/gest`), falling back to `~/.cache/gest` when `XDG_CACHE_HOME`
  ///    is unset (handled by `dir_spec::cache_home`).
  ///
  /// The cache directory holds regeneratable artifacts (such as the avatar cache) that gest owns
  /// internally. Users control only the root; layout beneath it is an implementation detail.
  pub fn cache_dir(&self) -> Result<PathBuf, Error> {
    if let Ok(path) = GEST_STORAGE__CACHE_DIR.value() {
      if path.is_absolute() {
        log::debug!("$GEST_STORAGE__CACHE_DIR is set to {:?}", path.display());
        log::trace!("using $GEST_STORAGE__CACHE_DIR");
        return Ok(path);
      }
      log::debug!("$GEST_STORAGE__CACHE_DIR: {:?} is not absolute", path.display());
      log::trace!("ignoring $GEST_STORAGE__CACHE_DIR");
      log::warn!("ignoring $GEST_STORAGE__CACHE_DIR: path is not absolute");
    } else {
      log::debug!("$GEST_STORAGE__CACHE_DIR is not set");
    }

    if let Some(path) = &self.cache_dir {
      if path.is_absolute() {
        log::debug!("storage.cache_dir is set to {:?}", path.display());
        log::trace!("using storage.cache_dir");
        return Ok(path.clone());
      }
      log::debug!("storage.cache_dir: {:?} is not absolute", path.display());
      log::trace!("ignoring storage.cache_dir");
      log::warn!("ignoring storage.cache_dir: path is not absolute");
    } else {
      log::debug!("storage.cache_dir is not set");
    }

    log::trace!("falling back to XDG cache home");
    dir_spec::cache_home()
      .map(|path| path.join("gest"))
      .ok_or(Error::XDGDirNotFound("cache"))
  }

  /// Resolves the data directory using the following precedence:
  ///
  /// 1. `GEST_STORAGE__DATA_DIR` environment variable (must be absolute)
  /// 2. `storage.data_dir` from the config file (must be absolute)
  /// 3. XDG data home (`$XDG_DATA_HOME/gest`)
  pub fn data_dir(&self) -> Result<PathBuf, Error> {
    if let Ok(path) = GEST_STORAGE__DATA_DIR.value() {
      if path.is_absolute() {
        log::debug!("$GEST_STORAGE__DATA_DIR is set to {:?}", path.display());
        log::trace!("using $GEST_STORAGE__DATA_DIR");
        return Ok(path);
      }
      log::debug!("$GEST_STORAGE__DATA_DIR: {:?} is not absolute", path.display());
      log::trace!("ignoring $GEST_STORAGE__DATA_DIR");
      log::warn!("ignoring $GEST_STORAGE__DATA_DIR: path is not absolute");
    } else {
      log::debug!("$GEST_STORAGE__DATA_DIR is not set");
    }

    if let Some(path) = &self.data_dir {
      if path.is_absolute() {
        log::debug!("storage.data_dir is set to {:?}", path.display());
        log::trace!("using storage.data_dir");
        return Ok(path.clone());
      }
      log::debug!("storage.data_dir: {:?} is not absolute", path.display());
      log::trace!("ignoring storage.data_dir");
      log::warn!("ignoring storage.data_dir: path is not absolute");
    } else {
      log::debug!("storage.data_dir is not set");
    }

    log::trace!("falling back to XDG data home");
    dir_spec::data_home()
      .map(|path| path.join("gest"))
      .ok_or(Error::XDGDirNotFound("data"))
  }

  /// Whether automatic file sync with `.gest/` directories is enabled.
  ///
  /// Precedence:
  /// 1. `GEST_STORAGE__SYNC` environment variable (boolean)
  /// 2. `storage.sync` from the config file
  /// 3. Defaults to `true`
  pub fn sync_enabled(&self) -> bool {
    if let Ok(v) = GEST_STORAGE__SYNC.value() {
      return v;
    }
    self.sync.unwrap_or(true)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  mod data_dir {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_falls_back_to_xdg_data_home() {
      let settings = Settings {
        cache_dir: None,
        data_dir: None,
        sync: None,
      };

      temp_env::with_var("GEST_STORAGE__DATA_DIR", None::<&str>, || {
        let result = settings.data_dir().unwrap();
        let expected = dir_spec::data_home().unwrap().join("gest");

        assert_eq!(result, expected);
      });
    }

    #[test]
    fn it_ignores_relative_config_value() {
      let settings = Settings {
        cache_dir: None,
        data_dir: Some(PathBuf::from("relative/path")),
        sync: None,
      };

      temp_env::with_var("GEST_STORAGE__DATA_DIR", None::<&str>, || {
        let result = settings.data_dir().unwrap();
        let expected = dir_spec::data_home().unwrap().join("gest");

        assert_eq!(result, expected);
      });
    }

    #[test]
    fn it_ignores_relative_env_var() {
      let settings = Settings {
        cache_dir: None,
        data_dir: Some(PathBuf::from("/from/config")),
        sync: None,
      };

      temp_env::with_var("GEST_STORAGE__DATA_DIR", Some("relative/path"), || {
        let result = settings.data_dir().unwrap();

        assert_eq!(result, PathBuf::from("/from/config"));
      });
    }

    #[test]
    fn it_prefers_env_var_over_config_and_xdg() {
      let settings = Settings {
        cache_dir: None,
        data_dir: Some(PathBuf::from("/from/config")),
        sync: None,
      };

      temp_env::with_var("GEST_STORAGE__DATA_DIR", Some("/from/env"), || {
        let result = settings.data_dir().unwrap();

        assert_eq!(result, PathBuf::from("/from/env"));
      });
    }

    #[test]
    fn it_uses_config_value_when_env_var_is_unset() {
      let settings = Settings {
        cache_dir: None,
        data_dir: Some(PathBuf::from("/from/config")),
        sync: None,
      };

      temp_env::with_var("GEST_STORAGE__DATA_DIR", None::<&str>, || {
        let result = settings.data_dir().unwrap();

        assert_eq!(result, PathBuf::from("/from/config"));
      });
    }
  }

  mod sync_enabled {
    use super::*;

    #[test]
    fn it_defaults_to_true() {
      let settings = Settings::default();

      temp_env::with_var("GEST_STORAGE__SYNC", None::<&str>, || {
        assert!(settings.sync_enabled());
      });
    }

    #[test]
    fn it_env_var_can_enable_over_config() {
      let settings = Settings {
        cache_dir: None,
        data_dir: None,
        sync: Some(false),
      };

      temp_env::with_var("GEST_STORAGE__SYNC", Some("true"), || {
        assert!(settings.sync_enabled());
      });
    }

    #[test]
    fn it_env_var_overrides_config() {
      let settings = Settings {
        cache_dir: None,
        data_dir: None,
        sync: Some(true),
      };

      temp_env::with_var("GEST_STORAGE__SYNC", Some("false"), || {
        assert!(!settings.sync_enabled());
      });
    }

    #[test]
    fn it_returns_false_when_config_sync_is_false() {
      let settings = Settings {
        cache_dir: None,
        data_dir: None,
        sync: Some(false),
      };

      temp_env::with_var("GEST_STORAGE__SYNC", None::<&str>, || {
        assert!(!settings.sync_enabled());
      });
    }

    #[test]
    fn it_returns_true_when_config_sync_is_true() {
      let settings = Settings {
        cache_dir: None,
        data_dir: None,
        sync: Some(true),
      };

      temp_env::with_var("GEST_STORAGE__SYNC", None::<&str>, || {
        assert!(settings.sync_enabled());
      });
    }
  }
}
