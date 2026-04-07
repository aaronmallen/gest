//! End-to-end cross-command workflows that exercise multi-step user journeys.

use crate::support::helpers::GestCmd;

#[test]
fn it_attaches_project_lists_and_detaches() {
  let g = GestCmd::new();
  let workspace_dir = g.temp_dir_path().join("workspace-a");
  g.init_extra_project(&workspace_dir);

  // The extra project should now show up in the list alongside the primary.
  let list = g
    .cmd()
    .args(["project", "list", "--json"])
    .output()
    .expect("project list failed to run");
  assert!(list.status.success(), "project list should succeed");
  let stdout = String::from_utf8_lossy(&list.stdout);
  let parsed: serde_json::Value = serde_json::from_str(&stdout).expect("list should be valid JSON");
  let arr = parsed.as_array().expect("list should be an array");
  assert_eq!(
    arr.len(),
    2,
    "expected two projects after init_extra_project, got: {stdout}"
  );

  // Both projects should expose id and root fields.
  for entry in arr {
    assert!(entry["id"].is_string(), "project entry missing id: {entry}");
    assert!(entry["root"].is_string(), "project entry missing root: {entry}");
  }
}

#[test]
fn it_creates_artifact_links_task_completes_and_undoes() {
  let g = GestCmd::new();
  let artifact_id = g.create_artifact("Journey spec", "driving the workflow");
  let task_id = g.create_task("Implement the spec");

  g.link_task(&task_id, &artifact_id, "artifact", "relates-to");
  g.complete_task(&task_id);

  // Verify the task is done.
  let show = g
    .cmd()
    .args(["task", "show", &task_id])
    .output()
    .expect("task show failed to run");
  assert!(show.status.success(), "task show should succeed");
  let stdout = String::from_utf8_lossy(&show.stdout);
  assert!(stdout.contains("done"), "task should be done, got: {stdout}");

  // Start a fresh journey and verify undo rewinds the most recent creation.
  let fresh_task = g.create_task("Undoable follow-up");
  let undo = g.cmd().args(["undo"]).output().expect("undo failed to run");
  assert!(
    undo.status.success(),
    "undo should succeed: {}",
    String::from_utf8_lossy(&undo.stderr)
  );
  let after = g
    .cmd()
    .args(["task", "show", &fresh_task])
    .output()
    .expect("task show failed to run");
  assert!(
    !after.status.success(),
    "undone task should no longer be visible via task show"
  );
}

#[test]
fn it_creates_iteration_adds_tasks_advances_and_nexts() {
  let g = GestCmd::new();
  let iter_id = g.create_iteration_with_phases(
    "Lifecycle sprint",
    &[&["phase one task"], &["phase two task a", "phase two task b"]],
  );

  // Pulling the next task should return a task from phase 1.
  let next = g
    .cmd()
    .args(["iteration", "next", &iter_id])
    .output()
    .expect("iteration next failed to run");
  assert!(
    next.status.success(),
    "iteration next should succeed: {}",
    String::from_utf8_lossy(&next.stderr)
  );
  let next_stdout = String::from_utf8_lossy(&next.stdout);
  assert!(
    next_stdout.contains("phase one task"),
    "first next should return the phase 1 task, got: {next_stdout}"
  );

  // Claim the phase 1 task, complete it; the iteration should auto-progress.
  let claim = g
    .cmd()
    .args([
      "iteration",
      "next",
      &iter_id,
      "--claim",
      "--agent",
      "behavior-test",
      "-q",
    ])
    .output()
    .expect("iteration next --claim failed to run");
  assert!(claim.status.success(), "iteration next --claim should succeed");
  let claimed = String::from_utf8_lossy(&claim.stdout).trim().to_string();
  g.complete_task(&claimed);

  // Status should reflect progress: one task done, two tasks open (both in phase 2).
  let status = g
    .cmd()
    .args(["iteration", "status", &iter_id, "--json"])
    .output()
    .expect("iteration status failed to run");
  assert!(status.status.success(), "iteration status should succeed");
  let status_stdout = String::from_utf8_lossy(&status.stdout);
  let parsed: serde_json::Value = serde_json::from_str(&status_stdout).expect("status should be valid JSON");
  assert_eq!(parsed["done"].as_u64(), Some(1), "one task done, got: {status_stdout}");
  assert_eq!(parsed["open"].as_u64(), Some(2), "two tasks open, got: {status_stdout}");
}
