use predicates::prelude::*;

use crate::support::helpers::GestCmd;

#[test]
fn it_creates_a_task() {
  let env = GestCmd::new();

  env
    .run(&["task", "create", "My task"])
    .success()
    .stdout(predicate::str::contains("created task"));
}

#[test]
fn it_creates_a_task_with_priority() {
  let env = GestCmd::new();

  env
    .run(&["task", "create", "My task", "--priority", "1"])
    .success()
    .stdout(predicate::str::contains("created task"));
}

#[test]
fn it_errors_without_title() {
  let env = GestCmd::new();

  env.run(&["task", "create"]).failure();
}
