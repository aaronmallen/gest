//! Auto-discovery of legacy v0.4 `.gest/` data directories.

use std::{
  fmt::Write,
  io::{Error, ErrorKind},
  path::{Path, PathBuf},
};

use sha2::{Digest, Sha256};

/// Marker subdirectories that indicate a v0.4 legacy `.gest/` directory.
const LEGACY_SUBDIRS: &[&str] = &["tasks", "artifacts", "iterations"];

/// Discover a legacy v0.4 data directory starting from `cwd`.
///
/// Resolution order:
/// 1. `cwd/.gest/` if it contains flat-file subdirs
/// 2. Walk ancestors of `cwd` looking for `.gest/`
/// 3. Global hash: `$XDG_DATA_HOME/gest/<sha256(canonicalize(cwd))[..8]>/`
pub fn find_legacy_dir(cwd: &Path) -> Result<PathBuf, Error> {
  // 1. Local
  let local = cwd.join(".gest");
  if is_legacy_dir(&local) {
    return Ok(local);
  }

  // 2. Ancestor walk
  if let Some(found) = walk_ancestors(cwd) {
    return Ok(found);
  }

  // 3. Global hash
  if let Some(found) = global_hash_dir(cwd) {
    return Ok(found);
  }

  Err(Error::new(
    ErrorKind::NotFound,
    format!(
      "no legacy .gest/ directory found from {} (checked local, ancestors, and global data dir)",
      cwd.display()
    ),
  ))
}

/// Check whether a directory looks like a v0.4 legacy `.gest/` dir.
fn is_legacy_dir(path: &Path) -> bool {
  path.is_dir() && LEGACY_SUBDIRS.iter().any(|sub| path.join(sub).is_dir())
}

/// Walk up from `cwd` checking each parent for `.gest/` with legacy subdirs.
fn walk_ancestors(cwd: &Path) -> Option<PathBuf> {
  let mut current = cwd.to_path_buf();
  loop {
    if !current.pop() {
      return None;
    }
    let candidate = current.join(".gest");
    if is_legacy_dir(&candidate) {
      return Some(candidate);
    }
  }
}

/// Check the global XDG data directory for a hash-based project dir.
fn global_hash_dir(cwd: &Path) -> Option<PathBuf> {
  let data_home = dir_spec::data_home()?;
  let hash = path_hash(cwd);
  let candidate = data_home.join("gest").join(hash);
  if is_legacy_dir(&candidate) {
    Some(candidate)
  } else {
    None
  }
}

/// Produce a short hex hash of the canonicalized path, matching v0.4.x behavior.
fn path_hash(path: &Path) -> String {
  let canonical = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
  let mut hasher = Sha256::new();
  hasher.update(canonical.as_os_str().as_encoded_bytes());
  let result = hasher.finalize();
  let mut hash = String::with_capacity(16);
  for b in &result[..8] {
    write!(hash, "{b:02x}").expect("writing to String is infallible");
  }
  hash
}

#[cfg(test)]
mod tests {
  use super::*;

  mod find_legacy_dir {
    use super::*;

    #[test]
    fn it_finds_local_gest_dir() {
      let tmp = tempfile::tempdir().unwrap();
      let gest = tmp.path().join(".gest");
      std::fs::create_dir_all(gest.join("tasks")).unwrap();

      let result = find_legacy_dir(tmp.path()).unwrap();
      assert_eq!(result, gest);
    }

    #[test]
    fn it_walks_ancestors() {
      let tmp = tempfile::tempdir().unwrap();
      let gest = tmp.path().join(".gest");
      std::fs::create_dir_all(gest.join("artifacts")).unwrap();
      let child = tmp.path().join("sub/dir");
      std::fs::create_dir_all(&child).unwrap();

      let result = find_legacy_dir(&child).unwrap();
      assert_eq!(result, gest);
    }

    #[test]
    fn it_returns_error_when_not_found() {
      let tmp = tempfile::tempdir().unwrap();
      let result = find_legacy_dir(tmp.path());
      assert!(result.is_err());
    }

    #[test]
    fn it_ignores_gest_dir_without_legacy_subdirs() {
      let tmp = tempfile::tempdir().unwrap();
      let gest = tmp.path().join(".gest");
      std::fs::create_dir_all(&gest).unwrap();

      let result = find_legacy_dir(tmp.path());
      assert!(result.is_err());
    }
  }

  mod path_hash {
    use super::*;

    #[test]
    fn it_produces_16_char_hex_string() {
      let hash = path_hash(Path::new("/tmp/test"));
      assert_eq!(hash.len(), 16);
      assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn it_is_deterministic() {
      let a = path_hash(Path::new("/some/path"));
      let b = path_hash(Path::new("/some/path"));
      assert_eq!(a, b);
    }
  }
}
