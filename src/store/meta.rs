//! TOML metadata helpers for reading and writing dot-delimited key paths.
//!
//! This module re-exports generic dot-path operations specialized for TOML values.

use toml::Value;

use super::meta_value::MetaValue;
pub use super::meta_value::{resolve_dot_path, set_dot_path};

/// Parse a string into a typed TOML value (integer, float, boolean, or string fallback).
pub fn parse_toml_value(s: &str) -> Value {
  <Value as MetaValue>::parse_scalar(s)
}

/// Print a TOML value to stdout in a human-friendly format.
///
/// Scalars are printed as plain text; arrays and tables are pretty-printed as JSON.
pub fn print_toml_value(value: &Value) {
  super::meta_value::MetaValue::print(value);
}

#[cfg(test)]
mod tests {
  use super::*;

  mod resolve_dot_path {
    use pretty_assertions::assert_eq;
    use toml::value::Table;

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
    use toml::Value;

    use crate::store::meta_value::MetaValue;

    #[test]
    fn it_falls_back_to_string() {
      assert_eq!(
        <Value as MetaValue>::parse_scalar("hello"),
        Value::String("hello".to_string())
      );
    }

    #[test]
    fn it_parses_booleans() {
      assert_eq!(<Value as MetaValue>::parse_scalar("true"), Value::Boolean(true));
      assert_eq!(<Value as MetaValue>::parse_scalar("false"), Value::Boolean(false));
    }

    #[test]
    fn it_parses_floats() {
      assert_eq!(<Value as MetaValue>::parse_scalar("3.14"), Value::Float(3.14));
    }

    #[test]
    fn it_parses_integers() {
      assert_eq!(<Value as MetaValue>::parse_scalar("42"), Value::Integer(42));
    }
  }
}
