use predicates::prelude::*;

use crate::support::helpers::GestCmd;

#[test]
fn it_lists_empty_when_no_tasks() {
  let env = GestCmd::new();

  env.run(&["task", "list"]).success();
}

#[test]
fn it_lists_tasks() {
  let env = GestCmd::new();

  env.run(&["task", "create", "List me please"]).success();

  env
    .run(&["task", "list"])
    .success()
    .stdout(predicate::str::contains("List me please"));
}

#[test]
fn it_supports_json_output() {
  let env = GestCmd::new();

  env.run(&["task", "create", "JSON task"]).success();

  let output = env
    .cmd()
    .args(["task", "list", "--json"])
    .output()
    .expect("failed to run command");

  assert!(output.status.success());

  let stdout = String::from_utf8(output.stdout).expect("output not valid utf8");
  let parsed: serde_json::Value = serde_json::from_str(&stdout).expect("output is not valid JSON");

  assert!(parsed.is_array());
  let tasks = parsed.as_array().unwrap();
  assert!(!tasks.is_empty());
  assert_eq!(tasks[0]["title"].as_str().unwrap(), "JSON task");
}
