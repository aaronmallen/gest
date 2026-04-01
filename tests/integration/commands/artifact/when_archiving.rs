use predicates::prelude::*;

use crate::support::helpers::GestCmd;

#[test]
fn it_archives_an_artifact() {
  let env = GestCmd::new();

  let id = env.create_artifact("Archive Me", "content to archive");

  env
    .cmd()
    .args(["artifact", "archive", &id])
    .assert()
    .success()
    .stdout(predicate::str::contains("Archived artifact"));

  // After archiving, the artifact should not appear in the default list.
  env
    .cmd()
    .args(["artifact", "list"])
    .assert()
    .success()
    .stdout(predicate::str::contains("no artifacts found"));
}
