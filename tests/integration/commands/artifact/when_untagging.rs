use predicates::prelude::*;

use crate::support::helpers::GestCmd;

#[test]
fn it_untags_an_artifact() {
  let env = GestCmd::new();

  let id = env.create_artifact("Untag Me", "artifact content");

  // First tag the artifact.
  env
    .cmd()
    .args(["artifact", "tag", &id, "keep", "remove"])
    .assert()
    .success();

  // Now untag one of the tags.
  env
    .cmd()
    .args(["artifact", "untag", &id, "remove"])
    .assert()
    .success()
    .stdout(predicate::str::contains("Untagged artifact"));

  // Verify the removed tag is gone but the kept tag remains.
  env
    .cmd()
    .args(["artifact", "show", &id])
    .assert()
    .success()
    .stdout(predicate::str::contains("keep"))
    .stdout(predicate::str::contains("#remove").not());
}
