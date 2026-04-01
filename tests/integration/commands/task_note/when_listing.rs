use predicates::prelude::*;

use crate::support::helpers::GestCmd;

fn create_task_id(env: &GestCmd) -> String {
  env
    .cmd()
    .args(["task", "create", "Test Task", "--description", "A test task"])
    .assert()
    .success();

  let output = env
    .cmd()
    .args(["task", "list", "--json", "--all"])
    .output()
    .expect("failed to run task list");

  let tasks: serde_json::Value = serde_json::from_slice(&output.stdout).expect("failed to parse task list JSON");

  tasks[0]["id"].as_str().expect("task id not found in JSON").to_string()
}

#[test]
fn it_lists_notes() {
  let env = GestCmd::new();
  let task_id = create_task_id(&env);

  env
    .cmd()
    .args([
      "task",
      "note",
      "add",
      &task_id,
      "--agent",
      "test-agent",
      "--body",
      "First note",
    ])
    .assert()
    .success();

  env
    .cmd()
    .args(["task", "note", "list", &task_id])
    .assert()
    .success()
    .stdout(predicate::str::contains("First note"));
}

#[test]
fn it_lists_empty_when_no_notes() {
  let env = GestCmd::new();
  let task_id = create_task_id(&env);

  env
    .cmd()
    .args(["task", "note", "list", &task_id])
    .assert()
    .success()
    .stdout(predicate::str::contains("No notes on task"));
}
