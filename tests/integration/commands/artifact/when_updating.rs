use predicates::prelude::*;

use crate::support::helpers::GestCmd;

fn create_artifact(env: &GestCmd) -> String {
  let output = env
    .run([
      "artifact",
      "create",
      "--title",
      "Update Test",
      "--body",
      "original body",
    ])
    .output()
    .expect("failed to create artifact");
  assert!(output.status.success());
  let stdout = String::from_utf8_lossy(&output.stdout);
  stdout.trim().split_whitespace().last().unwrap().to_string()
}

#[test]
fn it_updates_body_via_flag() {
  let env = GestCmd::new();
  let id = create_artifact(&env);

  env
    .run(["artifact", "update", &id, "--body", "new body"])
    .assert()
    .success()
    .stdout(predicate::str::contains("Updated artifact"));
}

#[test]
fn it_persists_updated_body() {
  let env = GestCmd::new();
  let id = create_artifact(&env);

  env
    .run(["artifact", "update", &id, "--body", "changed body"])
    .assert()
    .success();

  let output = env
    .run(["artifact", "show", &id, "--json"])
    .output()
    .expect("failed to run show");

  assert!(output.status.success());
  let stdout = String::from_utf8_lossy(&output.stdout);
  let json: serde_json::Value = serde_json::from_str(&stdout).expect("expected valid JSON");
  assert_eq!(json["body"], "changed body");
}

#[test]
fn it_updates_title() {
  let env = GestCmd::new();
  let id = create_artifact(&env);

  env
    .run(["artifact", "update", &id, "--title", "New Title"])
    .assert()
    .success();

  let output = env
    .run(["artifact", "show", &id, "--json"])
    .output()
    .expect("failed to run show");

  assert!(output.status.success());
  let stdout = String::from_utf8_lossy(&output.stdout);
  let json: serde_json::Value = serde_json::from_str(&stdout).expect("expected valid JSON");
  assert_eq!(json["title"], "New Title");
}
