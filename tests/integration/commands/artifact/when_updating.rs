use predicates::prelude::*;

use crate::support::helpers::GestCmd;

#[test]
fn it_updates_artifact_title() {
  let env = GestCmd::new();

  let id = env.create_artifact("Original Title", "artifact content");

  env
    .cmd()
    .args(["artifact", "update", &id, "--title", "Updated Title"])
    .assert()
    .success()
    .stdout(predicate::str::contains("updated artifact"))
    .stdout(predicate::str::contains("Updated Title"));

  // Verify the new title appears when showing the artifact.
  env
    .cmd()
    .args(["artifact", "show", &id])
    .assert()
    .success()
    .stdout(predicate::str::contains("Updated Title"));
}
