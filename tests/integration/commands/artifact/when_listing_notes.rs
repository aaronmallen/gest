use crate::support::helpers::GestCmd;

fn add_note(g: &GestCmd, artifact_id: &str, body: &str) -> String {
  let output = g
    .cmd()
    .args(["artifact", "note", "add", artifact_id, "-b", body, "--quiet"])
    .output()
    .expect("artifact note add failed");
  assert!(output.status.success(), "artifact note add should succeed");
  String::from_utf8_lossy(&output.stdout).trim().to_string()
}

#[test]
fn it_lists_notes_in_order() {
  let g = GestCmd::new();
  let artifact_id = g.create_artifact("Listable notes", "body");

  add_note(&g, &artifact_id, "first note");
  add_note(&g, &artifact_id, "second note");

  let output = g
    .cmd()
    .args(["artifact", "note", "list", &artifact_id])
    .output()
    .expect("artifact note list failed");
  assert!(output.status.success(), "artifact note list should succeed");
  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(stdout.contains("first note"), "missing first: {stdout}");
  assert!(stdout.contains("second note"), "missing second: {stdout}");
}

#[test]
fn it_shows_empty_when_no_notes() {
  let g = GestCmd::new();
  let artifact_id = g.create_artifact("No notes yet", "body");

  let output = g
    .cmd()
    .args(["artifact", "note", "list", &artifact_id])
    .output()
    .expect("artifact note list failed");
  assert!(output.status.success(), "artifact note list should succeed");
  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(
    stdout.to_lowercase().contains("no notes") || stdout.trim().is_empty() || stdout.contains("0 notes"),
    "expected empty-notes indicator, got: {stdout}"
  );
}

#[test]
fn it_lists_notes_as_json() {
  let g = GestCmd::new();
  let artifact_id = g.create_artifact("json notable", "body");
  add_note(&g, &artifact_id, "json body");

  let output = g
    .cmd()
    .args(["artifact", "note", "list", &artifact_id, "--json"])
    .output()
    .expect("artifact note list --json failed");

  assert!(output.status.success(), "artifact note list --json should succeed");
  let stdout = String::from_utf8_lossy(&output.stdout);
  let parsed: serde_json::Value = serde_json::from_str(&stdout).expect("valid json");
  let arr = parsed.as_array().expect("array");
  assert!(!arr.is_empty(), "should contain at least one note, got: {stdout}");
}

#[test]
fn it_lists_notes_as_quiet_short_ids() {
  let g = GestCmd::new();
  let artifact_id = g.create_artifact("quiet notable", "body");
  let first = add_note(&g, &artifact_id, "first note");
  let second = add_note(&g, &artifact_id, "second note");

  let output = g
    .cmd()
    .args(["artifact", "note", "list", &artifact_id, "--quiet"])
    .output()
    .expect("artifact note list --quiet failed");

  assert!(output.status.success(), "artifact note list --quiet should succeed");
  let stdout = String::from_utf8_lossy(&output.stdout);
  let lines: Vec<&str> = stdout.lines().collect();
  assert_eq!(lines.len(), 2, "expected one id per line, got: {stdout}");
  assert!(lines.contains(&first.as_str()), "missing first id {first}: {stdout}");
  assert!(lines.contains(&second.as_str()), "missing second id {second}: {stdout}");
}
