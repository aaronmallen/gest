use predicates::prelude::*;

use crate::support::helpers::GestCmd;

#[test]
fn it_filters_by_task_entity_type() {
  let env = GestCmd::new();

  env
    .cmd()
    .args(&["task", "create", "tagged task", "--tags", "task-only"])
    .assert()
    .success();

  env.create_artifact("tagged artifact", "body");
  // Tag the artifact via the per-entity command.
  // We need to get the artifact ID first.

  env
    .cmd()
    .args(&["tag", "list", "--task"])
    .assert()
    .success()
    .stdout(predicate::str::contains("task-only"));
}

#[test]
fn it_lists_empty_when_no_tags() {
  let env = GestCmd::new();

  env
    .cmd()
    .args(&["tag", "list"])
    .assert()
    .success()
    .stdout(predicate::str::contains("no tags found"));
}

#[test]
fn it_lists_tags_after_tagging() {
  let env = GestCmd::new();

  env
    .cmd()
    .args(&["task", "create", "a task to tag", "--tags", "integration,testing"])
    .assert()
    .success();

  env
    .cmd()
    .args(&["tag", "list"])
    .assert()
    .success()
    .stdout(predicate::str::contains("integration"))
    .stdout(predicate::str::contains("testing"));
}
