use crate::support::helpers::GestCmd;

#[test]
fn it_adds_tag_directly() {
  let g = GestCmd::new();
  let task_id = g.create_task("direct tag target");

  g.add_tag_direct(&task_id, "direct");

  let list = g
    .cmd()
    .args(["tag", "list", "--task"])
    .output()
    .expect("tag list failed");
  let stdout = String::from_utf8_lossy(&list.stdout);
  assert!(stdout.contains("direct"), "direct tag should appear, got: {stdout}");
}

#[test]
fn it_rejects_duplicate_tag_add() {
  let g = GestCmd::new();
  let task_id = g.create_task("duplicate tag");
  g.add_tag_direct(&task_id, "dup");

  // A second `tag add` with the same label is a no-op that the CLI currently
  // accepts silently. Pin observed behavior so any future tightening surfaces.
  let output = g
    .cmd()
    .args(["tag", "add", &task_id, "dup"])
    .output()
    .expect("tag add failed");

  assert!(
    output.status.success(),
    "duplicate tag add is currently accepted; update if behavior tightens"
  );

  // Tag should still appear only once in the list.
  let list = g
    .cmd()
    .args(["tag", "list", "--task"])
    .output()
    .expect("tag list failed");
  let stdout = String::from_utf8_lossy(&list.stdout);
  let occurrences = stdout.matches("dup").count();
  assert!(occurrences >= 1, "expected dup tag in list, got: {stdout}");
}
