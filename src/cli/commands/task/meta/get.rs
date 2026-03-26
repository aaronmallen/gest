use clap::Args;

use crate::{config, config::Config, store};

/// Get a metadata value from a task
#[derive(Debug, Args)]
pub struct Command {
  /// Task ID or unique prefix
  pub id: String,
  /// Dot-delimited key path (e.g. outer.inner)
  pub path: String,
}

impl Command {
  pub fn call(&self, config: &Config) -> crate::Result<()> {
    let data_dir = config::data_dir(config)?;
    let id = store::resolve_task_id(&data_dir, &self.id, false)?;
    let task = store::read_task(&data_dir, &id)?;

    let value = resolve_dot_path(&toml::Value::Table(task.metadata), &self.path)
      .ok_or_else(|| crate::Error::generic(format!("Metadata key not found: '{}'", self.path)))?;

    print_toml_value(&value);
    Ok(())
  }
}

fn print_toml_value(value: &toml::Value) {
  match value {
    toml::Value::String(s) => println!("{s}"),
    toml::Value::Boolean(b) => println!("{b}"),
    toml::Value::Integer(n) => println!("{n}"),
    toml::Value::Float(n) => println!("{n}"),
    toml::Value::Datetime(dt) => println!("{dt}"),
    toml::Value::Array(arr) => {
      let json = serde_json::to_string_pretty(arr).unwrap_or_else(|_| format!("{arr:?}"));
      println!("{json}");
    }
    toml::Value::Table(t) => {
      let json = serde_json::to_string_pretty(t).unwrap_or_else(|_| format!("{t:?}"));
      println!("{json}");
    }
  }
}

fn resolve_dot_path(root: &toml::Value, path: &str) -> Option<toml::Value> {
  let segments: Vec<&str> = path.split('.').collect();
  let mut current = root.clone();

  for segment in &segments {
    match current {
      toml::Value::Table(t) => {
        current = t.get(*segment)?.clone();
      }
      _ => return None,
    }
  }

  Some(current)
}

#[cfg(test)]
mod tests {
  use super::*;

  mod call {
    use super::*;
    use crate::test_helpers::{make_test_config, make_test_task};

    #[test]
    fn it_errors_on_missing_path() {
      let dir = tempfile::tempdir().unwrap();
      let config = make_test_config(dir.path());
      let task = make_test_task("zyxwvutsrqponmlkzyxwvutsrqponmlk");
      store::write_task(dir.path(), &task).unwrap();

      let cmd = Command {
        id: "zyxw".to_string(),
        path: "nonexistent".to_string(),
      };
      let result = cmd.call(&config);
      assert!(result.is_err());
    }

    #[test]
    fn it_reads_metadata_value() {
      let dir = tempfile::tempdir().unwrap();
      let config = make_test_config(dir.path());
      let mut task = make_test_task("zyxwvutsrqponmlkzyxwvutsrqponmlk");
      task
        .metadata
        .insert("priority".to_string(), toml::Value::String("high".to_string()));
      store::write_task(dir.path(), &task).unwrap();

      let cmd = Command {
        id: "zyxw".to_string(),
        path: "priority".to_string(),
      };
      cmd.call(&config).unwrap();
    }
  }

  mod resolve_dot_path {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_resolves_nested_key() {
      let mut inner = toml::Table::new();
      inner.insert("nested".to_string(), toml::Value::String("deep".to_string()));
      let mut table = toml::Table::new();
      table.insert("outer".to_string(), toml::Value::Table(inner));
      let result = resolve_dot_path(&table_value(table), "outer.nested");
      assert_eq!(result, Some(toml::Value::String("deep".to_string())));
    }

    #[test]
    fn it_resolves_top_level_key() {
      let mut table = toml::Table::new();
      table.insert("key".to_string(), toml::Value::String("value".to_string()));
      let result = resolve_dot_path(&table_value(table), "key");
      assert_eq!(result, Some(toml::Value::String("value".to_string())));
    }

    #[test]
    fn it_returns_none_for_missing_key() {
      let table = toml::Table::new();
      let result = resolve_dot_path(&table_value(table), "missing");
      assert_eq!(result, None);
    }

    #[test]
    fn it_returns_none_for_non_table_intermediate() {
      let mut table = toml::Table::new();
      table.insert("key".to_string(), toml::Value::String("value".to_string()));
      let result = resolve_dot_path(&table_value(table), "key.nested");
      assert_eq!(result, None);
    }

    fn table_value(table: toml::Table) -> toml::Value {
      toml::Value::Table(table)
    }
  }
}
