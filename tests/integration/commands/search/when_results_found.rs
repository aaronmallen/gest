use predicates::prelude::*;

use crate::support::helpers::GestCmd;

#[test]
fn it_finds_artifacts_by_title() {
  let env = GestCmd::new();

  env
    .cmd()
    .args([
      "artifact",
      "create",
      "--title",
      "uniqueartifacttitle_searchtest",
      "--body",
      "artifact body content",
    ])
    .assert()
    .success();

  env
    .cmd()
    .args(["search", "uniqueartifacttitle_searchtest"])
    .assert()
    .success()
    .stdout(predicate::str::contains("1 result for"))
    .stdout(predicate::str::contains("artifact"))
    .stdout(predicate::str::contains("uniqueartifacttitle_searchtest"));
}

#[test]
fn it_finds_tasks_by_title() {
  let env = GestCmd::new();

  env
    .cmd()
    .args(["task", "create", "uniquetasktitle_searchtest"])
    .assert()
    .success();

  env
    .cmd()
    .args(["search", "uniquetasktitle_searchtest"])
    .assert()
    .success()
    .stdout(predicate::str::contains("1 result for"))
    .stdout(predicate::str::contains("task"))
    .stdout(predicate::str::contains("uniquetasktitle_searchtest"));
}
