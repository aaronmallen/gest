//! Shared abstraction over TOML and YAML value types for dot-path metadata operations.

use serde::Serialize;

/// Maximum number of dot-delimited segments allowed in a key path.
const MAX_DEPTH: usize = 32;

// ── Traits ──────────────────────────────────────────────────────────────────────────────────────

/// Abstraction over a map-like container (TOML `Table` or YAML `Mapping`).
pub trait MetaTable: Sized {
  /// The value type stored in this table.
  type Value: MetaValue<Table = Self>;

  /// Look up a key by name.
  fn get_key(&self, key: &str) -> Option<&Self::Value>;

  /// Insert a key-value pair, replacing any previous value for the key.
  fn insert_key(&mut self, key: String, value: Self::Value);

  /// Create a new, empty table.
  fn new_empty() -> Self;

  /// Get a mutable reference to an existing entry, or insert a default and return it.
  fn get_or_insert_table(&mut self, key: &str) -> &mut Self::Value;
}

/// Abstraction over value types (TOML `Value` or YAML `Value`) for dot-path operations.
pub trait MetaValue: Clone + Serialize + std::fmt::Debug {
  /// The map/table type associated with this value type.
  type Table: MetaTable<Value = Self>;

  /// Try to interpret this value as a table, returning a reference to it.
  fn as_table(&self) -> Option<&Self::Table>;

  /// Try to interpret this value as a mutable table.
  fn as_table_mut(&mut self) -> Option<&mut Self::Table>;

  /// Wrap a table into a value.
  fn from_table(table: Self::Table) -> Self;

  /// Parse a string into the most specific scalar type for this value format.
  fn parse_scalar(s: &str) -> Self;

  /// Print this value to stdout in a human-friendly format.
  fn print(&self);
}

// ── TOML implementation ─────────────────────────────────────────────────────────────────────────

impl MetaTable for toml::value::Table {
  type Value = toml::Value;

  fn get_key(&self, key: &str) -> Option<&toml::Value> {
    self.get(key)
  }

  fn get_or_insert_table(&mut self, key: &str) -> &mut toml::Value {
    self
      .entry(key)
      .or_insert_with(|| toml::Value::Table(toml::value::Table::new()))
  }

  fn insert_key(&mut self, key: String, value: toml::Value) {
    self.insert(key, value);
  }

  fn new_empty() -> Self {
    toml::value::Table::new()
  }
}

impl MetaValue for toml::Value {
  type Table = toml::value::Table;

  fn as_table(&self) -> Option<&toml::value::Table> {
    self.as_table()
  }

  fn as_table_mut(&mut self) -> Option<&mut toml::value::Table> {
    self.as_table_mut()
  }

  fn from_table(table: toml::value::Table) -> Self {
    toml::Value::Table(table)
  }

  fn parse_scalar(s: &str) -> Self {
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

  fn print(&self) {
    match self {
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
}

// ── YAML implementation ─────────────────────────────────────────────────────────────────────────

impl MetaTable for yaml_serde::Mapping {
  type Value = yaml_serde::Value;

  fn get_key(&self, key: &str) -> Option<&yaml_serde::Value> {
    self.get(key)
  }

  fn get_or_insert_table(&mut self, key: &str) -> &mut yaml_serde::Value {
    let yaml_key = yaml_serde::Value::String(key.to_string());
    self
      .entry(yaml_key)
      .or_insert_with(|| yaml_serde::Value::Mapping(yaml_serde::Mapping::new()))
  }

  fn insert_key(&mut self, key: String, value: yaml_serde::Value) {
    self.insert(yaml_serde::Value::String(key), value);
  }

  fn new_empty() -> Self {
    yaml_serde::Mapping::new()
  }
}

impl MetaValue for yaml_serde::Value {
  type Table = yaml_serde::Mapping;

  fn as_table(&self) -> Option<&yaml_serde::Mapping> {
    self.as_mapping()
  }

  fn as_table_mut(&mut self) -> Option<&mut yaml_serde::Mapping> {
    self.as_mapping_mut()
  }

  fn from_table(table: yaml_serde::Mapping) -> Self {
    yaml_serde::Value::Mapping(table)
  }

  fn parse_scalar(s: &str) -> Self {
    if let Ok(n) = s.parse::<i64>() {
      return yaml_serde::Value::Number(yaml_serde::Number::from(n));
    }
    if let Ok(n) = s.parse::<f64>() {
      return yaml_serde::Value::Number(yaml_serde::Number::from(n));
    }
    match s {
      "true" => yaml_serde::Value::Bool(true),
      "false" => yaml_serde::Value::Bool(false),
      "null" => yaml_serde::Value::Null,
      _ => yaml_serde::Value::String(s.to_string()),
    }
  }

  fn print(&self) {
    match self {
      yaml_serde::Value::String(s) => println!("{s}"),
      yaml_serde::Value::Bool(b) => println!("{b}"),
      yaml_serde::Value::Number(n) => println!("{n}"),
      yaml_serde::Value::Null => println!("null"),
      yaml_serde::Value::Sequence(seq) => {
        let json = serde_json::to_string_pretty(seq).unwrap_or_else(|_| format!("{seq:?}"));
        println!("{json}");
      }
      yaml_serde::Value::Mapping(m) => {
        let json = serde_json::to_string_pretty(m).unwrap_or_else(|_| format!("{m:?}"));
        println!("{json}");
      }
      yaml_serde::Value::Tagged(t) => {
        t.value.print();
      }
    }
  }
}

// ── Generic functions ───────────────────────────────────────────────────────────────────────────

/// Walk a dot-delimited path through nested tables, returning a reference to the leaf value.
pub fn resolve_dot_path<'a, V: MetaValue>(root: &'a V, path: &str) -> Option<&'a V> {
  path
    .split('.')
    .try_fold(root, |current, seg| current.as_table()?.get_key(seg))
}

/// Insert a value at a dot-delimited path, creating intermediate tables as needed.
pub fn set_dot_path<T: MetaTable>(table: &mut T, path: &str, value: &str) -> super::Result<()> {
  let segments: Vec<&str> = path.split('.').collect();

  if segments.len() > MAX_DEPTH {
    return Err(super::Error::generic(format!(
      "key path exceeds maximum depth of {MAX_DEPTH} segments"
    )));
  }

  let parsed = T::Value::parse_scalar(value);

  if segments.len() == 1 {
    table.insert_key(segments[0].to_string(), parsed);
    return Ok(());
  }

  set_nested(table, &segments, parsed);
  Ok(())
}

/// Recursively descend into (or create) nested tables and insert the value at the final segment.
pub(crate) fn set_nested<T: MetaTable>(table: &mut T, segments: &[&str], value: T::Value) {
  let Some((&first, rest)) = segments.split_first() else {
    return;
  };

  if rest.is_empty() {
    table.insert_key(first.to_string(), value);
    return;
  }

  let nested = table.get_or_insert_table(first);

  if let Some(t) = nested.as_table_mut() {
    set_nested(t, rest, value);
  } else {
    let mut new_table = T::new_empty();
    set_nested(&mut new_table, rest, value);
    table.insert_key(first.to_string(), T::Value::from_table(new_table));
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  mod resolve_dot_path_toml {
    use pretty_assertions::assert_eq;
    use toml::{Value, value::Table};

    use super::*;

    #[test]
    fn it_resolves_nested_key() {
      let mut inner = Table::new();
      inner.insert("nested".to_string(), Value::String("deep".to_string()));
      let mut table = Table::new();
      table.insert("outer".to_string(), Value::Table(inner));
      let root = Value::Table(table);
      let result = resolve_dot_path(&root, "outer.nested");
      assert_eq!(result.cloned(), Some(Value::String("deep".to_string())));
    }

    #[test]
    fn it_resolves_top_level_key() {
      let mut table = Table::new();
      table.insert("key".to_string(), Value::String("value".to_string()));
      let root = Value::Table(table);
      let result = resolve_dot_path(&root, "key");
      assert_eq!(result.cloned(), Some(Value::String("value".to_string())));
    }

    #[test]
    fn it_returns_none_for_missing_key() {
      let table = Table::new();
      let root = Value::Table(table);
      let result = resolve_dot_path(&root, "missing");
      assert_eq!(result, None);
    }
  }

  mod resolve_dot_path_yaml {
    use pretty_assertions::assert_eq;
    use yaml_serde::{Mapping, Value};

    use super::*;

    #[test]
    fn it_resolves_nested_key() {
      let mut inner = Mapping::new();
      inner.insert(Value::String("nested".to_string()), Value::String("deep".to_string()));
      let mut mapping = Mapping::new();
      mapping.insert(Value::String("outer".to_string()), Value::Mapping(inner));
      let root = Value::Mapping(mapping);
      let result = resolve_dot_path(&root, "outer.nested");
      assert_eq!(result.cloned(), Some(Value::String("deep".to_string())));
    }

    #[test]
    fn it_resolves_top_level_key() {
      let mut mapping = Mapping::new();
      mapping.insert(Value::String("key".to_string()), Value::String("value".to_string()));
      let root = Value::Mapping(mapping);
      let result = resolve_dot_path(&root, "key");
      assert_eq!(result.cloned(), Some(Value::String("value".to_string())));
    }

    #[test]
    fn it_returns_none_for_missing_key() {
      let mapping = Mapping::new();
      let root = Value::Mapping(mapping);
      let result = resolve_dot_path(&root, "missing");
      assert_eq!(result, None);
    }
  }

  mod parse_scalar_toml {
    use pretty_assertions::assert_eq;
    use toml::Value;

    use super::*;

    #[test]
    fn it_falls_back_to_string() {
      assert_eq!(
        <toml::Value as MetaValue>::parse_scalar("hello"),
        Value::String("hello".to_string())
      );
    }

    #[test]
    fn it_parses_booleans() {
      assert_eq!(<toml::Value as MetaValue>::parse_scalar("true"), Value::Boolean(true));
      assert_eq!(<toml::Value as MetaValue>::parse_scalar("false"), Value::Boolean(false));
    }

    #[test]
    fn it_parses_floats() {
      assert_eq!(<toml::Value as MetaValue>::parse_scalar("3.14"), Value::Float(3.14));
    }

    #[test]
    fn it_parses_integers() {
      assert_eq!(<toml::Value as MetaValue>::parse_scalar("42"), Value::Integer(42));
    }
  }

  mod parse_scalar_yaml {
    use pretty_assertions::assert_eq;
    use yaml_serde::{Number, Value};

    use super::*;

    #[test]
    fn it_falls_back_to_string() {
      assert_eq!(
        <yaml_serde::Value as MetaValue>::parse_scalar("hello"),
        Value::String("hello".to_string())
      );
    }

    #[test]
    fn it_parses_booleans() {
      assert_eq!(
        <yaml_serde::Value as MetaValue>::parse_scalar("true"),
        Value::Bool(true)
      );
      assert_eq!(
        <yaml_serde::Value as MetaValue>::parse_scalar("false"),
        Value::Bool(false)
      );
    }

    #[test]
    fn it_parses_integers() {
      assert_eq!(
        <yaml_serde::Value as MetaValue>::parse_scalar("42"),
        Value::Number(Number::from(42))
      );
    }
  }
}
