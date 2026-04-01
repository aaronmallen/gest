use predicates::prelude::*;

use crate::support::helpers::GestCmd;

#[test]
fn it_expands_results() {
  let env = GestCmd::new();

  env
    .cmd()
    .args([
      "task",
      "create",
      "expandable_task_searchtest",
      "--description",
      "This is a detailed description for the expanded view test.",
    ])
    .assert()
    .success();

  env
    .cmd()
    .args(["search", "--expand", "expandable_task_searchtest"])
    .assert()
    .success()
    .stdout(predicate::str::contains("1 result for"))
    .stdout(predicate::str::contains("task"))
    .stdout(predicate::str::contains("expandable_task_searchtest"))
    .stdout(predicate::str::contains(
      "This is a detailed description for the expanded view test.",
    ));
}
