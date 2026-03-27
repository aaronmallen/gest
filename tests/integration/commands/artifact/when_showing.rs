use predicates::prelude::*;

use crate::support::helpers::GestCmd;

fn create_artifact(env: &GestCmd) -> String {
  let output = env
    .run([
      "artifact",
      "create",
      "--title",
      "Show Test",
      "--body",
      "show body",
      "--type",
      "spec",
    ])
    .output()
    .expect("failed to create artifact");
  assert!(output.status.success());
  let stdout = String::from_utf8_lossy(&output.stdout);
  stdout.trim().split_whitespace().last().unwrap().to_string()
}

#[test]
fn it_shows_an_artifact_by_id() {
  let env = GestCmd::new();
  let id = create_artifact(&env);

  env
    .run(["artifact", "show", &id])
    .assert()
    .success()
    .stdout(predicate::str::contains("Show Test"));
}

#[test]
fn it_shows_an_artifact_as_json() {
  let env = GestCmd::new();
  let id = create_artifact(&env);

  let output = env
    .run(["artifact", "show", &id, "--json"])
    .output()
    .expect("failed to run show --json");

  assert!(output.status.success());
  let stdout = String::from_utf8_lossy(&output.stdout);
  let json: serde_json::Value = serde_json::from_str(&stdout).expect("expected valid JSON");

  assert!(json.get("id").is_some(), "JSON should have 'id' key");
  assert!(json.get("title").is_some(), "JSON should have 'title' key");
  assert!(json.get("type").is_some(), "JSON should have 'type' key");
  assert!(json.get("tags").is_some(), "JSON should have 'tags' key");
  assert!(json.get("body").is_some(), "JSON should have 'body' key");
  assert!(json.get("metadata").is_some(), "JSON should have 'metadata' key");
  assert!(json.get("created_at").is_some(), "JSON should have 'created_at' key");
  assert!(json.get("updated_at").is_some(), "JSON should have 'updated_at' key");
}

#[test]
fn it_includes_correct_values_in_json() {
  let env = GestCmd::new();
  let id = create_artifact(&env);

  let output = env
    .run(["artifact", "show", &id, "--json"])
    .output()
    .expect("failed to run show --json");

  assert!(output.status.success());
  let stdout = String::from_utf8_lossy(&output.stdout);
  let json: serde_json::Value = serde_json::from_str(&stdout).expect("expected valid JSON");

  assert_eq!(json["title"], "Show Test");
  assert_eq!(json["type"], "spec");
  assert_eq!(json["body"], "show body");
}

#[test]
fn it_fails_for_nonexistent_id() {
  let env = GestCmd::new();

  env.run(["artifact", "show", "nonexistent"]).assert().failure();
}
