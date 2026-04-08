use crate::support::helpers::GestCmd;

#[test]
fn it_deletes_a_standalone_task_with_dependents_and_writes_a_tombstone() {
  let g = GestCmd::new();
  let task_id = g.create_task("Doomed");
  g.attach_tag("task", &task_id, "urgent");
  let note = g
    .cmd()
    .args(["task", "note", "add", &task_id, "--body", "please keep"])
    .output()
    .expect("task note add failed to run");
  assert!(note.status.success());

  let output = g
    .cmd()
    .args(["task", "delete", &task_id, "--yes"])
    .output()
    .expect("task delete failed to run");
  assert!(
    output.status.success(),
    "task delete exited non-zero: {}",
    String::from_utf8_lossy(&output.stderr)
  );
  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(stdout.contains("deleted task"), "got: {stdout}");

  let show = g
    .cmd()
    .args(["task", "show", &task_id])
    .output()
    .expect("task show failed to run");
  assert!(!show.status.success(), "expected task show to fail after delete");

  let task_dir = g.temp_dir_path().join(".gest").join("task");
  let tombstone = std::fs::read_dir(&task_dir)
    .expect("read task dir")
    .flatten()
    .map(|e| e.path())
    .find(|p| {
      p.is_file()
        && p.extension().is_some_and(|ext| ext == "yaml")
        && p
          .file_stem()
          .and_then(|s| s.to_str())
          .is_some_and(|s| s.starts_with(&task_id))
    })
    .expect("tombstone task yaml should still exist after delete");
  let raw = std::fs::read_to_string(&tombstone).expect("read tombstone");
  assert!(raw.contains("deleted_at:"), "missing deleted_at:\n{raw}");
}

#[test]
fn it_refuses_to_delete_a_task_in_an_iteration_without_force() {
  let g = GestCmd::new();
  let task_id = g.create_task("In sprint");
  let iteration_id = g.create_iteration("Sprint");
  let add = g
    .cmd()
    .args(["iteration", "add", &iteration_id, &task_id])
    .output()
    .expect("iteration add failed to run");
  assert!(
    add.status.success(),
    "iteration add exited non-zero: {}",
    String::from_utf8_lossy(&add.stderr)
  );

  let output = g
    .cmd()
    .args(["task", "delete", &task_id, "--yes"])
    .output()
    .expect("task delete failed to run");
  assert!(!output.status.success(), "expected non-zero exit without --force");
  let stderr = String::from_utf8_lossy(&output.stderr);
  assert!(
    stderr.contains("--force") && stderr.contains("iteration"),
    "expected error mentioning --force and iteration: {stderr}"
  );

  // Task still exists.
  let show = g
    .cmd()
    .args(["task", "show", &task_id])
    .output()
    .expect("task show failed to run");
  assert!(
    show.status.success(),
    "task should still exist after refused delete: {}",
    String::from_utf8_lossy(&show.stderr)
  );
  let _ = iteration_id;
}

#[test]
fn it_deletes_a_task_with_force_even_when_in_an_iteration_and_is_undoable() {
  let g = GestCmd::new();
  let task_id = g.create_task("In sprint");
  let iteration_id = g.create_iteration("Sprint");
  let add = g
    .cmd()
    .args(["iteration", "add", &iteration_id, &task_id])
    .output()
    .expect("iteration add failed to run");
  assert!(
    add.status.success(),
    "iteration add exited non-zero: {}",
    String::from_utf8_lossy(&add.stderr)
  );

  let delete = g
    .cmd()
    .args(["task", "delete", &task_id, "--yes", "--force"])
    .output()
    .expect("task delete failed to run");
  assert!(
    delete.status.success(),
    "task delete --force exited non-zero: {}",
    String::from_utf8_lossy(&delete.stderr)
  );

  // iteration_tasks row for this task is gone.
  let status = g
    .cmd()
    .args(["iteration", "status", &iteration_id, "--json"])
    .output()
    .expect("iteration status failed to run");
  assert!(status.status.success());
  let status_stdout = String::from_utf8_lossy(&status.stdout);
  assert!(
    status_stdout.contains("\"total_tasks\": 0"),
    "expected 0 task memberships after delete --force: {status_stdout}"
  );

  // Undo restores the task and its membership.
  let undo = g.cmd().args(["undo"]).output().expect("undo failed to run");
  assert!(
    undo.status.success(),
    "undo exited non-zero: {}",
    String::from_utf8_lossy(&undo.stderr)
  );

  let show = g
    .cmd()
    .args(["task", "show", &task_id])
    .output()
    .expect("task show failed to run");
  assert!(
    show.status.success(),
    "expected task show to succeed after undo: {}",
    String::from_utf8_lossy(&show.stderr)
  );

  let status_after = g
    .cmd()
    .args(["iteration", "status", &iteration_id, "--json"])
    .output()
    .expect("iteration status failed to run");
  assert!(status_after.status.success());
  let status_after_stdout = String::from_utf8_lossy(&status_after.stdout);
  assert!(
    status_after_stdout.contains("\"total_tasks\": 1"),
    "expected 1 task membership after undo: {status_after_stdout}"
  );
}
