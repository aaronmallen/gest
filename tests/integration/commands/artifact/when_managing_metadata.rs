use predicates::prelude::*;

use crate::support::helpers::GestCmd;

#[test]
fn it_gets_metadata() {
  let env = GestCmd::new();
  let id = env.create_artifact("Meta Artifact", "content");

  // Set a metadata value first.
  env
    .cmd()
    .args(["artifact", "meta", "set", &id, "status", "draft"])
    .assert()
    .success();

  // Get it back and verify the value.
  env
    .cmd()
    .args(["artifact", "meta", "get", &id, "status"])
    .assert()
    .success()
    .stdout(predicate::str::contains("draft"));
}

#[test]
fn it_sets_metadata() {
  let env = GestCmd::new();
  let id = env.create_artifact("Meta Artifact", "content");

  env
    .cmd()
    .args(["artifact", "meta", "set", &id, "priority", "high"])
    .assert()
    .success()
    .stdout(predicate::str::contains("priority"));
}
