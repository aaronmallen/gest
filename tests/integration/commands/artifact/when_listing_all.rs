use predicates::prelude::*;

use crate::support::helpers::GestCmd;

#[test]
fn it_lists_all_including_archived() {
  let env = GestCmd::new();

  let id = env.create_artifact("Archived Artifact", "content");

  // Archive the artifact.
  env.cmd().args(["artifact", "archive", &id]).assert().success();

  // Default list should not show archived artifacts.
  env
    .cmd()
    .args(["artifact", "list"])
    .assert()
    .success()
    .stdout(predicate::str::contains("no artifacts found"));

  // --all should include the archived artifact.
  env
    .cmd()
    .args(["artifact", "list", "--all"])
    .assert()
    .success()
    .stdout(predicate::str::contains("Archived Artifact"))
    .stdout(predicate::str::contains("archived"));
}
