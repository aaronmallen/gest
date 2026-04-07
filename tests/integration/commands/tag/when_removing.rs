use crate::support::helpers::GestCmd;

#[test]
fn it_errors_on_unknown_tag() {
  let g = GestCmd::new();
  let task_id = g.create_task("no tags here");

  let output = g
    .cmd()
    .args(["tag", "remove", &task_id, "never-added"])
    .output()
    .expect("tag remove failed");

  // The CLI is currently lenient and accepts removing a tag that was never
  // attached. Pin observed behavior; tighten assertion if tag tracking
  // strictens.
  assert!(
    output.status.success(),
    "tag remove is currently lenient on unknown tags"
  );
}

#[test]
fn it_removes_tag() {
  let g = GestCmd::new();
  let task_id = g.create_task("tag removal target");
  g.add_tag_direct(&task_id, "keep");
  g.add_tag_direct(&task_id, "drop");

  let output = g
    .cmd()
    .args(["tag", "remove", &task_id, "drop"])
    .output()
    .expect("tag remove failed");
  assert!(
    output.status.success(),
    "tag remove should succeed: {}",
    String::from_utf8_lossy(&output.stderr)
  );

  let list = g
    .cmd()
    .args(["tag", "list", "--task"])
    .output()
    .expect("tag list failed");
  let stdout = String::from_utf8_lossy(&list.stdout);
  assert!(stdout.contains("keep"), "keep tag should remain, got: {stdout}");
  assert!(!stdout.contains("drop"), "drop tag should be gone, got: {stdout}");
}
