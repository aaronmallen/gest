use std::{
  fs,
  path::{Path, PathBuf},
};

use super::Error;
use crate::model::Id;

/// Create all required store subdirectories under `data_dir`.
pub fn ensure_dirs(data_dir: &Path) -> super::Result<()> {
  fs::create_dir_all(data_dir.join("artifacts"))?;
  fs::create_dir_all(data_dir.join("artifacts/archive"))?;
  fs::create_dir_all(data_dir.join("iterations"))?;
  fs::create_dir_all(data_dir.join("iterations/resolved"))?;
  fs::create_dir_all(data_dir.join("tasks"))?;
  fs::create_dir_all(data_dir.join("tasks/resolved"))?;
  Ok(())
}

/// Result of scanning a directory for file stems matching a given prefix.
///
/// Only the first two matches are retained—enough to distinguish between
/// "not found", "unique", and "ambiguous".
enum PrefixMatch {
  /// No files matched the prefix.
  None,
  /// Exactly one file matched.
  Unique(String),
  /// Two or more files matched (carries the first two for diagnostics).
  Ambiguous(String, String),
}

/// Scan `dir` for file stems starting with `prefix` (with the given extension),
/// returning as soon as two matches are found.
fn collect_prefix_matches(dir: &Path, extension: &str, prefix: &str) -> super::Result<PrefixMatch> {
  let mut first: Option<String> = None;
  for path in read_dir_files(dir, extension)? {
    if let Some(stem) = path.file_stem().and_then(|s| s.to_str())
      && stem.starts_with(prefix)
    {
      match first {
        None => first = Some(stem.to_string()),
        Some(ref f) => return Ok(PrefixMatch::Ambiguous(f.clone(), stem.to_string())),
      }
    }
  }
  Ok(match first {
    Some(s) => PrefixMatch::Unique(s),
    None => PrefixMatch::None,
  })
}

/// Write `content` to `dest` and remove `src` if it exists, ensuring store dirs first.
pub(crate) fn move_entity_file(data_dir: &Path, content: &str, dest: &Path, src: &Path) -> super::Result<()> {
  ensure_dirs(data_dir)?;
  fs::write(dest, content)?;
  if src.exists() {
    fs::remove_file(src)?;
  }
  Ok(())
}

/// List files in `dir` with the given extension, sorted by path.
pub(crate) fn read_dir_files(dir: &Path, extension: &str) -> super::Result<Vec<PathBuf>> {
  if !dir.exists() {
    return Ok(Vec::new());
  }

  let mut paths = Vec::new();
  for entry in fs::read_dir(dir)? {
    let entry = entry?;
    let path = entry.path();
    if path.is_file()
      && let Some(ext) = path.extension().and_then(|e| e.to_str())
      && ext == extension
    {
      paths.push(path);
    }
  }
  paths.sort();
  Ok(paths)
}

/// Resolve an ID prefix to a unique [`Id`], searching `primary_dir` first and
/// optionally falling back to `secondary_dir`.
pub(crate) fn resolve_id(
  primary_dir: &Path,
  secondary_dir: Option<&Path>,
  extension: &str,
  prefix: &str,
  include_secondary: bool,
  hint: &str,
) -> super::Result<Id> {
  match collect_prefix_matches(primary_dir, extension, prefix)? {
    PrefixMatch::Unique(id) => return id.parse().map_err(|e: String| Error::generic(e)),
    PrefixMatch::Ambiguous(a, b) => {
      return Err(Error::generic(format!(
        "Ambiguous ID prefix '{prefix}', matches: {a}, {b}"
      )));
    }
    PrefixMatch::None => {}
  }

  if include_secondary && let Some(secondary) = secondary_dir {
    match collect_prefix_matches(secondary, extension, prefix)? {
      PrefixMatch::Unique(id) => return id.parse().map_err(|e: String| Error::generic(e)),
      PrefixMatch::Ambiguous(a, b) => {
        return Err(Error::generic(format!(
          "Ambiguous ID prefix '{prefix}', matches: {a}, {b}"
        )));
      }
      PrefixMatch::None => {}
    }
  }

  let mut msg = format!("{hint} not found: '{prefix}'");
  if !include_secondary {
    msg.push_str(" (try --all)");
  }
  Err(Error::generic(msg))
}

#[cfg(test)]
mod tests {
  mod ensure_dirs {
    #[test]
    fn it_creates_all_subdirectories() {
      let dir = tempfile::tempdir().unwrap();
      crate::store::ensure_dirs(dir.path()).unwrap();

      assert!(dir.path().join("artifacts").is_dir());
      assert!(dir.path().join("artifacts/archive").is_dir());
      assert!(dir.path().join("iterations").is_dir());
      assert!(dir.path().join("iterations/resolved").is_dir());
      assert!(dir.path().join("tasks").is_dir());
      assert!(dir.path().join("tasks/resolved").is_dir());
    }

    #[test]
    fn it_is_idempotent() {
      let dir = tempfile::tempdir().unwrap();
      crate::store::ensure_dirs(dir.path()).unwrap();
      crate::store::ensure_dirs(dir.path()).unwrap();

      assert!(dir.path().join("tasks").is_dir());
    }
  }
}
