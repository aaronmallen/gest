use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;

/// Build a `gest` command isolated to `temp_dir` but without setting `GEST_DATA_DIR`.
///
/// `GEST_DATA_DIR` must not be set to a non-existent path because config resolution
/// fails before `init` can create the directory. `--local` mode creates `.gest` in
/// the current working directory, bypassing the data-dir check.
fn gest_local_init_cmd(temp_dir: &TempDir) -> Command {
  let mut cmd = Command::cargo_bin("gest").unwrap();
  let path = temp_dir.path();
  cmd.current_dir(path);
  cmd.env("GEST_CONFIG", path.join("gest.toml"));
  cmd.env("GEST_STATE_DIR", path.join(".gest-state"));
  // Do NOT set GEST_DATA_DIR — it must not point to a non-existent directory.
  cmd
}

#[test]
fn it_creates_expected_directory_structure() {
  let temp_dir = TempDir::new().expect("failed to create temp dir");

  gest_local_init_cmd(&temp_dir)
    .args(["init", "--local"])
    .assert()
    .success();

  let base = temp_dir.path().join(".gest");
  assert!(base.join("tasks").is_dir(), ".gest/tasks should exist");
  assert!(
    base.join("tasks/resolved").is_dir(),
    ".gest/tasks/resolved should exist"
  );
  assert!(base.join("artifacts").is_dir(), ".gest/artifacts should exist");
  assert!(
    base.join("artifacts/archive").is_dir(),
    ".gest/artifacts/archive should exist"
  );
  assert!(base.join("iterations").is_dir(), ".gest/iterations should exist");
  assert!(
    base.join("iterations/resolved").is_dir(),
    ".gest/iterations/resolved should exist"
  );
}

#[test]
fn it_initializes_a_new_project() {
  let temp_dir = TempDir::new().expect("failed to create temp dir");

  gest_local_init_cmd(&temp_dir)
    .args(["init", "--local"])
    .assert()
    .success();

  assert!(temp_dir.path().join(".gest").is_dir());
}

#[test]
fn it_is_idempotent_when_already_initialized() {
  let temp_dir = TempDir::new().expect("failed to create temp dir");

  gest_local_init_cmd(&temp_dir)
    .args(["init", "--local"])
    .assert()
    .success();
  gest_local_init_cmd(&temp_dir)
    .args(["init", "--local"])
    .assert()
    .success();

  let base = temp_dir.path().join(".gest");
  assert!(base.join("tasks").is_dir());
  assert!(base.join("artifacts").is_dir());
  assert!(base.join("iterations").is_dir());
}

#[test]
fn it_outputs_initialized_gest_on_success() {
  let temp_dir = TempDir::new().expect("failed to create temp dir");

  gest_local_init_cmd(&temp_dir)
    .args(["init", "--local"])
    .assert()
    .success()
    .stdout(predicate::str::contains("initialized gest"));
}
