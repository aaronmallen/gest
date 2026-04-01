use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn it_runs_binary() {
  Command::cargo_bin("gest")
    .expect("gest binary not found")
    .arg("--help")
    .assert()
    .success();
}

#[test]
fn it_shows_help() {
  Command::cargo_bin("gest")
    .expect("gest binary not found")
    .arg("--help")
    .assert()
    .success()
    .stdout(predicate::str::contains("Usage"));
}

#[test]
fn it_errors_on_unknown_command() {
  Command::cargo_bin("gest")
    .expect("gest binary not found")
    .arg("nonexistent")
    .assert()
    .failure();
}
