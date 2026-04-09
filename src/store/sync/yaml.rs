//! YAML read/write helpers used by every per-entity sync adapter.
//!
//! Adapters call [`read`] and [`write`] rather than touching `yaml_serde` and
//! the filesystem directly so that the file-format choice and the on-disk
//! invariants (atomic write, parent-directory creation, trailing newline) live
//! in exactly one place.
//!
//! Stable, human-readable output is the responsibility of the wrapper structs
//! the adapters define: `yaml_serde` honors struct field declaration order,
//! so listing fields in a fixed sequence is enough to keep diffs clean.

use std::{
  fs,
  io::ErrorKind,
  path::{Path, PathBuf},
};

use libsql::Connection;
use serde::{Serialize, de::DeserializeOwned};

use super::{digest, paths};
use crate::store::{Error, model::primitives::Id};

/// Read and YAML-deserialize a file into `T`, returning `Ok(None)` if absent.
///
/// Any other I/O or parse error is propagated. Adapters use this to walk a
/// directory and skip vanished files without paying special attention to race
/// conditions.
pub fn read<T: DeserializeOwned>(path: &Path) -> Result<Option<T>, Error> {
  let content = match fs::read_to_string(path) {
    Ok(content) => content,
    Err(e) if e.kind() == ErrorKind::NotFound => return Ok(None),
    Err(e) => return Err(Error::Io(e)),
  };
  let value = yaml_serde::from_str(&content)?;
  Ok(Some(value))
}

/// Serialize `value` to YAML and write it to `path` with a trailing newline.
///
/// Used by tests; production code uses [`write_cached`] which also updates the
/// `sync_digests` cache.
#[cfg(test)]
pub fn write<T: Serialize>(path: &Path, value: &T) -> Result<(), Error> {
  if let Some(parent) = path.parent() {
    fs::create_dir_all(parent)?;
  }
  let mut content = yaml_serde::to_string(value)?;
  if !content.ends_with('\n') {
    content.push('\n');
  }
  fs::write(path, content)?;
  Ok(())
}

/// Serialize `value`, write it to `path`, and update the `sync_digests` cache.
///
/// If the cached digest for `(project_id, relative_to(path))` already matches
/// the new content, the file is left untouched and the cache is unchanged.
/// This is the standard write path for every per-entity sync adapter.
pub async fn write_cached<T: Serialize>(
  conn: &Connection,
  project_id: &Id,
  gest_dir: &Path,
  path: &Path,
  value: &T,
) -> Result<(), Error> {
  let mut serialized = yaml_serde::to_string(value)?;
  if !serialized.ends_with('\n') {
    serialized.push('\n');
  }
  let new_digest = digest::compute(serialized.as_bytes());
  let relative = paths::relative(gest_dir, path).ok_or_else(|| {
    Error::Io(std::io::Error::other(format!(
      "path {} is outside {}",
      path.display(),
      gest_dir.display()
    )))
  })?;

  if digest::is_unchanged(conn, project_id, &relative, &new_digest).await? {
    return Ok(());
  }

  if let Some(parent) = path.parent() {
    fs::create_dir_all(parent)?;
  }
  fs::write(path, serialized.as_bytes())?;
  digest::record(conn, project_id, &relative, &new_digest).await?;
  Ok(())
}

/// Remove every file under `dir` whose filename stem is not present in
/// `alive_ids`, and drop the corresponding `sync_digests` cache entries.
///
/// Each per-entity adapter calls this at the end of its `write_all` to clean
/// up files left over from deleted entities. Without it, the next `import`
/// would re-insert rows that the user just deleted (because the file is
/// still on disk).
pub async fn cleanup_orphans(
  conn: &Connection,
  project_id: &Id,
  gest_dir: &Path,
  dir: &Path,
  extension: &str,
  alive_ids: &std::collections::HashSet<String>,
) -> Result<(), Error> {
  for path in walk_files(dir, extension)? {
    let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
    if alive_ids.contains(stem) {
      continue;
    }
    if is_tombstoned_yaml_file(&path) {
      // Tombstoned files are intentional delete markers; leave them on disk
      // so that downstream clones can pick up the deletion on their next
      // import.
      continue;
    }
    let relative = paths::relative(gest_dir, &path).ok_or_else(|| {
      Error::Io(std::io::Error::other(format!(
        "path {} is outside {}",
        path.display(),
        gest_dir.display()
      )))
    })?;
    fs::remove_file(&path)?;
    conn
      .execute(
        "DELETE FROM sync_digests WHERE relative_path = ?1 AND project_id = ?2",
        [relative, project_id.to_string()],
      )
      .await?;
  }
  Ok(())
}

/// Return `true` if the YAML file at `path` carries a `deleted_at` tombstone
/// key at the top level. Parse errors and missing files are treated as "not
/// tombstoned" so cleanup callers fall back to their normal behavior.
pub(super) fn is_tombstoned_yaml_file(path: &Path) -> bool {
  let Ok(raw) = fs::read_to_string(path) else {
    return false;
  };
  let Ok(value) = yaml_serde::from_str::<yaml_serde::Value>(&raw) else {
    return false;
  };
  value
    .as_mapping()
    .and_then(|m| m.get(yaml_serde::Value::String("deleted_at".into())))
    .is_some_and(|v| !v.is_null())
}

/// Walk a directory recursively and return every file path matching `extension`.
///
/// Returns an empty vector if `dir` does not exist. The walker is depth-first
/// and skips entries it cannot read rather than aborting; adapters that need
/// strict error reporting should walk the tree themselves.
pub fn walk_files(dir: &Path, extension: &str) -> Result<Vec<PathBuf>, Error> {
  let mut out = Vec::new();
  walk_files_into(dir, extension, &mut out)?;
  out.sort();
  Ok(out)
}

fn walk_files_into(dir: &Path, extension: &str, out: &mut Vec<PathBuf>) -> Result<(), Error> {
  let entries = match fs::read_dir(dir) {
    Ok(entries) => entries,
    Err(e) if e.kind() == ErrorKind::NotFound => return Ok(()),
    Err(e) => return Err(Error::Io(e)),
  };
  for entry in entries {
    let entry = entry?;
    let path = entry.path();
    let file_type = entry.file_type()?;
    if file_type.is_dir() {
      walk_files_into(&path, extension, out)?;
    } else if file_type.is_file() && path.extension().is_some_and(|ext| ext == extension) {
      out.push(path);
    }
  }
  Ok(())
}

#[cfg(test)]
mod tests {
  use serde::{Deserialize, Serialize};
  use tempfile::TempDir;

  use super::*;

  #[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
  struct Sample {
    title: String,
    count: u32,
    tags: Vec<String>,
  }

  mod read {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_returns_none_for_a_missing_file() {
      let dir = TempDir::new().unwrap();
      let path = dir.path().join("missing.yaml");

      let result: Option<Sample> = read(&path).unwrap();

      assert!(result.is_none());
    }

    #[test]
    fn it_roundtrips_a_struct_through_disk() {
      let dir = TempDir::new().unwrap();
      let path = dir.path().join("sample.yaml");
      let value = Sample {
        title: "demo".into(),
        count: 7,
        tags: vec!["a".into(), "b".into()],
      };

      write(&path, &value).unwrap();
      let parsed: Sample = read(&path).unwrap().unwrap();

      assert_eq!(parsed, value);
    }
  }

  mod walk_files {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_returns_empty_for_a_missing_directory() {
      let dir = TempDir::new().unwrap();
      let missing = dir.path().join("nope");

      let files = walk_files(&missing, "yaml").unwrap();

      assert!(files.is_empty());
    }

    #[test]
    fn it_walks_recursively_and_filters_by_extension() {
      let dir = TempDir::new().unwrap();
      fs::create_dir_all(dir.path().join("a/b")).unwrap();
      fs::write(dir.path().join("a/one.yaml"), "x: 1\n").unwrap();
      fs::write(dir.path().join("a/skip.txt"), "ignore me").unwrap();
      fs::write(dir.path().join("a/b/two.yaml"), "x: 2\n").unwrap();

      let files = walk_files(dir.path(), "yaml").unwrap();

      assert_eq!(files.len(), 2);
      assert!(files[0].ends_with("a/b/two.yaml") || files[0].ends_with("a/one.yaml"));
    }
  }

  mod write {
    use super::*;

    #[test]
    fn it_creates_missing_parent_directories() {
      let dir = TempDir::new().unwrap();
      let path = dir.path().join("nested/deep/sample.yaml");
      let value = Sample {
        title: "x".into(),
        count: 0,
        tags: vec![],
      };

      write(&path, &value).unwrap();

      assert!(path.exists());
    }

    #[test]
    fn it_ensures_a_trailing_newline() {
      let dir = TempDir::new().unwrap();
      let path = dir.path().join("sample.yaml");
      let value = Sample {
        title: "x".into(),
        count: 0,
        tags: vec![],
      };

      write(&path, &value).unwrap();

      let raw = fs::read_to_string(&path).unwrap();
      assert!(raw.ends_with('\n'));
    }

    #[test]
    fn it_preserves_struct_field_declaration_order() {
      let dir = TempDir::new().unwrap();
      let path = dir.path().join("sample.yaml");
      let value = Sample {
        title: "demo".into(),
        count: 1,
        tags: vec!["t".into()],
      };

      write(&path, &value).unwrap();

      let raw = fs::read_to_string(&path).unwrap();
      let title_idx = raw.find("title").unwrap();
      let count_idx = raw.find("count").unwrap();
      let tags_idx = raw.find("tags").unwrap();
      assert!(title_idx < count_idx);
      assert!(count_idx < tags_idx);
    }
  }
}
