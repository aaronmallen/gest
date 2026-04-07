use crate::support::helpers::GestCmd;

#[test]
fn it_creates_an_artifact() {
  let g = GestCmd::new();
  let output = g
    .cmd()
    .args(["artifact", "create", "My spec", "--body", "The body."])
    .output()
    .expect("artifact create failed to run");

  assert!(output.status.success(), "artifact create exited non-zero");
  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(stdout.contains("created artifact"), "got: {stdout}");
  assert!(stdout.contains("My spec"), "got: {stdout}");
}

#[test]
fn it_lists_artifacts() {
  let g = GestCmd::new();
  g.create_artifact("Listable artifact", "body");

  let output = g
    .cmd()
    .args(["artifact", "list"])
    .output()
    .expect("artifact list failed to run");

  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(stdout.contains("Listable artifact"), "got: {stdout}");
}

#[test]
fn it_shows_an_artifact_by_id() {
  let g = GestCmd::new();
  let id = g.create_artifact("Showable artifact", "detailed body text");

  let output = g
    .cmd()
    .args(["artifact", "show", &id])
    .output()
    .expect("artifact show failed to run");

  assert!(output.status.success(), "artifact show exited non-zero");
  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(stdout.contains("Showable artifact"), "got: {stdout}");
}

#[test]
fn it_adds_a_note_via_body_flag() {
  let g = GestCmd::new();
  let id = g.create_artifact("Notable artifact", "body");

  let output = g
    .cmd()
    .args(["artifact", "note", "add", &id, "-b", "first note body"])
    .output()
    .expect("artifact note add failed to run");

  assert!(
    output.status.success(),
    "artifact note add exited non-zero: {}",
    String::from_utf8_lossy(&output.stderr)
  );
  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(stdout.contains("added note"), "got: {stdout}");
}

#[test]
fn it_rejects_a_positional_note_body() {
  let g = GestCmd::new();
  let id = g.create_artifact("Notable artifact", "body");

  let output = g
    .cmd()
    .args(["artifact", "note", "add", &id, "positional body"])
    .output()
    .expect("artifact note add failed to run");

  assert!(
    !output.status.success(),
    "artifact note add should reject positional body, got success"
  );
}

fn add_note_and_get_id(g: &GestCmd, artifact_id: &str, body: &str) -> String {
  let output = g
    .cmd()
    .args(["artifact", "note", "add", artifact_id, "-b", body, "--quiet"])
    .output()
    .expect("artifact note add failed to run");
  assert!(
    output.status.success(),
    "artifact note add exited non-zero: {}",
    String::from_utf8_lossy(&output.stderr)
  );
  String::from_utf8_lossy(&output.stdout).trim().to_string()
}

#[test]
fn it_shows_a_note_by_id() {
  let g = GestCmd::new();
  let artifact_id = g.create_artifact("Notable artifact", "body");
  let note_id = add_note_and_get_id(&g, &artifact_id, "showable body");

  let output = g
    .cmd()
    .args(["artifact", "note", "show", &note_id])
    .output()
    .expect("artifact note show failed to run");

  assert!(
    output.status.success(),
    "artifact note show exited non-zero: {}",
    String::from_utf8_lossy(&output.stderr)
  );
  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(stdout.contains("showable body"), "got: {stdout}");
}

#[test]
fn it_shows_a_note_as_json() {
  let g = GestCmd::new();
  let artifact_id = g.create_artifact("Notable artifact", "body");
  let note_id = add_note_and_get_id(&g, &artifact_id, "json body");

  let output = g
    .cmd()
    .args(["artifact", "note", "show", &note_id, "--json"])
    .output()
    .expect("artifact note show --json failed to run");

  assert!(output.status.success(), "artifact note show --json exited non-zero");
  let stdout = String::from_utf8_lossy(&output.stdout);
  let parsed: serde_json::Value = serde_json::from_str(&stdout).expect("output should be valid JSON");
  assert_eq!(parsed["body"].as_str(), Some("json body"));
}

#[test]
fn it_updates_a_note_body() {
  let g = GestCmd::new();
  let artifact_id = g.create_artifact("Notable artifact", "body");
  let note_id = add_note_and_get_id(&g, &artifact_id, "original body");

  let output = g
    .cmd()
    .args(["artifact", "note", "update", &note_id, "-b", "updated body"])
    .output()
    .expect("artifact note update failed to run");

  assert!(
    output.status.success(),
    "artifact note update exited non-zero: {}",
    String::from_utf8_lossy(&output.stderr)
  );
  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(stdout.contains("updated note"), "got: {stdout}");

  let show_output = g
    .cmd()
    .args(["artifact", "note", "show", &note_id])
    .output()
    .expect("artifact note show failed to run");
  let show_stdout = String::from_utf8_lossy(&show_output.stdout);
  assert!(show_stdout.contains("updated body"), "got: {show_stdout}");
}

#[test]
fn it_updates_a_note_body_as_json() {
  let g = GestCmd::new();
  let artifact_id = g.create_artifact("Notable artifact", "body");
  let note_id = add_note_and_get_id(&g, &artifact_id, "original body");

  let output = g
    .cmd()
    .args([
      "artifact",
      "note",
      "update",
      &note_id,
      "-b",
      "updated via json",
      "--json",
    ])
    .output()
    .expect("artifact note update --json failed to run");

  assert!(output.status.success(), "artifact note update --json exited non-zero");
  let stdout = String::from_utf8_lossy(&output.stdout);
  let parsed: serde_json::Value = serde_json::from_str(&stdout).expect("output should be valid JSON");
  assert_eq!(parsed["body"].as_str(), Some("updated via json"));
}
