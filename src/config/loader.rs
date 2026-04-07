use std::{env, fs, path::PathBuf};

use toml::{Value, value::Table};

use super::{Error, Settings, env::GEST_CONFIG};

/// Load and merge configuration from all sources.
///
/// Resolution order (each layer overrides the previous):
///
/// 1. Global config: `$GEST_CONFIG` or `$XDG_CONFIG_HOME/gest/config.toml`
/// 2. Per-directory configs walking from filesystem root to `$CWD`, checking `.config/gest.toml` and `.gest.toml` at
///    each level.
/// 3. Environment variables prefixed with `GEST_` (e.g. `GEST_STORAGE__DATA_DIR`).
pub fn load() -> Result<Settings, Error> {
  let mut merged = Table::new();

  if let Some(table) = read_toml(&global_config_path()?)? {
    merge_tables(&mut merged, table);
  }

  let cwd = env::current_dir().unwrap_or_default();
  let mut ancestors: Vec<PathBuf> = cwd.ancestors().map(PathBuf::from).collect();
  ancestors.reverse();

  for dir in ancestors {
    for candidate in [dir.join(".config/gest.toml"), dir.join(".gest.toml")] {
      if let Some(table) = read_toml(&candidate)? {
        merge_tables(&mut merged, table);
      }
    }
  }

  merge_env(&mut merged);

  let settings: Settings = Value::Table(merged).try_into()?;

  Ok(settings)
}

/// Resolve the path to the global config file, preferring `$GEST_CONFIG` over the XDG default.
fn global_config_path() -> Result<PathBuf, Error> {
  GEST_CONFIG
    .value()
    .ok()
    .or_else(|| dir_spec::config_home().map(|path| path.join("gest/config.toml")))
    .ok_or(Error::XDGDirNotFound("config"))
}

/// Merge environment variables prefixed with `GEST_` into the config table.
///
/// `__` separates nesting levels and key segments are lowercased.
/// For example, `GEST_STORAGE__DATA_DIR=foo` becomes `[storage] data_dir = "foo"`.
fn merge_env(base: &mut Table) {
  const PREFIX: &str = "GEST_";

  for (key, value) in env::vars() {
    if !key.starts_with(PREFIX) || key == GEST_CONFIG.name() {
      continue;
    }

    let path: Vec<&str> = key[PREFIX.len()..].split("__").collect();
    let lowered: Vec<String> = path.iter().map(|s| s.to_lowercase()).collect();

    let mut table = &mut *base;
    for segment in &lowered[..lowered.len() - 1] {
      table = table
        .entry(segment.clone())
        .or_insert_with(|| Value::Table(Table::new()))
        .as_table_mut()
        .unwrap_or_else(|| panic!("config key `{segment}` is not a table"));
    }

    table.insert(lowered.last().unwrap().clone(), Value::String(value));
  }
}

/// Recursively merge `overlay` into `base`. Nested tables are merged; all other values are overwritten.
fn merge_tables(base: &mut Table, overlay: Table) {
  for (key, overlay_value) in overlay {
    if let Value::Table(overlay_table) = &overlay_value
      && let Some(Value::Table(base_table)) = base.get_mut(&key)
    {
      merge_tables(base_table, overlay_table.clone());
      continue;
    }
    base.insert(key, overlay_value);
  }
}

/// Read and parse a TOML file into a [`Table`]. Returns `Ok(None)` if the file does not exist.
fn read_toml(path: &PathBuf) -> Result<Option<Table>, Error> {
  if !path.is_file() {
    return Ok(None);
  }

  let content = fs::read_to_string(path)?;
  let table: Table = toml::from_str(&content)?;

  Ok(Some(table))
}

#[cfg(test)]
mod tests {
  use std::io::Write;

  use tempfile::TempDir;
  use toml::Value;

  use super::*;

  mod load {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_merges_per_directory_config_into_settings() {
      let dir = TempDir::new().unwrap();
      let config_path = dir.path().join(".gest.toml");

      let mut file = fs::File::create(&config_path).unwrap();
      file.write_all(b"[storage]\ndata_dir = \"/custom/data\"\n").unwrap();

      temp_env::with_vars(
        [
          ("GEST_CONFIG", None::<&str>),
          ("GEST_DATA_DIR", None::<&str>),
          ("XDG_CONFIG_HOME", Some(dir.path().join(".config").to_str().unwrap())),
        ],
        || {
          let original_dir = env::current_dir().unwrap();
          env::set_current_dir(dir.path()).unwrap();

          let settings = load().unwrap();

          env::set_current_dir(original_dir).unwrap();

          assert_eq!(
            settings.storage().data_dir().unwrap(),
            std::path::PathBuf::from("/custom/data")
          );
        },
      );
    }
  }

  mod merge_env {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_inserts_a_flat_key() {
      temp_env::with_var("GEST_TESTFLAT", Some("hello"), || {
        let mut table = Table::new();
        merge_env(&mut table);

        assert_eq!(table.get("testflat").unwrap().as_str().unwrap(), "hello");
      });
    }

    #[test]
    fn it_inserts_a_nested_key() {
      temp_env::with_var("GEST_SECTION__NESTED_KEY", Some("val"), || {
        let mut table = Table::new();
        merge_env(&mut table);

        let section = table.get("section").unwrap().as_table().unwrap();
        assert_eq!(section.get("nested_key").unwrap().as_str().unwrap(), "val");
      });
    }

    #[test]
    fn it_lowercases_key_segments() {
      temp_env::with_var("GEST_UPPER__CASE", Some("low"), || {
        let mut table = Table::new();
        merge_env(&mut table);

        let section = table.get("upper").unwrap().as_table().unwrap();
        assert_eq!(section.get("case").unwrap().as_str().unwrap(), "low");
      });
    }

    #[test]
    fn it_skips_gest_config_env_var() {
      temp_env::with_var("GEST_CONFIG", Some("/some/path"), || {
        let mut table = Table::new();
        merge_env(&mut table);

        assert!(table.get("config").is_none());
      });
    }
  }

  mod merge_tables {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_inserts_new_keys() {
      let mut base = Table::new();
      let mut overlay = Table::new();
      overlay.insert("key".into(), Value::String("value".into()));

      merge_tables(&mut base, overlay);

      assert_eq!(base.get("key").unwrap().as_str().unwrap(), "value");
    }

    #[test]
    fn it_overwrites_scalar_values() {
      let mut base = Table::new();
      base.insert("key".into(), Value::String("old".into()));

      let mut overlay = Table::new();
      overlay.insert("key".into(), Value::String("new".into()));

      merge_tables(&mut base, overlay);

      assert_eq!(base.get("key").unwrap().as_str().unwrap(), "new");
    }

    #[test]
    fn it_recursively_merges_nested_tables() {
      let mut inner_base = Table::new();
      inner_base.insert("a".into(), Value::String("1".into()));
      inner_base.insert("b".into(), Value::String("2".into()));

      let mut base = Table::new();
      base.insert("section".into(), Value::Table(inner_base));

      let mut inner_overlay = Table::new();
      inner_overlay.insert("b".into(), Value::String("overridden".into()));
      inner_overlay.insert("c".into(), Value::String("3".into()));

      let mut overlay = Table::new();
      overlay.insert("section".into(), Value::Table(inner_overlay));

      merge_tables(&mut base, overlay);

      let section = base.get("section").unwrap().as_table().unwrap();
      assert_eq!(section.get("a").unwrap().as_str().unwrap(), "1");
      assert_eq!(section.get("b").unwrap().as_str().unwrap(), "overridden");
      assert_eq!(section.get("c").unwrap().as_str().unwrap(), "3");
    }

    #[test]
    fn it_replaces_scalar_with_table() {
      let mut base = Table::new();
      base.insert("key".into(), Value::String("scalar".into()));

      let mut inner = Table::new();
      inner.insert("nested".into(), Value::String("value".into()));

      let mut overlay = Table::new();
      overlay.insert("key".into(), Value::Table(inner));

      merge_tables(&mut base, overlay);

      let table = base.get("key").unwrap().as_table().unwrap();
      assert_eq!(table.get("nested").unwrap().as_str().unwrap(), "value");
    }

    #[test]
    fn it_replaces_table_with_scalar() {
      let mut inner = Table::new();
      inner.insert("nested".into(), Value::String("value".into()));

      let mut base = Table::new();
      base.insert("key".into(), Value::Table(inner));

      let mut overlay = Table::new();
      overlay.insert("key".into(), Value::String("scalar".into()));

      merge_tables(&mut base, overlay);

      assert_eq!(base.get("key").unwrap().as_str().unwrap(), "scalar");
    }
  }

  mod read_toml {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_returns_none_for_missing_file() {
      let dir = TempDir::new().unwrap();
      let path = dir.path().join("nonexistent.toml");

      let result = read_toml(&path).unwrap();

      assert!(result.is_none());
    }

    #[test]
    fn it_parses_a_valid_toml_file() {
      let dir = TempDir::new().unwrap();
      let path = dir.path().join("config.toml");

      let mut file = fs::File::create(&path).unwrap();
      file.write_all(b"[section]\nkey = \"value\"\n").unwrap();

      let result = read_toml(&path).unwrap().unwrap();

      let section = result.get("section").unwrap().as_table().unwrap();
      assert_eq!(section.get("key").unwrap().as_str().unwrap(), "value");
    }

    #[test]
    fn it_returns_an_error_for_invalid_toml() {
      let dir = TempDir::new().unwrap();
      let path = dir.path().join("bad.toml");

      let mut file = fs::File::create(&path).unwrap();
      file.write_all(b"not valid [[ toml").unwrap();

      let result = read_toml(&path);

      assert!(result.is_err());
    }
  }
}
