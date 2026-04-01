use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;

/// Build a `gest` command isolated to `temp_dir` without setting `GEST_DATA_DIR`.
///
/// `GEST_DATA_DIR` must not be set to a non-existent path because config resolution
/// fails before the command even runs. For `self-update --help` no data directory is
/// needed at all; we omit the env var so the binary falls back to its own default.
fn gest_cmd(temp_dir: &TempDir) -> Command {
  let mut cmd = Command::cargo_bin("gest").expect("gest binary not found");
  let path = temp_dir.path();
  cmd.current_dir(path);
  cmd.env("GEST_CONFIG", path.join("gest.toml"));
  cmd
}

#[test]
fn it_shows_help_without_error() {
  let temp_dir = TempDir::new().expect("failed to create temp dir");

  gest_cmd(&temp_dir)
    .args(&["self-update", "--help"])
    .assert()
    .success()
    .stdout(predicate::str::contains("self-update"));
}

#[test]
fn it_includes_target_flag_in_help() {
  let temp_dir = TempDir::new().expect("failed to create temp dir");

  gest_cmd(&temp_dir)
    .args(&["self-update", "--help"])
    .assert()
    .success()
    .stdout(predicate::str::contains("--target"));
}

#[test]
fn it_describes_the_target_flag_as_pinning_to_a_specific_version() {
  let temp_dir = TempDir::new().expect("failed to create temp dir");

  gest_cmd(&temp_dir)
    .args(&["self-update", "--help"])
    .assert()
    .success()
    .stdout(predicate::str::contains("version"));
}

#[test]
fn it_rejects_unknown_flags() {
  let temp_dir = TempDir::new().expect("failed to create temp dir");

  gest_cmd(&temp_dir)
    .args(&["self-update", "--nonexistent-flag"])
    .assert()
    .failure();
}
