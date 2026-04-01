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
fn it_adds_a_note() {
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
      "My note",
    ])
    .assert()
    .success()
    .stdout(predicate::str::contains("added note"));
}

#[test]
fn it_errors_on_nonexistent_task() {
  let env = GestCmd::new();

  env
    .cmd()
    .args([
      "task",
      "note",
      "add",
      "nonexistent-task-id",
      "--agent",
      "test-agent",
      "--body",
      "My note",
    ])
    .assert()
    .failure();
}
