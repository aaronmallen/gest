use crate::support::helpers::GestCmd;

#[test]
fn it_aliases_remove_to_rm() {
  let g = GestCmd::new();
  let iter_id = g.create_iteration("rm sprint");
  let task_id = g.create_task("rm target");
  g.cmd()
    .args(["iteration", "add", &iter_id, &task_id])
    .assert()
    .success();

  let output = g
    .cmd()
    .args(["iteration", "rm", &iter_id, &task_id])
    .output()
    .expect("iteration rm failed to run");

  assert!(
    output.status.success(),
    "iteration rm (alias) should succeed: {}",
    String::from_utf8_lossy(&output.stderr)
  );
}

#[test]
fn it_removes_task_from_iteration() {
  let g = GestCmd::new();
  let iter_id = g.create_iteration("remove sprint");
  let task_id = g.create_task("attachable task");

  g.cmd()
    .args(["iteration", "add", &iter_id, &task_id])
    .assert()
    .success();

  let output = g
    .cmd()
    .args(["iteration", "remove", &iter_id, &task_id])
    .output()
    .expect("iteration remove failed");

  assert!(
    output.status.success(),
    "iteration remove should succeed: {}",
    String::from_utf8_lossy(&output.stderr)
  );
}

#[test]
fn it_errors_on_unattached_task() {
  let g = GestCmd::new();
  let iter_id = g.create_iteration("empty sprint");

  // A bogus task id that doesn't resolve should produce an error.
  let output = g
    .cmd()
    .args(["iteration", "remove", &iter_id, "zzzzzzzz"])
    .output()
    .expect("iteration remove failed");

  assert!(
    !output.status.success(),
    "removing an unresolved task should exit non-zero"
  );
}
