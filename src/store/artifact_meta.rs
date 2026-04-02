//! YAML metadata helpers for reading and writing dot-delimited key paths on artifacts.
//!
//! This module re-exports generic dot-path operations specialized for YAML values.

use yaml_serde::Value;

pub use super::meta_value::{resolve_dot_path, set_dot_path};

/// Print a YAML value to stdout in a human-friendly format.
///
/// Scalars are printed as plain text; sequences and mappings are pretty-printed as JSON.
pub fn print_yaml_value(value: &Value) {
  super::meta_value::MetaValue::print(value);
}

#[cfg(test)]
mod tests {
  use super::*;

  mod resolve_dot_path {
    use pretty_assertions::assert_eq;
    use yaml_serde::Mapping;

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
    use yaml_serde::{Number, Value};

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
      assert_eq!(<Value as MetaValue>::parse_scalar("true"), Value::Bool(true));
      assert_eq!(<Value as MetaValue>::parse_scalar("false"), Value::Bool(false));
    }

    #[test]
    fn it_parses_integers() {
      assert_eq!(
        <Value as MetaValue>::parse_scalar("42"),
        Value::Number(Number::from(42))
      );
    }
  }
}
