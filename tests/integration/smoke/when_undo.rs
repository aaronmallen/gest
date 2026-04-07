use crate::support::helpers::GestCmd;

#[test]
fn it_undoes_a_task_creation() {
  let g = GestCmd::new();
  let id = g.create_task("Undoable task");

  // Verify the task exists
  g.cmd().args(["task", "show", &id]).assert().success();

  // Undo it
  let undo_out = g.cmd().args(["undo"]).output().expect("undo failed to run");
  assert!(undo_out.status.success(), "undo exited non-zero");
  let stdout = String::from_utf8_lossy(&undo_out.stdout);
  assert!(stdout.contains("undone"), "got: {stdout}");

  // Task should no longer be visible
  let show_out = g.cmd().args(["task", "show", &id]).output().expect("task show failed");
  assert!(!show_out.status.success(), "task show should fail after undo");
}

#[test]
fn it_errors_when_nothing_to_undo() {
  let g = GestCmd::new();
  let output = g.cmd().args(["undo"]).output().expect("undo failed to run");

  assert!(!output.status.success(), "undo should fail when there is no history");
  let stderr = String::from_utf8_lossy(&output.stderr);
  assert!(stderr.to_lowercase().contains("nothing to undo"), "stderr: {stderr}");
}
