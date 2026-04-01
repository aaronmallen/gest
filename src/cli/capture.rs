//! Filesystem capture for the event store.
//!
//! Snapshots `data_dir` before and after a CLI command, recording changed files
//! as events in the event store. Only files that actually changed produce events;
//! if nothing changed, no transaction is created.

use std::{collections::HashMap, fs, path::Path};

use sha2::{Digest, Sha256};

use crate::{
  config::Settings,
  event_store::{EventStore, EventType},
};

/// A pre-command snapshot of all files under `data_dir`.
pub(crate) struct Snapshot {
  /// Map from relative path (to data_dir) → (content hash, raw content).
  files: HashMap<String, FileEntry>,
}

struct FileEntry {
  hash: [u8; 32],
  content: Vec<u8>,
}

impl Snapshot {
  /// Walk `data_dir` recursively and capture every file's content.
  pub(crate) fn capture(data_dir: &Path) -> Self {
    let mut files = HashMap::new();
    if data_dir.is_dir() {
      walk_dir(data_dir, data_dir, &mut files);
    }
    Self {
      files,
    }
  }

  /// Compare against the current filesystem state, recording any differences
  /// as events in the event store.
  ///
  /// Returns `true` if any events were recorded.
  pub(crate) fn record_changes(
    &self,
    data_dir: &Path,
    store: &EventStore,
    transaction_id: &str,
  ) -> crate::event_store::Result<bool> {
    let after = Self::capture(data_dir);
    let mut recorded = false;

    // Check for modified and deleted files.
    for (rel_path, before_entry) in &self.files {
      match after.files.get(rel_path) {
        Some(after_entry) if before_entry.hash != after_entry.hash => {
          // File was modified.
          store.record_event(
            transaction_id,
            rel_path,
            EventType::Modified,
            Some(&before_entry.content),
          )?;
          recorded = true;
        }
        None => {
          // File was deleted.
          store.record_event(
            transaction_id,
            rel_path,
            EventType::Deleted,
            Some(&before_entry.content),
          )?;
          recorded = true;
        }
        _ => {} // Unchanged.
      }
    }

    // Check for created files (present after but not before).
    for rel_path in after.files.keys() {
      if !self.files.contains_key(rel_path) {
        store.record_event(transaction_id, rel_path, EventType::Created, None)?;
        recorded = true;
      }
    }

    Ok(recorded)
  }
}

/// Recursively collect all files under `dir`, keyed by path relative to `base`.
fn walk_dir(base: &Path, dir: &Path, out: &mut HashMap<String, FileEntry>) {
  let entries = match fs::read_dir(dir) {
    Ok(e) => e,
    Err(_) => return,
  };

  for entry in entries.flatten() {
    let path = entry.path();
    if path.is_dir() {
      walk_dir(base, &path, out);
    } else if path.is_file()
      && let Ok(content) = fs::read(&path)
    {
      let rel = path.strip_prefix(base).unwrap_or(&path).to_string_lossy().into_owned();
      let hash = Sha256::digest(&content).into();
      out.insert(
        rel,
        FileEntry {
          hash,
          content,
        },
      );
    }
  }
}

/// Build the command string from process arguments for the transaction record.
pub(crate) fn command_string() -> String {
  std::env::args().collect::<Vec<_>>().join(" ")
}

/// Extract the project ID from the state directory path.
///
/// The state dir is always `<state_home>/gest/<project_hash>/`, so the last
/// component is the project hash.
pub(crate) fn project_id(settings: &Settings) -> String {
  settings
    .state_dir()
    .file_name()
    .map(|n| n.to_string_lossy().into_owned())
    .unwrap_or_default()
}
