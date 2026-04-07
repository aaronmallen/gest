//! Lightweight smoke tests that exercise the bare form of every top-level subcommand.
//!
//! Each test invokes the command with no arguments (or with the minimum arguments
//! needed for clap to dispatch) and asserts the exit status matches the command's
//! contract -- zero for list-like defaults, non-zero for commands that require a
//! subcommand.

use crate::support::helpers::GestCmd;

#[test]
fn it_runs_artifact() {
  let g = GestCmd::new();

  let output = g.cmd().args(["artifact"]).output().expect("artifact failed to run");

  assert!(
    !output.status.success(),
    "artifact without a subcommand should print usage and exit non-zero"
  );
}

#[test]
fn it_runs_config() {
  let g = GestCmd::new();

  let output = g.cmd().args(["config"]).output().expect("config failed to run");

  assert!(
    !output.status.success(),
    "config without a subcommand should print usage and exit non-zero"
  );
}

#[test]
fn it_runs_generate() {
  let g = GestCmd::new_uninit();

  let output = g.cmd().args(["generate"]).output().expect("generate failed to run");

  assert!(
    !output.status.success(),
    "generate without a subcommand should print usage and exit non-zero"
  );
}

#[test]
fn it_runs_init() {
  let g = GestCmd::new_uninit();

  let output = g.cmd().args(["init"]).output().expect("init failed to run");

  assert!(output.status.success(), "init should succeed on a fresh temp dir");
}

#[test]
fn it_runs_iteration() {
  let g = GestCmd::new();

  let output = g.cmd().args(["iteration"]).output().expect("iteration failed to run");

  assert!(
    !output.status.success(),
    "iteration without a subcommand should print usage and exit non-zero"
  );
}

#[test]
fn it_runs_project() {
  let g = GestCmd::new();

  let output = g.cmd().args(["project"]).output().expect("project failed to run");

  assert!(output.status.success(), "project (bare) should show current project");
}

#[test]
fn it_runs_search() {
  let g = GestCmd::new();

  let output = g
    .cmd()
    .args(["search", "anything"])
    .output()
    .expect("search failed to run");

  assert!(output.status.success(), "search with a query should succeed");
}

#[test]
fn it_runs_tag() {
  let g = GestCmd::new();

  let output = g.cmd().args(["tag"]).output().expect("tag failed to run");

  assert!(output.status.success(), "tag (bare) lists tags and should succeed");
}

#[test]
fn it_runs_task() {
  let g = GestCmd::new();

  let output = g.cmd().args(["task"]).output().expect("task failed to run");

  assert!(
    !output.status.success(),
    "task without a subcommand should print usage and exit non-zero"
  );
}

#[test]
fn it_runs_undo() {
  let g = GestCmd::new();
  g.create_task("Undoable");

  let output = g.cmd().args(["undo"]).output().expect("undo failed to run");

  assert!(output.status.success(), "undo with history should succeed");
}

#[test]
fn it_runs_version() {
  let g = GestCmd::new_uninit();

  let output = g.cmd().args(["--version"]).output().expect("--version failed to run");

  assert!(output.status.success(), "--version should succeed");
}
