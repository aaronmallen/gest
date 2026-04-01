use predicates::prelude::*;

use crate::support::helpers::GestCmd;

#[test]
fn it_lists_tags_after_tagging() {
  let env = GestCmd::new();

  // Create a task with tags directly.
  env
    .cmd()
    .args(&["task", "create", "a task to tag", "--tags", "integration,testing"])
    .assert()
    .success();

  // List tags and verify both tags appear.
  env
    .cmd()
    .args(&["tags"])
    .assert()
    .success()
    .stdout(predicate::str::contains("integration"))
    .stdout(predicate::str::contains("testing"));
}

#[test]
fn it_lists_empty_when_no_tags() {
  let env = GestCmd::new();

  env
    .cmd()
    .args(&["tags"])
    .assert()
    .success()
    .stdout(predicate::str::contains("no tags found"));
}
