use std::{fmt::Display, io::IsTerminal, str::FromStr};

/// Build a `toml::Table` from `"key=value"` CLI strings.
pub fn build_toml_metadata(pairs: &[String]) -> crate::cli::Result<toml::Table> {
  let kvs = split_key_value_pairs(pairs)?;
  let mut table = toml::Table::new();
  for (key, value) in kvs {
    table.insert(key, crate::store::meta::parse_toml_value(&value));
  }
  Ok(table)
}

/// Build a `yaml_serde::Mapping` from `"key=value"` CLI strings.
pub fn build_yaml_metadata(pairs: &[String]) -> crate::cli::Result<yaml_serde::Mapping> {
  let kvs = split_key_value_pairs(pairs)?;
  let mut mapping = yaml_serde::Mapping::new();
  for (key, value) in kvs {
    mapping.insert(yaml_serde::Value::String(key), yaml_serde::Value::String(value));
  }
  Ok(mapping)
}

/// Merge `"key=value"` CLI strings into an existing `toml::Table`, returning `None` if pairs is empty.
pub fn merge_toml_metadata(pairs: &[String], mut existing: toml::Table) -> crate::cli::Result<Option<toml::Table>> {
  if pairs.is_empty() {
    return Ok(None);
  }
  let kvs = split_key_value_pairs(pairs)?;
  for (key, value) in kvs {
    existing.insert(key, crate::store::meta::parse_toml_value(&value));
  }
  Ok(Some(existing))
}

/// Parse an optional status string into a typed status value.
///
/// Returns `Ok(None)` when the input is `None`, `Ok(Some(status))` on a valid parse,
/// or a `cli::Error` when the string is not a recognised status.
pub fn parse_optional_status<T>(raw: Option<&str>) -> crate::cli::Result<Option<T>>
where
  T: FromStr,
  T::Err: Display,
{
  raw
    .map(|s| s.parse::<T>().map_err(|e| crate::cli::Error::generic(e.to_string())))
    .transpose()
}

/// Split a comma-separated string into trimmed, non-empty tag strings.
pub fn parse_tags(s: &str) -> Vec<String> {
  s.split(',')
    .map(|s| s.trim().to_string())
    .filter(|s| !s.is_empty())
    .collect()
}

/// Open `$EDITOR` with a temporary file and return the content, or fall back to an empty string.
///
/// If `explicit` is `Some`, that value is returned immediately (no editor opened).
/// Otherwise, when stdin is a terminal and `$EDITOR` is set, a temp file is opened.
/// Returns an error with `abort_message` if the user saves an empty file.
/// Falls back to an empty string when stdin is not a terminal or no editor is configured.
pub fn read_from_editor(
  explicit: Option<&str>,
  file_extension: &str,
  abort_message: &str,
) -> crate::cli::Result<String> {
  if let Some(value) = explicit {
    return Ok(value.to_string());
  }

  if std::io::stdin().is_terminal()
    && let Some(_editor) = crate::cli::editor::resolve_editor()
  {
    let content = crate::cli::editor::edit_temp(None, file_extension)?;
    if content.trim().is_empty() {
      return Err(crate::cli::Error::generic(abort_message));
    }
    return Ok(content);
  }

  Ok(String::new())
}

/// Parse `"key=value"` strings into `(key, value)` pairs, returning an error on missing `=`.
pub fn split_key_value_pairs(pairs: &[String]) -> crate::cli::Result<Vec<(String, String)>> {
  pairs
    .iter()
    .map(|pair| {
      pair
        .split_once('=')
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .ok_or_else(|| crate::cli::Error::generic(format!("Invalid metadata format '{pair}', expected key=value")))
    })
    .collect()
}

#[cfg(test)]
mod tests {
  use super::*;

  mod parse_tags {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_filters_empty_strings() {
      assert_eq!(parse_tags("rust,,cli,,,test"), vec!["rust", "cli", "test"]);
    }

    #[test]
    fn it_handles_single_tag() {
      assert_eq!(parse_tags("rust"), vec!["rust"]);
    }

    #[test]
    fn it_returns_empty_vec_for_empty_string() {
      let result: Vec<String> = parse_tags("");
      assert!(result.is_empty());
    }

    #[test]
    fn it_returns_empty_vec_for_only_commas() {
      let result: Vec<String> = parse_tags(",,,");
      assert!(result.is_empty());
    }

    #[test]
    fn it_splits_comma_separated_tags() {
      assert_eq!(parse_tags("rust,cli,test"), vec!["rust", "cli", "test"]);
    }

    #[test]
    fn it_trims_whitespace() {
      assert_eq!(parse_tags(" rust , cli , test "), vec!["rust", "cli", "test"]);
    }
  }

  mod split_key_value_pairs {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_errors_on_missing_equals() {
      let pairs = vec!["invalid".to_string()];
      let result = split_key_value_pairs(&pairs);

      assert!(result.is_err());
    }

    #[test]
    fn it_handles_empty_list() {
      let pairs: Vec<String> = vec![];
      let result = split_key_value_pairs(&pairs).unwrap();

      assert!(result.is_empty());
    }

    #[test]
    fn it_handles_empty_value() {
      let pairs = vec!["key=".to_string()];
      let result = split_key_value_pairs(&pairs).unwrap();

      assert_eq!(result, vec![("key".to_string(), String::new())]);
    }

    #[test]
    fn it_handles_value_with_equals_sign() {
      let pairs = vec!["key=val=ue".to_string()];
      let result = split_key_value_pairs(&pairs).unwrap();

      assert_eq!(result, vec![("key".to_string(), "val=ue".to_string())]);
    }

    #[test]
    fn it_parses_key_value_pairs() {
      let pairs = vec!["foo=bar".to_string(), "baz=qux".to_string()];
      let result = split_key_value_pairs(&pairs).unwrap();

      assert_eq!(
        result,
        vec![
          ("foo".to_string(), "bar".to_string()),
          ("baz".to_string(), "qux".to_string()),
        ]
      );
    }
  }
}
