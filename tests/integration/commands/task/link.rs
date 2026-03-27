use predicates::prelude::*;

use crate::support::helpers::GestCmd;

/// Extract the 8-character ID from a confirmation line like "Created task <id>" or "Created artifact <id>".
fn extract_id(output: &str, entity: &str) -> String {
  let prefix = format!("Created {entity} ");
  output
    .trim()
    .strip_prefix(&prefix)
    .unwrap_or_else(|| panic!("expected 'Created {entity} <id>' but got: {output}"))
    .to_string()
}

#[test]
fn it_links_two_tasks() {
  let env = GestCmd::new();

  let out1 = env
    .run(["task", "create", "Source Task", "-d", "source"])
    .output()
    .expect("failed to create source task");
  let source_id = extract_id(&String::from_utf8_lossy(&out1.stdout), "task");

  let out2 = env
    .run(["task", "create", "Target Task", "-d", "target"])
    .output()
    .expect("failed to create target task");
  let target_id = extract_id(&String::from_utf8_lossy(&out2.stdout), "task");

  env
    .run(["task", "link", &source_id, "relates-to", &target_id])
    .assert()
    .success()
    .stdout(predicate::str::contains("Linked"));

  // Verify link appears in show output
  let show_output = env
    .run(["task", "show", &source_id, "--json"])
    .output()
    .expect("failed to show task");
  let stdout = String::from_utf8_lossy(&show_output.stdout);
  let json: serde_json::Value = serde_json::from_str(&stdout).expect("invalid JSON");
  let links = json["links"].as_array().expect("links should be an array");
  assert_eq!(links.len(), 1, "Should have exactly one link");
  assert_eq!(links[0]["rel"], "relates-to");
}

#[test]
fn it_links_a_task_to_an_artifact() {
  let env = GestCmd::new();

  let task_out = env
    .run(["task", "create", "Link Source", "-d", "will link to artifact"])
    .output()
    .expect("failed to create task");
  let task_id = extract_id(&String::from_utf8_lossy(&task_out.stdout), "task");

  let artifact_out = env
    .run(["artifact", "create", "-t", "My Artifact", "-b", "artifact body"])
    .output()
    .expect("failed to create artifact");
  let artifact_id = extract_id(&String::from_utf8_lossy(&artifact_out.stdout), "artifact");

  env
    .run(["task", "link", &task_id, "relates-to", &artifact_id, "--artifact"])
    .assert()
    .success()
    .stdout(predicate::str::contains("Linked"));

  // Verify link appears in show output
  let show_output = env
    .run(["task", "show", &task_id, "--json"])
    .output()
    .expect("failed to show task");
  let stdout = String::from_utf8_lossy(&show_output.stdout);
  let json: serde_json::Value = serde_json::from_str(&stdout).expect("invalid JSON");
  let links = json["links"].as_array().expect("links should be an array");
  assert_eq!(links.len(), 1);
  assert!(links[0]["ref"].as_str().unwrap().starts_with("artifacts/"));
}
