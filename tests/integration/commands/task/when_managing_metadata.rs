use predicates::prelude::*;

use crate::support::helpers::GestCmd;

fn create_task_and_get_id(env: &GestCmd, title: &str) -> String {
  let output = env
    .cmd()
    .args(["task", "create", title])
    .output()
    .expect("failed to run gest task create");

  assert!(output.status.success(), "task create failed");

  let stdout = String::from_utf8(output.stdout).expect("stdout is not valid utf8");
  let first_line = stdout.lines().next().expect("no output from task create");
  first_line
    .split_whitespace()
    .last()
    .expect("no ID in create output")
    .to_string()
}

#[test]
fn it_sets_metadata() {
  let env = GestCmd::new();
  let id = create_task_and_get_id(&env, "Meta task");

  env.run(&["task", "meta", "set", &id, "priority", "high"]).success();
}

#[test]
fn it_gets_metadata() {
  let env = GestCmd::new();
  let id = create_task_and_get_id(&env, "Meta get task");

  env.run(&["task", "meta", "set", &id, "priority", "high"]).success();

  env
    .run(&["task", "meta", "get", &id, "priority"])
    .success()
    .stdout(predicate::str::contains("high"));
}
