use crate::support::helpers::GestCmd;

#[test]
fn it_creates_a_task() {
  let g = GestCmd::new();
  let output = g
    .cmd()
    .args(["task", "create", "Hello task"])
    .output()
    .expect("task create failed to run");

  assert!(output.status.success(), "task create exited non-zero");
  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(stdout.contains("created task"), "got: {stdout}");
  assert!(stdout.contains("Hello task"), "got: {stdout}");
}

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
fn it_shows_a_task_by_id() {
  let g = GestCmd::new();
  let id = g.create_task("Detail task");

  let output = g
    .cmd()
    .args(["task", "show", &id])
    .output()
    .expect("task show failed to run");

  assert!(output.status.success(), "task show exited non-zero");
  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(stdout.contains("Detail task"), "got: {stdout}");
}

#[test]
fn it_updates_task_status_to_done() {
  let g = GestCmd::new();
  let id = g.create_task("Finishable task");

  g.cmd()
    .args(["task", "update", &id, "--status", "done"])
    .assert()
    .success();

  let output = g.cmd().args(["task", "show", &id]).output().expect("task show failed");
  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(stdout.contains("done"), "task should be done, got: {stdout}");
}
