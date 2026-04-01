use predicates::prelude::*;

use crate::support::helpers::GestCmd;

#[test]
fn it_lists_artifacts() {
  let env = GestCmd::new();

  env
    .cmd()
    .args([
      "artifact",
      "create",
      "--title",
      "Listed Artifact",
      "--body",
      "some content",
    ])
    .assert()
    .success();

  env
    .cmd()
    .args(["artifact", "list"])
    .assert()
    .success()
    .stdout(predicate::str::contains("Listed Artifact"));
}

#[test]
fn it_lists_empty_when_none() {
  let env = GestCmd::new();

  env
    .cmd()
    .args(["artifact", "list"])
    .assert()
    .success()
    .stdout(predicate::str::contains("no artifacts found"));
}
