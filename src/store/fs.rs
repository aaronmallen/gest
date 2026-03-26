use std::{
  fs,
  path::{Path, PathBuf},
};

use crate::model::Id;

pub fn ensure_dirs(data_dir: &Path) -> crate::Result<()> {
  fs::create_dir_all(data_dir.join("artifacts"))?;
  fs::create_dir_all(data_dir.join("artifacts/archive"))?;
  fs::create_dir_all(data_dir.join("tasks"))?;
  fs::create_dir_all(data_dir.join("tasks/resolved"))?;
  Ok(())
}

pub(crate) fn resolve_id(
  primary_dir: &Path,
  secondary_dir: Option<&Path>,
  extension: &str,
  prefix: &str,
  include_secondary: bool,
  hint: &str,
) -> crate::Result<Id> {
  let active_matches = collect_prefix_matches(primary_dir, extension, prefix)?;

  match active_matches.len() {
    1 => {
      return active_matches[0].parse().map_err(|e: String| crate::Error::generic(e));
    }
    n if n > 1 => {
      let ids = active_matches.join(", ");
      return Err(crate::Error::generic(format!(
        "Ambiguous ID prefix '{prefix}', matches: {ids}"
      )));
    }
    _ => {}
  }

  if include_secondary && let Some(secondary) = secondary_dir {
    let secondary_matches = collect_prefix_matches(secondary, extension, prefix)?;
    match secondary_matches.len() {
      0 => {}
      1 => {
        return secondary_matches[0]
          .parse()
          .map_err(|e: String| crate::Error::generic(e));
      }
      _ => {
        let ids = secondary_matches.join(", ");
        return Err(crate::Error::generic(format!(
          "Ambiguous ID prefix '{prefix}', matches: {ids}"
        )));
      }
    }
  }

  let mut msg = format!("{hint} not found: '{prefix}'");
  if !include_secondary {
    if hint == "Artifact" {
      msg.push_str(" (try --include-archived)");
    } else {
      msg.push_str(" (try --all)");
    }
  }
  Err(crate::Error::generic(msg))
}

pub(crate) fn collect_prefix_matches(dir: &Path, extension: &str, prefix: &str) -> crate::Result<Vec<String>> {
  let mut matches = Vec::new();
  for path in read_dir_files(dir, extension)? {
    if let Some(stem) = path.file_stem().and_then(|s| s.to_str())
      && stem.starts_with(prefix)
    {
      matches.push(stem.to_string());
    }
  }
  Ok(matches)
}

pub(crate) fn read_dir_files(dir: &Path, extension: &str) -> crate::Result<Vec<PathBuf>> {
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

#[cfg(test)]
mod tests {
  mod ensure_dirs {
    #[test]
    fn it_creates_all_subdirectories() {
      let dir = tempfile::tempdir().unwrap();
      crate::store::ensure_dirs(dir.path()).unwrap();

      assert!(dir.path().join("tasks").is_dir());
      assert!(dir.path().join("tasks/resolved").is_dir());
      assert!(dir.path().join("artifacts").is_dir());
      assert!(dir.path().join("artifacts/archive").is_dir());
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
