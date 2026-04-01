use std::{
  fs,
  path::{Path, PathBuf},
};

use super::Error;
use crate::{config::Settings, model::Id};

/// Maximum number of ID generation attempts before giving up.
const MAX_ID_ATTEMPTS: usize = 100;

/// Result of scanning a directory for file stems matching a given prefix.
///
/// Only the first two matches are retained—enough to distinguish between
/// "not found", "unique", and "ambiguous".
enum PrefixMatch {
  /// Two or more files matched (carries the first two for diagnostics).
  Ambiguous(String, String),
  /// No files matched the prefix.
  None,
  /// Exactly one file matched.
  Unique(String),
}

/// Create all required store subdirectories under the layout's entity dirs.
pub fn ensure_dirs(config: &Settings) -> super::Result<()> {
  let storage = config.storage();
  fs::create_dir_all(storage.artifact_dir())?;
  fs::create_dir_all(storage.artifact_dir().join("archive"))?;
  fs::create_dir_all(storage.iteration_dir())?;
  fs::create_dir_all(storage.iteration_dir().join("resolved"))?;
  fs::create_dir_all(storage.state_dir())?;
  fs::create_dir_all(storage.task_dir())?;
  fs::create_dir_all(storage.task_dir().join("resolved"))?;
  Ok(())
}

/// Write `content` to `dest` and remove `src` if it exists, ensuring store dirs first.
pub(crate) fn move_entity_file(config: &Settings, content: &str, dest: &Path, src: &Path) -> super::Result<()> {
  ensure_dirs(config)?;
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
  Id::validate_prefix(prefix).map_err(Error::generic)?;

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

/// Generate a new [`Id`] that does not collide (by short prefix) with any
/// existing entity across artifacts, tasks, and iterations.
pub(crate) fn next_id(config: &Settings) -> super::Result<Id> {
  let storage = config.storage();

  let artifact_archive = storage.artifact_dir().join("archive");
  let task_resolved = storage.task_dir().join("resolved");
  let iteration_resolved = storage.iteration_dir().join("resolved");

  let dirs: &[(&Path, &str)] = &[
    (storage.artifact_dir(), "md"),
    (&artifact_archive, "md"),
    (storage.task_dir(), "toml"),
    (&task_resolved, "toml"),
    (storage.iteration_dir(), "toml"),
    (&iteration_resolved, "toml"),
  ];

  for _ in 0..MAX_ID_ATTEMPTS {
    let id = Id::new();
    let short = id.short();

    let mut collision = false;
    for &(dir, ext) in dirs {
      if has_prefix_match(dir, ext, &short)? {
        collision = true;
        break;
      }
    }

    if !collision {
      return Ok(id);
    }
  }

  Err(super::Error::generic(format!(
    "failed to generate a unique ID after {MAX_ID_ATTEMPTS} attempts"
  )))
}

/// Check whether any file in `dir` with the given extension has a stem starting with `prefix`.
fn has_prefix_match(dir: &Path, extension: &str, prefix: &str) -> super::Result<bool> {
  for path in read_dir_files(dir, extension)? {
    if let Some(stem) = path.file_stem().and_then(|s| s.to_str())
      && stem.starts_with(prefix)
    {
      return Ok(true);
    }
  }
  Ok(false)
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

#[cfg(test)]
mod tests {
  mod ensure_dirs {
    use crate::config::Settings;

    fn make_config(base: &std::path::Path) -> Settings {
      crate::test_helpers::make_test_config(base.to_path_buf())
    }

    #[test]
    fn it_creates_all_subdirectories() {
      let dir = tempfile::tempdir().unwrap();
      let config = make_config(dir.path());
      crate::store::ensure_dirs(&config).unwrap();

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
      let config = make_config(dir.path());
      crate::store::ensure_dirs(&config).unwrap();
      crate::store::ensure_dirs(&config).unwrap();

      assert!(dir.path().join("tasks").is_dir());
    }
  }

  mod next_id {
    use crate::config::Settings;

    fn make_config(base: &std::path::Path) -> Settings {
      crate::test_helpers::make_test_config(base.to_path_buf())
    }

    #[test]
    fn it_avoids_short_prefix_collisions_across_entity_types() {
      let dir = tempfile::tempdir().unwrap();
      let config = make_config(dir.path());
      crate::store::ensure_dirs(&config).unwrap();

      // Create a task
      let task = crate::test_helpers::make_test_task("zyxwvutsrqponmlkzyxwvutsrqponmlk");
      crate::store::write_task(&config, &task).unwrap();

      // Generate many IDs — none should share the task's 8-char short prefix
      let task_short = task.id.short();
      for _ in 0..50 {
        let id = super::super::next_id(&config).unwrap();
        assert_ne!(
          id.short(),
          task_short,
          "generated ID {id} collides with existing task short prefix {task_short}"
        );
      }
    }

    #[test]
    fn it_checks_artifact_directories_too() {
      let dir = tempfile::tempdir().unwrap();
      let config = make_config(dir.path());
      crate::store::ensure_dirs(&config).unwrap();

      // Create an artifact
      let artifact = crate::test_helpers::make_test_artifact("zyxwvutsrqponmlkzyxwvutsrqponmlk");
      crate::store::write_artifact(&config, &artifact).unwrap();

      // No generated ID should share the artifact's short prefix
      let artifact_short = artifact.id.short();
      for _ in 0..50 {
        let id = super::super::next_id(&config).unwrap();
        assert_ne!(id.short(), artifact_short);
      }
    }

    #[test]
    fn it_generates_a_unique_id() {
      let dir = tempfile::tempdir().unwrap();
      let config = make_config(dir.path());
      crate::store::ensure_dirs(&config).unwrap();

      let id = super::super::next_id(&config).unwrap();
      assert_eq!(id.to_string().len(), 32);
    }
  }
}
