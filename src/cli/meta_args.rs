//! Shared `--metadata` / `--metadata-json` flag parsing for create/update commands.

use serde_json::{Map, Value};

use crate::{cli::Error, store::meta};

/// Build a metadata `Value::Object` from repeated `--metadata key=value` pairs and
/// `--metadata-json '<json-object>'` strings, layered on top of an optional base.
///
/// Scalar pairs are inserted first (with dot-path support and scalar inference);
/// `--metadata-json` objects are then merged shallow on top so they can override.
/// Returns `None` when no inputs are provided and no base exists.
pub fn build_metadata(base: Option<Value>, pairs: &[String], json_blobs: &[String]) -> Result<Option<Value>, Error> {
  if pairs.is_empty() && json_blobs.is_empty() {
    return Ok(base);
  }

  let mut metadata = match base {
    Some(Value::Object(map)) => Value::Object(map),
    Some(_) | None => Value::Object(Map::new()),
  };

  for pair in pairs {
    let (key, raw) = pair
      .split_once('=')
      .ok_or_else(|| Error::Editor(format!("invalid --metadata pair (expected key=value): {pair}")))?;
    let value = meta::parse_scalar(raw);
    if !meta::set_path(&mut metadata, key, value) {
      return Err(Error::Editor(format!("invalid --metadata key path: {key}")));
    }
  }

  for blob in json_blobs {
    let parsed: Value =
      serde_json::from_str(blob).map_err(|e| Error::Editor(format!("invalid --metadata-json: {e}")))?;
    let object = match parsed {
      Value::Object(map) => map,
      _ => return Err(Error::Editor("--metadata-json must be a JSON object".to_string())),
    };
    if let Some(existing) = metadata.as_object_mut() {
      for (k, v) in object {
        existing.insert(k, v);
      }
    }
  }

  Ok(Some(metadata))
}

#[cfg(test)]
mod tests {
  use serde_json::json;

  use super::*;

  mod build_metadata {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_returns_base_when_no_inputs() {
      let base = Some(json!({"k": 1}));

      let out = build_metadata(base.clone(), &[], &[]).unwrap();

      assert_eq!(out, base);
    }

    #[test]
    fn it_merges_dot_path_pairs() {
      let pairs = vec!["a.b".to_string(), "c".to_string()];
      let pairs_kv = vec!["a.b=deep".to_string(), "c=42".to_string()];

      let out = build_metadata(None, &pairs_kv, &[]).unwrap().unwrap();

      assert_eq!(out, json!({"a": {"b": "deep"}, "c": 42}));
      let _ = pairs;
    }

    #[test]
    fn it_overlays_metadata_json_after_pairs() {
      let pairs = vec!["k=1".to_string()];
      let blobs = vec![r#"{"k": 2, "extra": true}"#.to_string()];

      let out = build_metadata(None, &pairs, &blobs).unwrap().unwrap();

      assert_eq!(out, json!({"k": 2, "extra": true}));
    }

    #[test]
    fn it_preserves_unrelated_base_keys() {
      let base = Some(json!({"keep": "yes", "k": 1}));
      let pairs = vec!["k=2".to_string()];

      let out = build_metadata(base, &pairs, &[]).unwrap().unwrap();

      assert_eq!(out, json!({"keep": "yes", "k": 2}));
    }

    #[test]
    fn it_errors_on_pair_without_equals() {
      let pairs = vec!["bad".to_string()];

      assert!(build_metadata(None, &pairs, &[]).is_err());
    }

    #[test]
    fn it_errors_on_non_object_metadata_json() {
      let blobs = vec!["[1,2,3]".to_string()];

      assert!(build_metadata(None, &[], &blobs).is_err());
    }
  }
}
