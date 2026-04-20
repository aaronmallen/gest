use crate::support::helpers::GestCmd;

#[test]
fn it_advances_to_next_phase() {
  let g = GestCmd::new();
  let iter_id = g.create_iteration_with_phases(
    "advance sprint",
    &[&["phase one a", "phase one b"], &["phase two task"]],
  );

  // With two open tasks in phase 1, advance --force should jump to phase 2 even
  // though phase 1 is non-terminal, exercising the phase state transition.
  let output = g
    .cmd()
    .args(["iteration", "advance", &iter_id, "--force"])
    .output()
    .expect("iteration advance failed");
  assert!(
    output.status.success(),
    "iteration advance --force should succeed: {}",
    String::from_utf8_lossy(&output.stderr)
  );
}

#[test]
fn it_rejects_advance_when_tasks_incomplete() {
  let g = GestCmd::new();
  let iter_id = g.create_iteration_with_phases("stuck sprint", &[&["still open task"], &["future"]]);

  let output = g
    .cmd()
    .args(["iteration", "advance", &iter_id])
    .output()
    .expect("iteration advance failed");

  assert!(
    !output.status.success(),
    "advance should refuse when current phase has open tasks"
  );
  let stderr = String::from_utf8_lossy(&output.stderr);
  assert!(
    stderr.to_lowercase().contains("non-terminal") || stderr.to_lowercase().contains("incomplete"),
    "stderr should explain why advance was blocked, got: {stderr}"
  );
}

#[test]
fn it_exits_with_ex_unavailable_when_iteration_not_active() {
  let g = GestCmd::new();
  let iter_id = g.create_iteration_with_phases("done sprint", &[&["only task"]]);

  // Complete the only task so the iteration reaches a terminal status.
  let claim = g
    .cmd()
    .args(["iteration", "next", &iter_id, "--claim", "--agent", "test", "-q"])
    .output()
    .expect("iteration next --claim failed");
  assert!(claim.status.success());
  let task_id = String::from_utf8_lossy(&claim.stdout).trim().to_string();
  g.complete_task(&task_id);

  let output = g
    .cmd()
    .args(["iteration", "advance", &iter_id])
    .output()
    .expect("iteration advance failed");

  assert_eq!(
    output.status.code(),
    Some(69),
    "advancing a non-active iteration should exit with EX_UNAVAILABLE (69), got {:?}: {}",
    output.status.code(),
    String::from_utf8_lossy(&output.stderr)
  );
}

#[test]
fn it_advances_past_last_phase() {
  let g = GestCmd::new();
  let iter_id = g.create_iteration_with_phases("final sprint", &[&["only task"]]);

  // Complete the only task; the iteration transitions past the last phase
  // into a completed state automatically.
  let claim = g
    .cmd()
    .args(["iteration", "next", &iter_id, "--claim", "--agent", "test", "-q"])
    .output()
    .expect("iteration next --claim failed");
  assert!(claim.status.success());
  let task_id = String::from_utf8_lossy(&claim.stdout).trim().to_string();
  g.complete_task(&task_id);

  // Attempting to advance a completed iteration is an error.
  let output = g
    .cmd()
    .args(["iteration", "advance", &iter_id])
    .output()
    .expect("iteration advance failed");
  assert!(
    !output.status.success(),
    "advancing past the last phase of a completed iteration should error"
  );
  let stderr = String::from_utf8_lossy(&output.stderr);
  assert!(
    stderr.to_lowercase().contains("complete"),
    "stderr should mention iteration completion, got: {stderr}"
  );
}
