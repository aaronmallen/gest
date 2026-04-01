//! Hierarchical TOML configuration loader.
//!
//! Merges the global config with per-directory configs found by walking
//! from the filesystem root down to `cwd`. Closer configs win.

use std::path::{Path, PathBuf};

use toml::{Table, Value};

use super::{Error, Settings, env::GEST_CONFIG};

/// Project-level config file names, checked in priority order within each directory.
const CONFIG_NAMES: &[&str] = &[".config/gest.toml", ".gest/config.toml", ".gest.toml"];

/// Filename for the global config under the user's config home.
const GLOBAL_CONFIG_NAME: &str = "config.toml";

/// Loads and merges configuration from global and per-directory TOML files.
///
/// Storage paths are resolved automatically so the returned [`Settings`] is
/// ready to use immediately.
pub fn load(cwd: &Path) -> Result<Settings, Error> {
  let mut merged = empty_table();

  deep_merge(&mut merged, load_global()?);
  for dir in ancestors_root_first(cwd) {
    deep_merge(&mut merged, load_first_match(&dir, CONFIG_NAMES)?);
  }

  let mut settings: Settings = merged
    .try_into()
    .map_err(|e: toml::de::Error| Error::Config(e.to_string()))?;

  settings.storage.resolve(cwd.to_path_buf())?;

  Ok(settings)
}

/// Returns all ancestor directories of `path`, ordered from root to `path` itself.
fn ancestors_root_first(path: &Path) -> Vec<PathBuf> {
  let mut dirs: Vec<_> = path.ancestors().map(Path::to_path_buf).collect();
  dirs.reverse();
  dirs
}

/// Recursively merges `overlay` into `base`, with overlay values taking precedence.
fn deep_merge(base: &mut Value, overlay: Value) {
  match (base, overlay) {
    (Value::Table(base_map), Value::Table(overlay_map)) => {
      for (key, value) in overlay_map {
        deep_merge(base_map.entry(key).or_insert(empty_table()), value);
      }
    }
    (base, overlay) => *base = overlay,
  }
}

fn empty_table() -> Value {
  Value::Table(Table::new())
}

/// Returns the parsed TOML of the first file in `names` that exists under `dir`.
fn load_first_match(dir: &Path, names: &[&str]) -> Result<Value, Error> {
  for name in names {
    let value = read_toml(&dir.join(name))?;
    if value != empty_table() {
      return Ok(value);
    }
  }
  Ok(empty_table())
}

/// Loads the global config from `$GEST_CONFIG` or the platform config home.
fn load_global() -> Result<Value, Error> {
  if let Ok(path) = GEST_CONFIG.value() {
    log::debug!("loading global config from $GEST_CONFIG: {}", path.display());
    return read_toml(&path);
  }

  if let Some(config_home) = dir_spec::config_home() {
    let path = config_home.join("gest").join(GLOBAL_CONFIG_NAME);
    log::debug!("searching for global config at {}", path.display());
    return read_toml(&path);
  }

  log::trace!("no global config home found");
  Ok(empty_table())
}

/// Reads and parses a TOML file, returning an empty table if the file does not exist.
fn read_toml(path: &Path) -> Result<Value, Error> {
  match std::fs::read_to_string(path) {
    Ok(content) => {
      log::trace!("loaded config: {}", path.display());
      toml::from_str(&content).map_err(|e| Error::Config(e.to_string()))
    }
    Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(empty_table()),
    Err(e) => Err(e.into()),
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  mod ancestors_root_first {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_returns_ancestors_ordered_from_root_to_path() {
      let dirs = ancestors_root_first(Path::new("/a/b/c"));

      assert_eq!(
        dirs,
        vec![
          PathBuf::from("/"),
          PathBuf::from("/a"),
          PathBuf::from("/a/b"),
          PathBuf::from("/a/b/c"),
        ]
      );
    }

    #[test]
    fn it_returns_root_for_root_path() {
      let dirs = ancestors_root_first(Path::new("/"));

      assert_eq!(dirs, vec![PathBuf::from("/")]);
    }
  }

  mod deep_merge {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_adds_new_keys() {
      let mut base: Value = toml::from_str("[storage]\ndata_dir = \"/a\"").unwrap();
      let overlay: Value = toml::from_str("name = \"test\"").unwrap();
      deep_merge(&mut base, overlay);

      assert_eq!(base["storage"]["data_dir"], Value::String("/a".into()));
      assert_eq!(base["name"], Value::String("test".into()));
    }

    #[test]
    fn it_merges_nested_tables() {
      let mut base: Value = toml::from_str("[storage]\ndata_dir = \"/a\"").unwrap();
      let overlay: Value = toml::from_str("[storage]\nbackup = true").unwrap();
      deep_merge(&mut base, overlay);

      assert_eq!(base["storage"]["data_dir"], Value::String("/a".into()));
      assert_eq!(base["storage"]["backup"], Value::Boolean(true));
    }

    #[test]
    fn it_overwrites_scalars() {
      let mut base: Value = toml::from_str("[storage]\ndata_dir = \"/a\"").unwrap();
      let overlay: Value = toml::from_str("[storage]\ndata_dir = \"/b\"").unwrap();
      deep_merge(&mut base, overlay);

      assert_eq!(base["storage"]["data_dir"], Value::String("/b".into()));
    }
  }

  mod load {
    use pretty_assertions::assert_eq;
    use temp_env::with_vars_unset;
    use tempfile::TempDir;

    use super::*;

    const UNSET_VARS: [&str; 2] = ["GEST_CONFIG", "GEST_DATA_DIR"];

    #[test]
    fn it_loads_config_from_dot_config_gest_toml() {
      let tmp = TempDir::new().unwrap();
      let data_dir = tmp.path().join("data");
      std::fs::create_dir_all(&data_dir).unwrap();
      std::fs::create_dir_all(tmp.path().join(".config")).unwrap();
      std::fs::write(
        tmp.path().join(".config/gest.toml"),
        format!("[storage]\ndata_dir = \"{}\"", data_dir.display()),
      )
      .unwrap();

      with_vars_unset(UNSET_VARS, || {
        let settings = load(tmp.path()).unwrap();
        assert_eq!(
          settings.storage().resolve_data_dir(tmp.path().into()).unwrap(),
          data_dir
        );
      })
    }

    #[test]
    fn it_loads_config_from_gest_config_toml() {
      let tmp = TempDir::new().unwrap();
      let data_dir = tmp.path().join("data");
      std::fs::create_dir_all(&data_dir).unwrap();
      std::fs::create_dir_all(tmp.path().join(".gest")).unwrap();
      std::fs::write(
        tmp.path().join(".gest/config.toml"),
        format!("[storage]\ndata_dir = \"{}\"", data_dir.display()),
      )
      .unwrap();

      with_vars_unset(UNSET_VARS, || {
        let settings = load(tmp.path()).unwrap();
        assert_eq!(
          settings.storage().resolve_data_dir(tmp.path().into()).unwrap(),
          data_dir
        );
      })
    }

    #[test]
    fn it_loads_config_from_gest_toml() {
      let tmp = TempDir::new().unwrap();
      let data_dir = tmp.path().join("data");
      std::fs::create_dir_all(&data_dir).unwrap();
      std::fs::write(
        tmp.path().join(".gest.toml"),
        format!("[storage]\ndata_dir = \"{}\"", data_dir.display()),
      )
      .unwrap();

      with_vars_unset(UNSET_VARS, || {
        let settings = load(tmp.path()).unwrap();
        assert_eq!(
          settings.storage().resolve_data_dir(tmp.path().into()).unwrap(),
          data_dir
        );
      })
    }

    #[test]
    fn it_loads_default_settings_when_no_config_exists() {
      let tmp = TempDir::new().unwrap();

      with_vars_unset(UNSET_VARS, || {
        let settings = load(tmp.path()).unwrap();
        // Config values should match defaults; resolved storage paths will differ.
        assert_eq!(settings.colors(), &crate::config::colors::Settings::default());
        assert_eq!(settings.log(), &crate::config::log::Settings::default());
        assert_eq!(settings.serve(), &crate::config::serve::Settings::default());
      })
    }

    #[test]
    fn it_merges_child_config_over_parent() {
      let tmp = TempDir::new().unwrap();
      let parent_dir = tmp.path().join("parent_data");
      let child_dir = tmp.path().join("child_data");
      let child = tmp.path().join("child");
      std::fs::create_dir_all(&parent_dir).unwrap();
      std::fs::create_dir_all(&child_dir).unwrap();
      std::fs::create_dir_all(&child).unwrap();

      std::fs::write(
        tmp.path().join(".gest.toml"),
        format!("[storage]\ndata_dir = \"{}\"", parent_dir.display()),
      )
      .unwrap();
      std::fs::write(
        child.join(".gest.toml"),
        format!("[storage]\ndata_dir = \"{}\"", child_dir.display()),
      )
      .unwrap();

      with_vars_unset(UNSET_VARS, || {
        let settings = load(&child).unwrap();
        assert_eq!(settings.storage().resolve_data_dir(child.clone()).unwrap(), child_dir);
      })
    }
  }

  mod load_first_match {
    use pretty_assertions::assert_eq;
    use tempfile::TempDir;

    use super::*;

    #[test]
    fn it_returns_empty_table_when_no_files_exist() {
      let tmp = TempDir::new().unwrap();
      let value = load_first_match(tmp.path(), CONFIG_NAMES).unwrap();

      assert_eq!(value, empty_table());
    }

    #[test]
    fn it_returns_first_matching_file() {
      let tmp = TempDir::new().unwrap();
      std::fs::create_dir_all(tmp.path().join(".config")).unwrap();
      std::fs::write(tmp.path().join(".config/gest.toml"), "[storage]\ndata_dir = \"/first\"").unwrap();
      std::fs::write(tmp.path().join(".gest.toml"), "[storage]\ndata_dir = \"/second\"").unwrap();

      let value = load_first_match(tmp.path(), CONFIG_NAMES).unwrap();

      assert_eq!(value["storage"]["data_dir"], Value::String("/first".into()));
    }
  }

  mod load_global {
    use pretty_assertions::assert_eq;
    use temp_env::{with_var, with_var_unset};
    use tempfile::TempDir;

    use super::*;

    #[test]
    fn it_loads_from_gest_config_env_var() {
      let tmp = TempDir::new().unwrap();
      let path = tmp.path().join("custom.toml");
      std::fs::write(&path, "[storage]\ndata_dir = \"/custom\"").unwrap();

      with_var("GEST_CONFIG", Some(path.to_str().unwrap()), || {
        let value = load_global().unwrap();
        assert_eq!(value["storage"]["data_dir"], Value::String("/custom".into()));
      })
    }

    #[test]
    fn it_returns_empty_table_when_no_global_config_exists() {
      with_var_unset("GEST_CONFIG", || {
        let value = load_global().unwrap();
        assert_eq!(value, empty_table());
      })
    }
  }

  mod read_toml {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_parses_a_toml_file() {
      let tmp = tempfile::tempdir().unwrap();
      let path = tmp.path().join("config.toml");
      std::fs::write(&path, "[storage]\ndata_dir = \"/tmp/gest\"").unwrap();

      let value = read_toml(&path).unwrap();

      assert_eq!(value["storage"]["data_dir"], Value::String("/tmp/gest".into()));
    }

    #[test]
    fn it_returns_an_empty_table_when_not_found() {
      let value = read_toml(Path::new("/nonexistent/config.toml")).unwrap();
      assert_eq!(value, empty_table());
    }
  }
}
