use predicates::prelude::*;

use crate::support::helpers::GestCmd;

#[test]
fn it_returns_empty_for_no_match() {
  let env = GestCmd::new();

  env
    .cmd()
    .args(["search", "nonexistent_xyz_query_abc"])
    .assert()
    .success()
    .stdout(predicate::str::contains("0 results for"))
    .stdout(predicate::str::contains("try broadening your query"));
}
