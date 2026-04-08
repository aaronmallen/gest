use crate::support::helpers::GestCmd;

#[test]
fn it_lists_open_tasks() {
  let g = GestCmd::new();
  g.create_task("Listed task");

  let output = g
    .cmd()
    .args(["task", "list"])
    .output()
    .expect("task list failed to run");

  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(stdout.contains("Listed task"), "got: {stdout}");
}

#[test]
fn it_respects_no_pager() {
  let g = GestCmd::new();
  g.create_task("Pager-bypassed task");

  let output = g
    .cmd()
    .args(["--no-pager", "task", "list"])
    .output()
    .expect("task list --no-pager failed to run");

  assert!(output.status.success(), "task list --no-pager exited non-zero");
  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(stdout.contains("Pager-bypassed task"), "got: {stdout}");
}
