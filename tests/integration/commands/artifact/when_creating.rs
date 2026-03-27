use predicates::prelude::*;

use crate::support::helpers::GestCmd;

#[test]
fn it_creates_an_artifact_with_body_and_title_flags() {
  let env = GestCmd::new();

  env
    .run([
      "artifact",
      "create",
      "--title",
      "My Artifact",
      "--body",
      "Some body content",
    ])
    .assert()
    .success()
    .stdout(predicate::str::contains("Created artifact"));
}

#[test]
fn it_creates_an_artifact_with_type_flag() {
  let env = GestCmd::new();

  env
    .run([
      "artifact",
      "create",
      "--title",
      "My Spec",
      "--type",
      "spec",
      "--body",
      "Spec body",
    ])
    .assert()
    .success()
    .stdout(predicate::str::contains("Created artifact"));
}

#[test]
fn it_outputs_a_short_id_on_create() {
  let env = GestCmd::new();

  let output = env
    .run(["artifact", "create", "--title", "ID Test", "--body", "body"])
    .output()
    .expect("failed to run command");

  assert!(output.status.success());
  let stdout = String::from_utf8_lossy(&output.stdout);
  // Output format: "Created artifact <short_id>"
  let id = stdout
    .trim()
    .split_whitespace()
    .last()
    .expect("expected an ID in output");
  assert!(!id.is_empty(), "expected non-empty artifact ID");
}

#[test]
fn it_creates_an_artifact_from_stdin() {
  let env = GestCmd::new();

  env
    .cmd()
    .args(["artifact", "create", "--title", "Stdin Artifact"])
    .write_stdin("Body from stdin")
    .assert()
    .success()
    .stdout(predicate::str::contains("Created artifact"));
}

#[test]
fn it_creates_an_artifact_with_tags() {
  let env = GestCmd::new();

  env
    .run([
      "artifact", "create", "--title", "Tagged", "--body", "body", "--tags", "rust,cli",
    ])
    .assert()
    .success()
    .stdout(predicate::str::contains("Created artifact"));
}

#[test]
fn it_creates_an_artifact_with_metadata() {
  let env = GestCmd::new();

  env
    .run([
      "artifact",
      "create",
      "--title",
      "With Meta",
      "--body",
      "body",
      "-m",
      "priority=high",
    ])
    .assert()
    .success()
    .stdout(predicate::str::contains("Created artifact"));
}
