use predicates::prelude::*;

use crate::support::helpers::GestCmd;

fn create_iteration(env: &GestCmd, title: &str) -> String {
  let output = env
    .cmd()
    .args(["iteration", "create", title])
    .output()
    .expect("failed to run gest iteration create");

  let stdout = String::from_utf8_lossy(&output.stdout);
  stdout
    .split_whitespace()
    .last()
    .expect("no output from iteration create")
    .to_string()
}

#[test]
fn it_updates_iteration_status() {
  let env = GestCmd::new();
  let id = create_iteration(&env, "Sprint 1");

  env
    .cmd()
    .args(["iteration", "update", &id, "--status", "completed"])
    .assert()
    .success()
    .stdout(predicate::str::contains("Updated iteration"));
}

#[test]
fn it_updates_iteration_title() {
  let env = GestCmd::new();
  let id = create_iteration(&env, "Old Title");

  env
    .cmd()
    .args(["iteration", "update", &id, "--title", "New Title"])
    .assert()
    .success()
    .stdout(predicate::str::contains("Updated iteration"));

  env
    .cmd()
    .args(["iteration", "show", &id])
    .assert()
    .success()
    .stdout(predicate::str::contains("New Title"));
}
