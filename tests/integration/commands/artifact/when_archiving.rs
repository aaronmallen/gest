use predicates::prelude::*;

use crate::support::helpers::GestCmd;

fn create_artifact(env: &GestCmd) -> String {
  let output = env
    .run(["artifact", "create", "--title", "Archive Test", "--body", "body"])
    .output()
    .expect("failed to create artifact");
  assert!(output.status.success());
  let stdout = String::from_utf8_lossy(&output.stdout);
  stdout.trim().split_whitespace().last().unwrap().to_string()
}

#[test]
fn it_archives_an_artifact() {
  let env = GestCmd::new();
  let id = create_artifact(&env);

  env
    .run(["artifact", "archive", &id])
    .assert()
    .success()
    .stdout(predicate::str::contains("Archived artifact"));
}

#[test]
fn it_sets_archived_at_timestamp() {
  let env = GestCmd::new();
  let id = create_artifact(&env);

  env.run(["artifact", "archive", &id]).assert().success();

  let output = env
    .run(["artifact", "show", &id, "--json"])
    .output()
    .expect("failed to run show");

  assert!(output.status.success());
  let stdout = String::from_utf8_lossy(&output.stdout);
  let json: serde_json::Value = serde_json::from_str(&stdout).expect("expected valid JSON");
  assert!(!json["archived_at"].is_null(), "expected archived_at to be set");
}

#[test]
fn it_hides_archived_artifacts_from_default_list() {
  let env = GestCmd::new();
  let id = create_artifact(&env);

  env.run(["artifact", "archive", &id]).assert().success();

  env
    .run(["artifact", "list"])
    .assert()
    .success()
    .stdout(predicate::str::contains("Archive Test").not());
}
