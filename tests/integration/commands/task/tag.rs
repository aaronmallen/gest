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
fn it_adds_a_tag_to_a_task() {
  let env = GestCmd::new();

  let output = env
    .run(["task", "create", "Tag Me", "-d", "to be tagged"])
    .output()
    .expect("failed to create task");
  let id = extract_task_id(&String::from_utf8_lossy(&output.stdout));

  env
    .run(["task", "tag", &id, "important"])
    .assert()
    .success()
    .stdout(predicate::str::contains("Tagged task"));

  // Verify tag appears in show
  let show_output = env
    .run(["task", "show", &id, "--json"])
    .output()
    .expect("failed to show task");
  let stdout = String::from_utf8_lossy(&show_output.stdout);
  let json: serde_json::Value = serde_json::from_str(&stdout).expect("invalid JSON");
  let tags = json["tags"].as_array().expect("tags should be an array");
  assert!(
    tags.iter().any(|t| t == "important"),
    "Tag 'important' should be present"
  );
}

#[test]
fn it_adds_multiple_tags() {
  let env = GestCmd::new();

  let output = env
    .run(["task", "create", "Multi Tag", "-d", "multi tags"])
    .output()
    .expect("failed to create task");
  let id = extract_task_id(&String::from_utf8_lossy(&output.stdout));

  env.run(["task", "tag", &id, "rust", "cli"]).assert().success();

  let show_output = env
    .run(["task", "show", &id, "--json"])
    .output()
    .expect("failed to show task");
  let stdout = String::from_utf8_lossy(&show_output.stdout);
  let json: serde_json::Value = serde_json::from_str(&stdout).expect("invalid JSON");
  let tags = json["tags"].as_array().expect("tags should be an array");
  assert_eq!(tags.len(), 2);
}
