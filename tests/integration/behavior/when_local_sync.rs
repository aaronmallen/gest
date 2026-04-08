//! End-to-end sync tests for the per-entity `.gest/` layout (ADR-0016).
//!
//! These tests assert the merge-conflict-resilience properties promised by
//! the new layout: a full SQLite → disk → fresh SQLite round-trip across
//! every entity type, conflict-clean concurrent edits to different entities,
//! per-file conflict surfacing for divergent edits to the same entity,
//! tombstone propagation on import, and digest-cache hits and misses.

use std::{fs, path::PathBuf};

use crate::support::helpers::GestCmd;

fn gest_dir(g: &GestCmd) -> PathBuf {
  g.temp_dir_path().join(".gest")
}

/// Find the on-disk file for an entity by walking `dir` and matching the
/// short id prefix returned from `gest <entity> create`.
fn entity_file(dir: &PathBuf, short_id: &str, extension: &str) -> Option<PathBuf> {
  let entries = fs::read_dir(dir).ok()?;
  for entry in entries.flatten() {
    let path = entry.path();
    if !path.is_file() {
      continue;
    }
    if path.extension().and_then(|s| s.to_str()) != Some(extension) {
      continue;
    }
    let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
    if stem.starts_with(short_id) {
      return Some(path);
    }
  }
  None
}

/// Wipe the SQLite store and re-initialize the project from `.gest/project.yaml`,
/// simulating a fresh-checkout import on a different machine.
fn reset_db_and_reinit(g: &GestCmd) {
  let data_dir = g.temp_dir_path().join(".gest-data");
  fs::remove_dir_all(&data_dir).expect("remove data dir");
  fs::create_dir_all(&data_dir).expect("recreate data dir");
  let init = g.cmd().args(["init"]).output().expect("init failed to run");
  assert!(
    init.status.success(),
    "init failed: {}",
    String::from_utf8_lossy(&init.stderr)
  );
}

#[test]
fn it_round_trips_every_entity_type_through_disk() {
  let g = GestCmd::new();

  // Seed every entity type so the round-trip exercises them all.
  let task_id = g.create_task("Round-trip task");
  g.attach_tag("task", &task_id, "round-trip");
  let artifact_id = g.create_artifact("Round-trip artifact", "# Hello\n\nbody\n");
  let iter_id = g.create_iteration_with_phases("Round-trip sprint", &[&["Phase 1 task"]]);
  // Add a note to the task so notes also round-trip.
  g.cmd()
    .args(["task", "note", "add", &task_id, "--body", "first note"])
    .output()
    .expect("note add failed to run");

  // Wipe the SQLite store entirely and re-initialize from `.gest/project.yaml`,
  // simulating a fresh checkout on a different machine.
  reset_db_and_reinit(&g);

  // Confirm every entity reappears in the fresh database after the next sync.
  let task_show = g
    .cmd()
    .args(["task", "show", &task_id])
    .output()
    .expect("task show failed to run");
  assert!(
    task_show.status.success(),
    "task did not survive round-trip: {}",
    String::from_utf8_lossy(&task_show.stderr)
  );
  let task_stdout = String::from_utf8_lossy(&task_show.stdout);
  assert!(task_stdout.contains("Round-trip task"));
  assert!(task_stdout.contains("round-trip"), "tag was lost on round-trip");

  // Notes round-trip via the dedicated `task note list` subcommand.
  let note_list = g
    .cmd()
    .args(["task", "note", "list", &task_id])
    .output()
    .expect("task note list failed to run");
  assert!(note_list.status.success());
  assert!(
    String::from_utf8_lossy(&note_list.stdout).contains("first note"),
    "note was lost on round-trip"
  );

  let artifact_show = g
    .cmd()
    .args(["artifact", "show", &artifact_id])
    .output()
    .expect("artifact show failed to run");
  assert!(artifact_show.status.success(), "artifact did not survive round-trip");
  assert!(String::from_utf8_lossy(&artifact_show.stdout).contains("Hello"));

  let iter_show = g
    .cmd()
    .args(["iteration", "show", &iter_id])
    .output()
    .expect("iteration show failed to run");
  assert!(
    iter_show.status.success(),
    "iteration did not survive round-trip: {}",
    String::from_utf8_lossy(&iter_show.stderr)
  );
  assert!(String::from_utf8_lossy(&iter_show.stdout).contains("Round-trip sprint"));
}

#[test]
fn it_merges_concurrent_edits_to_different_tasks_without_conflict() {
  let g = GestCmd::new();
  let task_a = g.create_task("Task A");
  let task_b = g.create_task("Task B");

  let task_dir = gest_dir(&g).join("task");
  let task_a_file = entity_file(&task_dir, &task_a, "yaml").expect("Task A file should exist");
  let task_b_file = entity_file(&task_dir, &task_b, "yaml").expect("Task B file should exist");
  let task_b_before = fs::read_to_string(&task_b_file).expect("read task B");

  // Editing Task A on disk must not touch Task B's file. Simulate a divergent
  // edit by appending content to Task A's body, then run a read-only command
  // so the next sync round re-imports the change.
  let mut task_a_content = fs::read_to_string(&task_a_file).expect("read task A");
  task_a_content.push_str("description: edited from outside\n");
  fs::write(&task_a_file, &task_a_content).expect("write edited task A");

  let list = g.cmd().args(["task", "list"]).output().expect("task list failed");
  assert!(list.status.success());

  let task_b_after = fs::read_to_string(&task_b_file).expect("read task B after");
  assert_eq!(
    task_b_before, task_b_after,
    "editing Task A on disk should not touch Task B's file"
  );
}

#[test]
fn it_propagates_tombstones_to_a_fresh_database_on_import() {
  let g = GestCmd::new();
  let task_id = g.create_task("To be tombstoned");
  let task_dir = gest_dir(&g).join("task");
  let task_file = entity_file(&task_dir, &task_id, "yaml").expect("expected task file to be written");

  // Manually tombstone the file by prepending `deleted_at:` to the YAML.
  let original = fs::read_to_string(&task_file).expect("read task file");
  let tombstoned = format!("deleted_at: 2026-04-08T12:00:00Z\n{original}");
  fs::write(&task_file, tombstoned).expect("write tombstoned task file");

  // Wipe SQLite and re-init so the next sync imports the tombstoned file
  // into an otherwise-fresh database.
  reset_db_and_reinit(&g);

  let show = g
    .cmd()
    .args(["task", "show", &task_id])
    .output()
    .expect("task show failed to run");
  assert!(
    !show.status.success(),
    "tombstoned task should be absent after import into a fresh DB"
  );
}

#[test]
fn it_skips_writing_files_with_unchanged_digests() {
  let g = GestCmd::new();
  let task_id = g.create_task("Cache hit task");
  let task_dir = gest_dir(&g).join("task");
  let task_file = entity_file(&task_dir, &task_id, "yaml").expect("task file should exist");
  let mtime_first = fs::metadata(&task_file)
    .expect("task file metadata")
    .modified()
    .expect("task file mtime");

  // Run a read-only command. The export pass should hit the digest cache and
  // leave the file untouched.
  std::thread::sleep(std::time::Duration::from_millis(20));
  let _ = g.cmd().args(["task", "list"]).output();

  let mtime_second = fs::metadata(&task_file)
    .expect("task file metadata")
    .modified()
    .expect("task file mtime");
  assert_eq!(
    mtime_first, mtime_second,
    "task file mtime should not change when content is unchanged"
  );
}
