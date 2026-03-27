use predicates::prelude::*;

use crate::support::helpers::GestCmd;

#[test]
fn it_exits_cleanly_with_no_matches() {
  let env = GestCmd::new();

  env
    .run(["search", "nonexistentxyz123"])
    .assert()
    .success()
    .stdout(predicate::str::contains("No results found for"));
}

#[test]
fn it_returns_empty_json_with_no_matches() {
  let env = GestCmd::new();

  let output = env
    .run(["search", "--json", "nonexistentxyz123"])
    .output()
    .expect("failed to run search --json");

  assert!(output.status.success(), "expected success exit code");

  let stdout = String::from_utf8_lossy(&output.stdout);
  let json: serde_json::Value = serde_json::from_str(&stdout).expect("expected valid JSON output");

  assert_eq!(json["tasks"], serde_json::json!([]));
  assert_eq!(json["artifacts"], serde_json::json!([]));
}
