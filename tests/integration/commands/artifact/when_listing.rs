use predicates::prelude::*;

use crate::support::helpers::GestCmd;

#[test]
fn it_returns_cleanly_with_no_items() {
  let env = GestCmd::new();

  env
    .run(["artifact", "list"])
    .assert()
    .success()
    .stdout(predicate::str::contains("No artifacts found."));
}

#[test]
fn it_lists_artifacts_after_creating_one() {
  let env = GestCmd::new();

  env
    .run(["artifact", "create", "--title", "List Test", "--body", "body"])
    .assert()
    .success();

  env
    .run(["artifact", "list"])
    .assert()
    .success()
    .stdout(predicate::str::contains("List Test"));
}

#[test]
fn it_outputs_json_with_empty_list() {
  let env = GestCmd::new();

  let output = env
    .run(["artifact", "list", "--json"])
    .output()
    .expect("failed to run list --json");

  assert!(output.status.success());
  let stdout = String::from_utf8_lossy(&output.stdout);
  let json: serde_json::Value = serde_json::from_str(&stdout).expect("expected valid JSON");
  assert!(json.is_array());
  assert_eq!(json.as_array().unwrap().len(), 0);
}

#[test]
fn it_outputs_json_with_items() {
  let env = GestCmd::new();

  env
    .run(["artifact", "create", "--title", "JSON List Item", "--body", "body"])
    .assert()
    .success();

  let output = env
    .run(["artifact", "list", "--json"])
    .output()
    .expect("failed to run list --json");

  assert!(output.status.success());
  let stdout = String::from_utf8_lossy(&output.stdout);
  let json: serde_json::Value = serde_json::from_str(&stdout).expect("expected valid JSON");
  let arr = json.as_array().expect("expected JSON array");
  assert_eq!(arr.len(), 1);
  assert_eq!(arr[0]["title"], "JSON List Item");
}
