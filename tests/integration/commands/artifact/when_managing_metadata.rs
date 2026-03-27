use predicates::prelude::*;

use crate::support::helpers::GestCmd;

fn create_artifact(env: &GestCmd) -> String {
  let output = env
    .run(["artifact", "create", "--title", "Meta Test", "--body", "body"])
    .output()
    .expect("failed to create artifact");
  assert!(output.status.success());
  let stdout = String::from_utf8_lossy(&output.stdout);
  stdout.trim().split_whitespace().last().unwrap().to_string()
}

#[test]
fn it_sets_a_metadata_key_value() {
  let env = GestCmd::new();
  let id = create_artifact(&env);

  env
    .run(["artifact", "meta", "set", &id, "team", "backend"])
    .assert()
    .success()
    .stdout(predicate::str::contains("Updated artifact"));
}

#[test]
fn it_gets_a_metadata_value() {
  let env = GestCmd::new();
  let id = create_artifact(&env);

  env
    .run(["artifact", "meta", "set", &id, "team", "backend"])
    .assert()
    .success();

  env
    .run(["artifact", "meta", "get", &id, "team"])
    .assert()
    .success()
    .stdout(predicate::str::contains("backend"));
}

#[test]
fn it_overwrites_existing_metadata() {
  let env = GestCmd::new();
  let id = create_artifact(&env);

  env
    .run(["artifact", "meta", "set", &id, "team", "frontend"])
    .assert()
    .success();

  env
    .run(["artifact", "meta", "set", &id, "team", "backend"])
    .assert()
    .success();

  env
    .run(["artifact", "meta", "get", &id, "team"])
    .assert()
    .success()
    .stdout(predicate::str::contains("backend"));
}

#[test]
fn it_reflects_metadata_in_show_json() {
  let env = GestCmd::new();
  let id = create_artifact(&env);

  env
    .run(["artifact", "meta", "set", &id, "priority", "high"])
    .assert()
    .success();

  let output = env
    .run(["artifact", "show", &id, "--json"])
    .output()
    .expect("failed to run show");

  assert!(output.status.success());
  let stdout = String::from_utf8_lossy(&output.stdout);
  let json: serde_json::Value = serde_json::from_str(&stdout).expect("expected valid JSON");
  assert_eq!(json["metadata"]["priority"], "high");
}

#[test]
fn it_fails_for_missing_metadata_key() {
  let env = GestCmd::new();
  let id = create_artifact(&env);

  env
    .run(["artifact", "meta", "get", &id, "nonexistent"])
    .assert()
    .failure();
}
