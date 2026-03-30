//! Shared TOML metadata helpers for reading and writing dot-delimited key paths.

/// Walk a dot-delimited path through nested TOML tables, returning the leaf value.
pub fn resolve_dot_path(root: &toml::Value, path: &str) -> Option<toml::Value> {
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

/// Print a TOML value to stdout in a human-friendly format.
///
/// Scalars are printed as plain text; arrays and tables are pretty-printed as JSON.
pub fn print_toml_value(value: &toml::Value) {
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

/// Parse a string into the most specific TOML scalar type (int, float, bool, or string).
pub fn parse_toml_value(s: &str) -> toml::Value {
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

/// Insert a value at a dot-delimited path, creating intermediate tables as needed.
pub fn set_dot_path(table: &mut toml::Table, path: &str, value: &str) {
  let segments: Vec<&str> = path.split('.').collect();
  let toml_value = parse_toml_value(value);

  if segments.len() == 1 {
    table.insert(segments[0].to_string(), toml_value);
    return;
  }

  set_nested(table, &segments, toml_value);
}

/// Recursively descend into (or create) nested tables and insert the value at the final segment.
pub fn set_nested(table: &mut toml::Table, segments: &[&str], value: toml::Value) {
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

  mod resolve_dot_path_tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_resolves_nested_key() {
      let mut inner = toml::Table::new();
      inner.insert("nested".to_string(), toml::Value::String("deep".to_string()));
      let mut table = toml::Table::new();
      table.insert("outer".to_string(), toml::Value::Table(inner));
      let result = resolve_dot_path(&toml::Value::Table(table), "outer.nested");
      assert_eq!(result, Some(toml::Value::String("deep".to_string())));
    }

    #[test]
    fn it_resolves_top_level_key() {
      let mut table = toml::Table::new();
      table.insert("key".to_string(), toml::Value::String("value".to_string()));
      let result = resolve_dot_path(&toml::Value::Table(table), "key");
      assert_eq!(result, Some(toml::Value::String("value".to_string())));
    }

    #[test]
    fn it_returns_none_for_missing_key() {
      let table = toml::Table::new();
      let result = resolve_dot_path(&toml::Value::Table(table), "missing");
      assert_eq!(result, None);
    }
  }

  mod parse_toml_value_tests {
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
  }
}
