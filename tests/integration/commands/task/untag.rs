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
fn it_removes_a_tag_from_a_task() {
  let env = GestCmd::new();

  let output = env
    .run([
      "task",
      "create",
      "Untag Me",
      "-d",
      "to be untagged",
      "--tags",
      "removeme,keepme",
    ])
    .output()
    .expect("failed to create task");
  let id = extract_task_id(&String::from_utf8_lossy(&output.stdout));

  env
    .run(["task", "untag", &id, "removeme"])
    .assert()
    .success()
    .stdout(predicate::str::contains("Untagged task"));

  // Verify the tag was removed
  let show_output = env
    .run(["task", "show", &id, "--json"])
    .output()
    .expect("failed to show task");
  let stdout = String::from_utf8_lossy(&show_output.stdout);
  let json: serde_json::Value = serde_json::from_str(&stdout).expect("invalid JSON");
  let tags = json["tags"].as_array().expect("tags should be an array");
  assert!(!tags.iter().any(|t| t == "removeme"), "Tag 'removeme' should be gone");
  assert!(tags.iter().any(|t| t == "keepme"), "Tag 'keepme' should remain");
}

#[test]
fn it_removes_all_tags() {
  let env = GestCmd::new();

  let output = env
    .run(["task", "create", "Strip Tags", "-d", "strip all", "--tags", "a,b"])
    .output()
    .expect("failed to create task");
  let id = extract_task_id(&String::from_utf8_lossy(&output.stdout));

  env.run(["task", "untag", &id, "a", "b"]).assert().success();

  let show_output = env
    .run(["task", "show", &id, "--json"])
    .output()
    .expect("failed to show task");
  let stdout = String::from_utf8_lossy(&show_output.stdout);
  let json: serde_json::Value = serde_json::from_str(&stdout).expect("invalid JSON");
  let tags = json["tags"].as_array().expect("tags should be an array");
  assert!(tags.is_empty(), "All tags should have been removed");
}
