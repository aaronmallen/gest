use crate::support::helpers::GestCmd;

fn create_tagged_artifact(env: &GestCmd) -> String {
  let output = env
    .run([
      "artifact",
      "create",
      "--title",
      "Untag Test",
      "--body",
      "body",
      "--tags",
      "rust,cli",
    ])
    .output()
    .expect("failed to create artifact");
  assert!(output.status.success());
  let stdout = String::from_utf8_lossy(&output.stdout);
  stdout.trim().split_whitespace().last().unwrap().to_string()
}

#[test]
fn it_removes_a_tag_from_an_artifact() {
  let env = GestCmd::new();
  let id = create_tagged_artifact(&env);

  env.run(["artifact", "untag", &id, "rust"]).assert().success();

  let output = env
    .run(["artifact", "show", &id, "--json"])
    .output()
    .expect("failed to run show");

  assert!(output.status.success());
  let stdout = String::from_utf8_lossy(&output.stdout);
  let json: serde_json::Value = serde_json::from_str(&stdout).expect("expected valid JSON");
  let tags = json["tags"].as_array().expect("tags should be an array");
  assert!(
    !tags.contains(&serde_json::json!("rust")),
    "expected 'rust' to be removed"
  );
  assert!(tags.contains(&serde_json::json!("cli")), "expected 'cli' to remain");
}

#[test]
fn it_removes_all_tags() {
  let env = GestCmd::new();
  let id = create_tagged_artifact(&env);

  env.run(["artifact", "untag", &id, "rust", "cli"]).assert().success();

  let output = env
    .run(["artifact", "show", &id, "--json"])
    .output()
    .expect("failed to run show");

  assert!(output.status.success());
  let stdout = String::from_utf8_lossy(&output.stdout);
  let json: serde_json::Value = serde_json::from_str(&stdout).expect("expected valid JSON");
  let tags = json["tags"].as_array().expect("tags should be an array");
  assert!(tags.is_empty(), "expected all tags to be removed, got {tags:?}");
}
