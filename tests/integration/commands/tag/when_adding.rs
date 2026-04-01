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
fn it_adds_tags_to_a_task() {
  let env = GestCmd::new();
  let id = create_task_id(&env, "tag target task");

  env
    .run(&["tag", "add", &id, "foo", "bar"])
    .success()
    .stdout(predicate::str::contains("Tagged task"));

  env
    .run(&["task", "show", &id])
    .success()
    .stdout(predicate::str::contains("foo"))
    .stdout(predicate::str::contains("bar"));
}

#[test]
fn it_adds_tags_to_an_artifact() {
  let env = GestCmd::new();
  let id = env.create_artifact("tag target artifact", "body");

  env
    .run(&["tag", "add", &id, "spec"])
    .success()
    .stdout(predicate::str::contains("Tagged artifact"));
}

#[test]
fn it_errors_on_unknown_id() {
  let env = GestCmd::new();

  env
    .run(&["tag", "add", "zzzzzzzz", "foo"])
    .failure()
    .stderr(predicate::str::contains("No entity found"));
}
