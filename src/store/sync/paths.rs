//! Repo-relative path helpers for the per-entity `.gest/` layout (ADR-0016).
//!
//! Every entity type has a single function that produces the absolute filesystem
//! path inside a project's `gest_dir`. The functions are pure — they do no I/O —
//! so they can be used both at write time (where we want to know where to put a
//! file) and at read time (where we walk the directory and need the inverse).
//!
//! The corresponding repo-relative path (suitable for the `sync_digests` cache
//! key) is produced by [`relative`], which strips the `gest_dir` prefix.

use std::path::{Path, PathBuf};

use chrono::{DateTime, Datelike, Utc};

use crate::store::model::primitives::Id;

/// File name of the project metadata file.
pub const PROJECT_FILE: &str = "project.yaml";

/// Subdirectory name for artifact entity files.
pub const ARTIFACT_DIR: &str = "artifact";

/// Subdirectory name for author entity files.
pub const AUTHOR_DIR: &str = "author";

/// Subdirectory name for event entity files (sharded by month).
pub const EVENT_DIR: &str = "event";

/// Subdirectory name for iteration entity files.
pub const ITERATION_DIR: &str = "iteration";

/// Subdirectory name nested under each entity type that owns notes.
pub const NOTES_DIR: &str = "notes";

/// Subdirectory name for relationship entity files.
pub const RELATIONSHIP_DIR: &str = "relationship";

/// Subdirectory name for tag entity files.
pub const TAG_DIR: &str = "tag";

/// Subdirectory name for task entity files.
pub const TASK_DIR: &str = "task";

/// Path to an artifact body file: `artifact/<id>.md`.
pub fn artifact_path(gest_dir: &Path, id: &Id) -> PathBuf {
  gest_dir.join(ARTIFACT_DIR).join(format!("{}.md", id))
}

/// Path to an artifact note file: `artifact/notes/<note_id>.yaml`.
pub fn artifact_note_path(gest_dir: &Path, note_id: &Id) -> PathBuf {
  gest_dir
    .join(ARTIFACT_DIR)
    .join(NOTES_DIR)
    .join(format!("{}.yaml", note_id))
}

/// Path to an author file: `author/<id>.yaml`.
pub fn author_path(gest_dir: &Path, id: &Id) -> PathBuf {
  gest_dir.join(AUTHOR_DIR).join(format!("{}.yaml", id))
}

/// Path to an event file: `event/<yyyy-mm>/<event_id>.yaml`.
///
/// Events are sharded into monthly subdirectories (per the event's
/// `created_at` timestamp) so that no single directory grows without bound.
pub fn event_path(gest_dir: &Path, event_id: &Id, created_at: &DateTime<Utc>) -> PathBuf {
  gest_dir
    .join(EVENT_DIR)
    .join(event_shard(created_at))
    .join(format!("{}.yaml", event_id))
}

/// Compute the monthly shard segment for an event timestamp (e.g. `2026-04`).
pub fn event_shard(created_at: &DateTime<Utc>) -> String {
  format!("{:04}-{:02}", created_at.year(), created_at.month())
}

/// Path to an iteration file: `iteration/<id>.yaml`.
pub fn iteration_path(gest_dir: &Path, id: &Id) -> PathBuf {
  gest_dir.join(ITERATION_DIR).join(format!("{}.yaml", id))
}

/// Path to an iteration note file: `iteration/notes/<note_id>.yaml`.
pub fn iteration_note_path(gest_dir: &Path, note_id: &Id) -> PathBuf {
  gest_dir
    .join(ITERATION_DIR)
    .join(NOTES_DIR)
    .join(format!("{}.yaml", note_id))
}

/// Path to the project metadata file: `project.yaml`.
pub fn project_path(gest_dir: &Path) -> PathBuf {
  gest_dir.join(PROJECT_FILE)
}

/// Compute the repo-relative form of `path` against `gest_dir`.
///
/// Returns `None` if `path` is not contained in `gest_dir`. The result uses
/// forward slashes regardless of host platform so that `sync_digests` keys are
/// portable across operating systems.
pub fn relative(gest_dir: &Path, path: &Path) -> Option<String> {
  let stripped = path.strip_prefix(gest_dir).ok()?;
  let mut parts = Vec::new();
  for component in stripped.components() {
    parts.push(component.as_os_str().to_string_lossy().into_owned());
  }
  Some(parts.join("/"))
}

/// Path to a relationship file: `relationship/<id>.yaml`.
pub fn relationship_path(gest_dir: &Path, id: &Id) -> PathBuf {
  gest_dir.join(RELATIONSHIP_DIR).join(format!("{}.yaml", id))
}

/// Path to a tag file: `tag/<id>.yaml`.
pub fn tag_path(gest_dir: &Path, id: &Id) -> PathBuf {
  gest_dir.join(TAG_DIR).join(format!("{}.yaml", id))
}

/// Path to a task file: `task/<id>.yaml`.
pub fn task_path(gest_dir: &Path, id: &Id) -> PathBuf {
  gest_dir.join(TASK_DIR).join(format!("{}.yaml", id))
}

/// Path to a task note file: `task/notes/<note_id>.yaml`.
pub fn task_note_path(gest_dir: &Path, note_id: &Id) -> PathBuf {
  gest_dir
    .join(TASK_DIR)
    .join(NOTES_DIR)
    .join(format!("{}.yaml", note_id))
}

#[cfg(test)]
mod tests {
  use std::path::PathBuf;

  use chrono::TimeZone;

  use super::*;

  fn gest_dir() -> PathBuf {
    PathBuf::from("/tmp/proj/.gest")
  }

  fn id(s: &str) -> Id {
    s.parse().unwrap()
  }

  mod artifact_path {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_places_artifact_under_artifact_directory_with_md_extension() {
      let path = artifact_path(&gest_dir(), &id("kkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkk"));

      assert_eq!(path, gest_dir().join("artifact/kkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkk.md"));
    }
  }

  mod artifact_note_path {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_nests_artifact_notes_under_artifact_notes_directory() {
      let path = artifact_note_path(&gest_dir(), &id("kkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkk"));

      assert_eq!(
        path,
        gest_dir().join("artifact/notes/kkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkk.yaml")
      );
    }
  }

  mod event_path {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_shards_events_by_month_subdirectory() {
      let when = Utc.with_ymd_and_hms(2026, 4, 15, 12, 0, 0).unwrap();
      let path = event_path(&gest_dir(), &id("kkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkk"), &when);

      assert_eq!(
        path,
        gest_dir().join("event/2026-04/kkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkk.yaml")
      );
    }

    #[test]
    fn it_zero_pads_month_for_january() {
      let when = Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap();
      let path = event_path(&gest_dir(), &id("kkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkk"), &when);

      assert_eq!(
        path,
        gest_dir().join("event/2026-01/kkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkk.yaml")
      );
    }
  }

  mod relative {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_strips_the_gest_dir_prefix() {
      let path = task_path(&gest_dir(), &id("kkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkk"));

      let rel = relative(&gest_dir(), &path);

      assert_eq!(rel.as_deref(), Some("task/kkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkk.yaml"));
    }

    #[test]
    fn it_returns_none_for_paths_outside_the_gest_dir() {
      let outside = PathBuf::from("/etc/passwd");

      let rel = relative(&gest_dir(), &outside);

      assert!(rel.is_none());
    }

    #[test]
    fn it_uses_forward_slashes_for_nested_paths() {
      let when = Utc.with_ymd_and_hms(2026, 4, 15, 0, 0, 0).unwrap();
      let path = event_path(&gest_dir(), &id("kkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkk"), &when);

      let rel = relative(&gest_dir(), &path);

      assert_eq!(
        rel.as_deref(),
        Some("event/2026-04/kkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkk.yaml")
      );
    }
  }
}
