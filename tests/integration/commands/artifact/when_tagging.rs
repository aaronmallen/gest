use predicates::prelude::*;

use crate::support::helpers::GestCmd;

#[test]
fn it_tags_an_artifact() {
  let env = GestCmd::new();

  let id = env.create_artifact("Tag Me", "artifact content");

  env
    .cmd()
    .args(["artifact", "tag", &id, "rust", "cli"])
    .assert()
    .success()
    .stdout(predicate::str::contains("Tagged artifact"));

  // Verify the tags appear when showing the artifact.
  env
    .cmd()
    .args(["artifact", "show", &id])
    .assert()
    .success()
    .stdout(predicate::str::contains("rust"))
    .stdout(predicate::str::contains("cli"));
}
