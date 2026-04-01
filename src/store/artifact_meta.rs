//! Shared YAML metadata helpers for reading and writing dot-delimited key paths on artifacts.

use yaml_serde::{Mapping, Number, Value};

/// Maximum number of dot-delimited segments allowed in a key path.
const MAX_DEPTH: usize = 32;

/// Walk a dot-delimited path through nested YAML mappings, returning a reference to the leaf value.
pub fn resolve_dot_path<'a>(root: &'a Value, path: &str) -> Option<&'a Value> {
  path
    .split('.')
    .try_fold(root, |current, seg| current.as_mapping()?.get(seg))
}

/// Print a YAML value to stdout in a human-friendly format.
///
/// Scalars are printed as plain text; sequences and mappings are pretty-printed as JSON.
pub fn print_yaml_value(value: &Value) {
  match value {
    Value::String(s) => println!("{s}"),
    Value::Bool(b) => println!("{b}"),
    Value::Number(n) => println!("{n}"),
    Value::Null => println!("null"),
    Value::Sequence(seq) => {
      let json = serde_json::to_string_pretty(seq).unwrap_or_else(|_| format!("{seq:?}"));
      println!("{json}");
    }
    Value::Mapping(m) => {
      let json = serde_json::to_string_pretty(m).unwrap_or_else(|_| format!("{m:?}"));
      println!("{json}");
    }
    Value::Tagged(t) => {
      print_yaml_value(&t.value);
    }
  }
}

/// Parse a string into the most specific YAML scalar type (integer, float, bool, null, or string).
pub fn parse_yaml_value(s: &str) -> Value {
  if let Ok(n) = s.parse::<i64>() {
    return Value::Number(Number::from(n));
  }
  if let Ok(n) = s.parse::<f64>() {
    return Value::Number(Number::from(n));
  }
  match s {
    "true" => Value::Bool(true),
    "false" => Value::Bool(false),
    "null" => Value::Null,
    _ => Value::String(s.to_string()),
  }
}

/// Set a value in a YAML mapping at the given dot-delimited path, creating intermediate mappings as needed.
pub fn set_dot_path(mapping: &mut Mapping, path: &str, value: &str) -> super::Result<()> {
  let segments: Vec<&str> = path.split('.').collect();

  if segments.len() > MAX_DEPTH {
    return Err(super::Error::generic(format!(
      "key path exceeds maximum depth of {MAX_DEPTH} segments"
    )));
  }

  let yaml_value = parse_yaml_value(value);

  if segments.len() == 1 {
    mapping.insert(Value::String(segments[0].to_string()), yaml_value);
    return Ok(());
  }

  set_nested(mapping, &segments, yaml_value);
  Ok(())
}

/// Recursively insert a value into nested YAML mappings along the given path segments.
pub(crate) fn set_nested(mapping: &mut Mapping, segments: &[&str], value: Value) {
  let Some((&first, rest)) = segments.split_first() else {
    return;
  };
  let key = Value::String(first.to_string());

  if rest.is_empty() {
    mapping.insert(key, value);
    return;
  }

  let nested = mapping
    .entry(key.clone())
    .or_insert_with(|| Value::Mapping(Mapping::new()));

  if let Value::Mapping(m) = nested {
    set_nested(m, rest, value);
  } else {
    let mut new_mapping = Mapping::new();
    set_nested(&mut new_mapping, rest, value);
    mapping.insert(key, Value::Mapping(new_mapping));
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

  mod parse_yaml_value {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_falls_back_to_string() {
      assert_eq!(parse_yaml_value("hello"), Value::String("hello".to_string()));
    }

    #[test]
    fn it_parses_booleans() {
      assert_eq!(parse_yaml_value("true"), Value::Bool(true));
      assert_eq!(parse_yaml_value("false"), Value::Bool(false));
    }

    #[test]
    fn it_parses_integers() {
      assert_eq!(parse_yaml_value("42"), Value::Number(Number::from(42)));
    }
  }
}
