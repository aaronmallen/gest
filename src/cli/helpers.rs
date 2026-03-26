/// Split a comma-separated string into a `Vec<String>`, trimming whitespace and filtering empty strings.
pub fn parse_tags(s: &str) -> Vec<String> {
  s.split(',')
    .map(|s| s.trim().to_string())
    .filter(|s| !s.is_empty())
    .collect()
}

/// Parse a slice of "key=value" strings into a `Vec` of `(String, String)` pairs.
///
/// Returns an error if any entry does not contain an `=` separator.
pub fn split_key_value_pairs(pairs: &[String]) -> crate::Result<Vec<(String, String)>> {
  pairs
    .iter()
    .map(|pair| {
      pair
        .split_once('=')
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .ok_or_else(|| crate::Error::generic(format!("Invalid metadata format '{pair}', expected key=value")))
    })
    .collect()
}

#[cfg(test)]
mod tests {
  use super::*;

  mod parse_tags_tests {
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

  mod split_key_value_pairs_tests {
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
