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

fn create_task(env: &GestCmd, title: &str) -> String {
  let output = env
    .cmd()
    .args(["task", "create", title])
    .output()
    .expect("failed to run gest task create");

  let stdout = String::from_utf8_lossy(&output.stdout);
  // Output first line: "  ✓  created task  <8-char-id>"
  stdout
    .lines()
    .next()
    .and_then(|line| line.split_whitespace().last())
    .expect("no output from task create")
    .to_string()
}

#[test]
fn it_removes_a_task_from_iteration() {
  let env = GestCmd::new();
  let iter_id = create_iteration(&env, "Sprint 1");
  let task_id = create_task(&env, "Implement feature");

  env
    .cmd()
    .args(["iteration", "add", &iter_id, &task_id])
    .assert()
    .success();

  env
    .cmd()
    .args(["iteration", "remove", &iter_id, &task_id])
    .assert()
    .success()
    .stdout(predicate::str::contains("Removed task"));
}
