use predicates::prelude::*;

use crate::support::helpers::GestCmd;

/// Extract the 8-character task ID from create output like "Created task abcdefgh".
fn extract_task_id(create_output: &str) -> String {
  create_output
    .trim()
    .strip_prefix("Created task ")
    .expect("expected 'Created task <id>' output")
    .to_string()
}

#[test]
fn it_shows_a_task_by_id() {
  let env = GestCmd::new();

  let output = env
    .run(["task", "create", "Show Me", "-d", "details here"])
    .output()
    .expect("failed to create task");
  let id = extract_task_id(&String::from_utf8_lossy(&output.stdout));

  env
    .run(["task", "show", &id])
    .assert()
    .success()
    .stdout(predicate::str::contains("Show Me"));
}

#[test]
fn it_shows_a_task_as_json() {
  let env = GestCmd::new();

  let output = env
    .run(["task", "create", "JSON Show", "-d", "json details"])
    .output()
    .expect("failed to create task");
  let id = extract_task_id(&String::from_utf8_lossy(&output.stdout));

  env
    .run(["task", "show", &id, "--json"])
    .assert()
    .success()
    .stdout(predicate::str::contains("\"title\""))
    .stdout(predicate::str::contains("JSON Show"));
}

#[test]
fn it_includes_expected_json_fields() {
  let env = GestCmd::new();

  let output = env
    .run(["task", "create", "Fields Task", "-d", "check fields"])
    .output()
    .expect("failed to create task");
  let id = extract_task_id(&String::from_utf8_lossy(&output.stdout));

  let show_output = env
    .run(["task", "show", &id, "--json"])
    .output()
    .expect("failed to show task");
  let stdout = String::from_utf8_lossy(&show_output.stdout);
  let json: serde_json::Value = serde_json::from_str(&stdout).expect("invalid JSON output");

  assert!(json.get("id").is_some(), "JSON should contain 'id'");
  assert!(json.get("title").is_some(), "JSON should contain 'title'");
  assert!(json.get("description").is_some(), "JSON should contain 'description'");
  assert!(json.get("status").is_some(), "JSON should contain 'status'");
  assert!(json.get("tags").is_some(), "JSON should contain 'tags'");
  assert!(json.get("links").is_some(), "JSON should contain 'links'");
  assert!(json.get("created_at").is_some(), "JSON should contain 'created_at'");
  assert!(json.get("updated_at").is_some(), "JSON should contain 'updated_at'");
}
