use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;

#[test]
fn it_creates_gest_directory_structure() {
  let temp_dir = TempDir::new().expect("failed to create temp dir");

  Command::cargo_bin("gest")
    .expect("failed to find gest binary")
    .arg("init")
    .current_dir(temp_dir.path())
    .env_remove("EDITOR")
    .env_remove("VISUAL")
    .assert()
    .success();

  assert!(temp_dir.path().join(".gest/tasks").is_dir());
  assert!(temp_dir.path().join(".gest/tasks/archive").is_dir());
  assert!(temp_dir.path().join(".gest/artifacts").is_dir());
  assert!(temp_dir.path().join(".gest/artifacts/archive").is_dir());
}

#[test]
fn it_prints_created_message() {
  let temp_dir = TempDir::new().expect("failed to create temp dir");

  Command::cargo_bin("gest")
    .expect("failed to find gest binary")
    .arg("init")
    .current_dir(temp_dir.path())
    .env_remove("EDITOR")
    .env_remove("VISUAL")
    .assert()
    .success()
    .stdout(predicate::str::contains("tasks"));
}
