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
