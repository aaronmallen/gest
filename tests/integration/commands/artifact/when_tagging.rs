use predicates::prelude::*;

use crate::support::helpers::GestCmd;

fn create_artifact(env: &GestCmd) -> String {
  let output = env
    .run(["artifact", "create", "--title", "Tag Test", "--body", "body"])
    .output()
    .expect("failed to create artifact");
  assert!(output.status.success());
  let stdout = String::from_utf8_lossy(&output.stdout);
  stdout.trim().split_whitespace().last().unwrap().to_string()
}

#[test]
fn it_adds_a_tag_to_an_artifact() {
  let env = GestCmd::new();
  let id = create_artifact(&env);

  env
    .run(["artifact", "tag", &id, "important"])
    .assert()
    .success()
    .stdout(predicate::str::contains("Tagged artifact"));
}

#[test]
fn it_shows_added_tag_in_json() {
  let env = GestCmd::new();
  let id = create_artifact(&env);

  env.run(["artifact", "tag", &id, "rust"]).assert().success();

  let output = env
    .run(["artifact", "show", &id, "--json"])
    .output()
    .expect("failed to run show");

  assert!(output.status.success());
  let stdout = String::from_utf8_lossy(&output.stdout);
  let json: serde_json::Value = serde_json::from_str(&stdout).expect("expected valid JSON");
  let tags = json["tags"].as_array().expect("tags should be an array");
  assert!(
    tags.contains(&serde_json::json!("rust")),
    "expected 'rust' tag in {tags:?}"
  );
}

#[test]
fn it_adds_multiple_tags() {
  let env = GestCmd::new();
  let id = create_artifact(&env);

  env.run(["artifact", "tag", &id, "rust", "cli"]).assert().success();

  let output = env
    .run(["artifact", "show", &id, "--json"])
    .output()
    .expect("failed to run show");

  assert!(output.status.success());
  let stdout = String::from_utf8_lossy(&output.stdout);
  let json: serde_json::Value = serde_json::from_str(&stdout).expect("expected valid JSON");
  let tags = json["tags"].as_array().expect("tags should be an array");
  assert!(tags.contains(&serde_json::json!("rust")));
  assert!(tags.contains(&serde_json::json!("cli")));
}
