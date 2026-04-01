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
fn it_advances_phase() {
  let env = GestCmd::new();
  let iter_id = create_iteration(&env, "Sprint 1");

  // Create two tasks in different phases
  let task1_id = create_task(&env, "Phase 1 task");
  let task2_id = create_task(&env, "Phase 2 task");

  // Assign tasks to phases
  env
    .cmd()
    .args(["task", "update", &task1_id, "--phase", "1"])
    .assert()
    .success();

  env
    .cmd()
    .args(["task", "update", &task2_id, "--phase", "2"])
    .assert()
    .success();

  // Add both tasks to the iteration
  env
    .cmd()
    .args(["iteration", "add", &iter_id, &task1_id])
    .assert()
    .success();

  env
    .cmd()
    .args(["iteration", "add", &iter_id, &task2_id])
    .assert()
    .success();

  // Force advance the iteration (phase 1 still has an open task, but we force it)
  env
    .cmd()
    .args(["iteration", "advance", &iter_id, "--force"])
    .assert()
    .success()
    .stdout(predicate::str::contains("Advanced iteration").or(predicate::str::contains("All phases complete")));
}
