//! Exit code contract tests covering every sysexits family the CLI promises,
//! per ADR-0017 "Exit Code Contract for the gest CLI".
//!
//! Each test runs one command known to surface a specific `cli::Error` variant
//! and asserts the process exit code matches the sysexits mapping. These tests
//! are the wire-level guard for that contract: if a refactor reroutes an error
//! through a different variant and the numeric code shifts, one of these
//! assertions fires.

use crate::support::helpers::GestCmd;

#[test]
fn it_exits_0_on_success() {
  let g = GestCmd::new_uninit();

  let output = g.cmd().args(["--version"]).output().expect("--version failed to run");

  assert_eq!(
    output.status.code(),
    Some(0),
    "--version should exit 0, got {:?}: {}",
    output.status.code(),
    String::from_utf8_lossy(&output.stderr)
  );
}

#[test]
fn it_exits_64_on_argument_error() {
  // `iteration next --agent X` without `--claim` is rejected in the handler as
  // `Error::Argument`, which maps to EX_USAGE (64). Clap-level parse failures
  // exit with its own default (2) and are not part of the sysexits contract.
  let g = GestCmd::new();
  let iter_id = g.create_iteration("usage sprint");

  let output = g
    .cmd()
    .args(["iteration", "next", &iter_id, "--agent", "noone"])
    .output()
    .expect("iteration next failed to run");

  assert_eq!(
    output.status.code(),
    Some(64),
    "--agent without --claim should exit EX_USAGE (64), got {:?}: {}",
    output.status.code(),
    String::from_utf8_lossy(&output.stderr)
  );
}

#[test]
fn it_exits_65_on_data_error() {
  // `task meta set --as-json` feeds the value to `serde_json::from_str`; an
  // invalid JSON literal surfaces `Error::Serialize`, which maps to EX_DATAERR (65).
  let g = GestCmd::new();
  let task_id = g.create_task("dataerr task");

  let output = g
    .cmd()
    .args(["task", "meta", "set", &task_id, "foo", "{not json", "--as-json"])
    .output()
    .expect("task meta set failed to run");

  assert_eq!(
    output.status.code(),
    Some(65),
    "invalid --as-json payload should exit EX_DATAERR (65), got {:?}: {}",
    output.status.code(),
    String::from_utf8_lossy(&output.stderr)
  );
}

#[test]
fn it_exits_66_on_missing_metadata_key() {
  // `task meta get <id> <path>` returns `Error::MetaKeyNotFound` when the
  // dot-path does not resolve, mapping to EX_NOINPUT (66).
  let g = GestCmd::new();
  let task_id = g.create_task("noinput task");

  let output = g
    .cmd()
    .args(["task", "meta", "get", &task_id, "never.set.here"])
    .output()
    .expect("task meta get failed to run");

  assert_eq!(
    output.status.code(),
    Some(66),
    "missing metadata key should exit EX_NOINPUT (66), got {:?}: {}",
    output.status.code(),
    String::from_utf8_lossy(&output.stderr)
  );
}

#[test]
fn it_exits_69_on_invalid_state() {
  // Completing the single task in a one-phase iteration auto-completes the
  // iteration; subsequent `iteration advance` rejects the transition with
  // `Error::InvalidState`, mapping to EX_UNAVAILABLE (69).
  let g = GestCmd::new();
  let iter_id = g.create_iteration_with_phases("done sprint", &[&["only task"]]);

  let claim = g
    .cmd()
    .args(["iteration", "next", &iter_id, "--claim", "--agent", "test", "-q"])
    .output()
    .expect("iteration next --claim failed to run");
  assert!(claim.status.success(), "claim should succeed");
  let task_id = String::from_utf8_lossy(&claim.stdout).trim().to_string();
  g.complete_task(&task_id);

  let output = g
    .cmd()
    .args(["iteration", "advance", &iter_id])
    .output()
    .expect("iteration advance failed to run");

  assert_eq!(
    output.status.code(),
    Some(69),
    "advancing a non-active iteration should exit EX_UNAVAILABLE (69), got {:?}: {}",
    output.status.code(),
    String::from_utf8_lossy(&output.stderr)
  );
}

#[test]
fn it_exits_70_on_editor_failure() {
  // `task note add -b -` invokes `$EDITOR`; pointing it at `false` forces a
  // non-zero exit inside the editor process, surfacing `Error::Editor` which
  // maps to EX_SOFTWARE (70).
  let g = GestCmd::new();
  let task_id = g.create_task("editor task");

  let output = g
    .cmd()
    .env("EDITOR", "false")
    .env("VISUAL", "")
    .args(["task", "note", "add", &task_id, "-b", "-"])
    .output()
    .expect("task note add failed to run");

  assert_eq!(
    output.status.code(),
    Some(70),
    "editor process failure should exit EX_SOFTWARE (70), got {:?}: {}",
    output.status.code(),
    String::from_utf8_lossy(&output.stderr)
  );
}

#[test]
fn it_exits_74_on_io_error() {
  // Planting a directory at the sqlite db path makes `gest init` fail to open
  // the connection. The resulting I/O error is wrapped in `Error::Store` which
  // maps to EX_IOERR (74).
  let g = GestCmd::new_uninit();
  let db_path = g.db_path();
  std::fs::create_dir_all(&db_path).expect("failed to plant dir at db path");

  let output = g.cmd().args(["init"]).output().expect("init failed to run");

  assert_eq!(
    output.status.code(),
    Some(74),
    "init against unusable db path should exit EX_IOERR (74), got {:?}: {}",
    output.status.code(),
    String::from_utf8_lossy(&output.stderr)
  );
}

#[test]
fn it_exits_75_on_no_tasks_available() {
  // `iteration next` on an iteration with no open candidates surfaces
  // `Error::NoTasksAvailable`, which maps to EX_TEMPFAIL (75).
  let g = GestCmd::new();
  let iter_id = g.create_iteration("empty sprint");

  let output = g
    .cmd()
    .args(["iteration", "next", &iter_id])
    .output()
    .expect("iteration next failed to run");

  assert_eq!(
    output.status.code(),
    Some(75),
    "iteration next with no tasks should exit EX_TEMPFAIL (75), got {:?}: {}",
    output.status.code(),
    String::from_utf8_lossy(&output.stderr)
  );
}

#[test]
fn it_exits_78_on_uninitialized_project() {
  // Project-dependent commands short-circuit with `Error::UninitializedProject`
  // in an unconfigured directory; the variant maps to EX_CONFIG (78).
  let g = GestCmd::new_uninit();

  let output = g
    .cmd()
    .args(["task", "list"])
    .output()
    .expect("task list failed to run");

  assert_eq!(
    output.status.code(),
    Some(78),
    "task list without a project should exit EX_CONFIG (78), got {:?}: {}",
    output.status.code(),
    String::from_utf8_lossy(&output.stderr)
  );
}
