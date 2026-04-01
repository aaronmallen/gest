//! Shared TOML metadata helpers for reading and writing dot-delimited key paths.

use toml::{Value, value::Table};

/// Maximum number of dot-delimited segments allowed in a key path.
const MAX_DEPTH: usize = 32;

/// Walk a dot-delimited path through nested TOML tables, returning a reference to the leaf value.
pub fn resolve_dot_path<'a>(root: &'a Value, path: &str) -> Option<&'a Value> {
  path
    .split('.')
    .try_fold(root, |current, seg| current.as_table()?.get(seg))
}

/// Print a TOML value to stdout in a human-friendly format.
///
/// Scalars are printed as plain text; arrays and tables are pretty-printed as JSON.
pub fn print_toml_value(value: &Value) {
  match value {
    Value::String(s) => println!("{s}"),
    Value::Boolean(b) => println!("{b}"),
    Value::Integer(n) => println!("{n}"),
    Value::Float(n) => println!("{n}"),
    Value::Datetime(dt) => println!("{dt}"),
    Value::Array(arr) => {
      let json = serde_json::to_string_pretty(arr).unwrap_or_else(|_| format!("{arr:?}"));
      println!("{json}");
    }
    Value::Table(t) => {
      let json = serde_json::to_string_pretty(t).unwrap_or_else(|_| format!("{t:?}"));
      println!("{json}");
    }
  }
}

/// Parse a string into the most specific TOML scalar type (int, float, bool, or string).
pub fn parse_toml_value(s: &str) -> Value {
  if let Ok(n) = s.parse::<i64>() {
    return Value::Integer(n);
  }
  if let Ok(n) = s.parse::<f64>() {
    return Value::Float(n);
  }
  match s {
    "true" => Value::Boolean(true),
    "false" => Value::Boolean(false),
    _ => Value::String(s.to_string()),
  }
}

/// Insert a value at a dot-delimited path, creating intermediate tables as needed.
pub fn set_dot_path(table: &mut Table, path: &str, value: &str) -> super::Result<()> {
  let segments: Vec<&str> = path.split('.').collect();

  if segments.len() > MAX_DEPTH {
    return Err(super::Error::generic(format!(
      "key path exceeds maximum depth of {MAX_DEPTH} segments"
    )));
  }

  let toml_value = parse_toml_value(value);

  if segments.len() == 1 {
    table.insert(segments[0].to_string(), toml_value);
    return Ok(());
  }

  set_nested(table, &segments, toml_value);
  Ok(())
}

/// Recursively descend into (or create) nested tables and insert the value at the final segment.
pub(crate) fn set_nested(table: &mut Table, segments: &[&str], value: Value) {
  let Some((&first, rest)) = segments.split_first() else {
    return;
  };
  let key = first.to_string();

  if rest.is_empty() {
    table.insert(key, value);
    return;
  }

  let nested = table.entry(&key).or_insert_with(|| Value::Table(Table::new()));

  if let Value::Table(t) = nested {
    set_nested(t, rest, value);
  } else {
    let mut new_table = Table::new();
    set_nested(&mut new_table, rest, value);
    table.insert(key, Value::Table(new_table));
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  mod resolve_dot_path {
    use pretty_assertions::assert_eq;

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

  mod parse_toml_value {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_falls_back_to_string() {
      assert_eq!(parse_toml_value("hello"), Value::String("hello".to_string()));
    }

    #[test]
    fn it_parses_booleans() {
      assert_eq!(parse_toml_value("true"), Value::Boolean(true));
      assert_eq!(parse_toml_value("false"), Value::Boolean(false));
    }

    #[test]
    fn it_parses_floats() {
      assert_eq!(parse_toml_value("3.14"), Value::Float(3.14));
    }

    #[test]
    fn it_parses_integers() {
      assert_eq!(parse_toml_value("42"), Value::Integer(42));
    }
  }
}
