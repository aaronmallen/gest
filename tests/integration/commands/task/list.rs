use predicates::prelude::*;

use crate::support::helpers::GestCmd;

#[test]
fn it_lists_with_no_tasks() {
  let env = GestCmd::new();

  env
    .run(["task", "list"])
    .assert()
    .success()
    .stdout(predicate::str::contains("No tasks found."));
}

#[test]
fn it_lists_after_creating_a_task() {
  let env = GestCmd::new();

  env
    .run(["task", "create", "Listed Task", "-d", "for listing"])
    .assert()
    .success();

  env
    .run(["task", "list"])
    .assert()
    .success()
    .stdout(predicate::str::contains("Listed Task"));
}

#[test]
fn it_lists_tasks_as_json() {
  let env = GestCmd::new();

  env
    .run(["task", "create", "JSON Listed", "-d", "json list"])
    .assert()
    .success();

  let output = env
    .run(["task", "list", "--json"])
    .output()
    .expect("failed to list tasks");
  assert!(output.status.success());

  let stdout = String::from_utf8_lossy(&output.stdout);
  let json: serde_json::Value = serde_json::from_str(&stdout).expect("invalid JSON output");
  let arr = json.as_array().expect("JSON output should be an array");
  assert_eq!(arr.len(), 1, "Should have exactly one task");
  assert_eq!(arr[0]["title"], "JSON Listed");
}
