use predicates::prelude::*;

use crate::support::helpers::{GestCmd, extract_id_from_create_output};

fn create_task_id(env: &GestCmd, title: &str) -> String {
  let output = env
    .cmd()
    .args(["task", "create", title])
    .output()
    .expect("failed to run gest task create");
  assert!(output.status.success(), "task create failed");
  let stdout = String::from_utf8(output.stdout).unwrap();
  extract_id_from_create_output(&stdout).expect("no ID in create output")
}

#[test]
fn it_removes_tags_from_a_task() {
  let env = GestCmd::new();
  let id = create_task_id(&env, "untag target");

  env.run(&["tag", "add", &id, "keep", "drop"]).success();

  env
    .run(&["tag", "remove", &id, "drop"])
    .success()
    .stdout(predicate::str::contains("Untagged task"));

  env
    .run(&["task", "show", &id])
    .success()
    .stdout(predicate::str::contains("keep"))
    .stdout(predicate::str::contains("drop").not());
}
