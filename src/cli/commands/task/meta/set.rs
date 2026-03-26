use chrono::Utc;
use clap::Args;

use crate::{
  config,
  config::Config,
  store,
  ui::{components::MetadataSet, theme::Theme},
};

/// Set a metadata value on a task
#[derive(Debug, Args)]
pub struct Command {
  /// Task ID or unique prefix
  pub id: String,
  /// Dot-delimited key path (e.g. outer.inner)
  pub path: String,
  /// Value to set (strings, numbers, and booleans are auto-detected)
  pub value: String,
}

impl Command {
  pub fn call(&self, config: &Config, _theme: &Theme) -> crate::Result<()> {
    let data_dir = config::data_dir(config)?;
    let id = store::resolve_task_id(&data_dir, &self.id, false)?;
    let mut task = store::read_task(&data_dir, &id)?;

    set_dot_path(&mut task.metadata, &self.path, &self.value)?;

    task.updated_at = Utc::now();
    store::write_task(&data_dir, &task)?;

    MetadataSet::new(&id, &self.path, &self.value).write_to(&mut std::io::stdout())?;
    Ok(())
  }
}

fn parse_toml_value(s: &str) -> toml::Value {
  if let Ok(n) = s.parse::<i64>() {
    return toml::Value::Integer(n);
  }
  if let Ok(n) = s.parse::<f64>() {
    return toml::Value::Float(n);
  }
  match s {
    "true" => toml::Value::Boolean(true),
    "false" => toml::Value::Boolean(false),
    _ => toml::Value::String(s.to_string()),
  }
}

fn set_dot_path(table: &mut toml::Table, path: &str, value: &str) -> crate::Result<()> {
  let segments: Vec<&str> = path.split('.').collect();
  let toml_value = parse_toml_value(value);

  if segments.len() == 1 {
    table.insert(segments[0].to_string(), toml_value);
    return Ok(());
  }

  set_nested(table, &segments, toml_value);
  Ok(())
}

fn set_nested(table: &mut toml::Table, segments: &[&str], value: toml::Value) {
  let key = segments[0].to_string();

  if segments.len() == 1 {
    table.insert(key, value);
    return;
  }

  let nested = table
    .entry(&key)
    .or_insert_with(|| toml::Value::Table(toml::Table::new()));

  if let toml::Value::Table(t) = nested {
    set_nested(t, &segments[1..], value);
  } else {
    let mut new_table = toml::Table::new();
    set_nested(&mut new_table, &segments[1..], value);
    table.insert(key, toml::Value::Table(new_table));
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  mod call {
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::test_helpers::{make_test_config, make_test_task};

    #[test]
    fn it_sets_metadata_value() {
      let dir = tempfile::tempdir().unwrap();
      let config = make_test_config(dir.path());
      let task = make_test_task("zyxwvutsrqponmlkzyxwvutsrqponmlk");
      store::write_task(dir.path(), &task).unwrap();

      let cmd = Command {
        id: "zyxw".to_string(),
        path: "priority".to_string(),
        value: "high".to_string(),
      };
      cmd.call(&config, &Theme::default()).unwrap();

      let loaded = store::read_task(dir.path(), &task.id).unwrap();
      assert_eq!(
        loaded.metadata.get("priority"),
        Some(&toml::Value::String("high".to_string()))
      );
    }

    #[test]
    fn it_sets_nested_metadata_value() {
      let dir = tempfile::tempdir().unwrap();
      let config = make_test_config(dir.path());
      let task = make_test_task("zyxwvutsrqponmlkzyxwvutsrqponmlk");
      store::write_task(dir.path(), &task).unwrap();

      let cmd = Command {
        id: "zyxw".to_string(),
        path: "config.timeout".to_string(),
        value: "30".to_string(),
      };
      cmd.call(&config, &Theme::default()).unwrap();

      let loaded = store::read_task(dir.path(), &task.id).unwrap();
      let config = loaded.metadata.get("config").unwrap().as_table().unwrap();
      assert_eq!(config.get("timeout"), Some(&toml::Value::Integer(30)));
    }
  }

  mod parse_toml_value {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_falls_back_to_string() {
      assert_eq!(parse_toml_value("hello"), toml::Value::String("hello".to_string()));
    }

    #[test]
    fn it_parses_booleans() {
      assert_eq!(parse_toml_value("true"), toml::Value::Boolean(true));
      assert_eq!(parse_toml_value("false"), toml::Value::Boolean(false));
    }

    #[test]
    fn it_parses_floats() {
      assert_eq!(parse_toml_value("3.14"), toml::Value::Float(3.14));
    }

    #[test]
    fn it_parses_integers() {
      assert_eq!(parse_toml_value("42"), toml::Value::Integer(42));
    }

    #[test]
    fn it_parses_negative_integers() {
      assert_eq!(parse_toml_value("-7"), toml::Value::Integer(-7));
    }
  }

  mod set_dot_path {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_creates_deeply_nested_path() {
      let mut table = toml::Table::new();
      set_dot_path(&mut table, "a.b.c", "leaf").unwrap();

      let a = table.get("a").unwrap().as_table().unwrap();
      let b = a.get("b").unwrap().as_table().unwrap();
      assert_eq!(b.get("c"), Some(&toml::Value::String("leaf".to_string())));
    }

    #[test]
    fn it_overwrites_existing_value() {
      let mut table = toml::Table::new();
      table.insert("key".to_string(), toml::Value::String("old".to_string()));
      set_dot_path(&mut table, "key", "new").unwrap();
      assert_eq!(table.get("key"), Some(&toml::Value::String("new".to_string())));
    }

    #[test]
    fn it_preserves_sibling_keys_in_nested_path() {
      let mut table = toml::Table::new();
      set_dot_path(&mut table, "outer.first", "one").unwrap();
      set_dot_path(&mut table, "outer.second", "two").unwrap();

      let outer = table.get("outer").unwrap().as_table().unwrap();
      assert_eq!(outer.get("first"), Some(&toml::Value::String("one".to_string())));
      assert_eq!(outer.get("second"), Some(&toml::Value::String("two".to_string())));
    }

    #[test]
    fn it_sets_nested_key() {
      let mut table = toml::Table::new();
      set_dot_path(&mut table, "outer.inner", "deep").unwrap();

      let outer = table.get("outer").unwrap();
      if let toml::Value::Table(inner) = outer {
        assert_eq!(inner.get("inner"), Some(&toml::Value::String("deep".to_string())));
      } else {
        panic!("Expected table at 'outer'");
      }
    }

    #[test]
    fn it_sets_top_level_key() {
      let mut table = toml::Table::new();
      set_dot_path(&mut table, "key", "value").unwrap();
      assert_eq!(table.get("key"), Some(&toml::Value::String("value".to_string())));
    }
  }
}
