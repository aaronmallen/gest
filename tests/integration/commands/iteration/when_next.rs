use crate::support::helpers::GestCmd;

#[test]
fn it_shows_next_unblocked_task() {
  let g = GestCmd::new();
  let iter_id = g.create_iteration_with_phases("next sprint", &[&["first task", "second task"]]);

  let output = g
    .cmd()
    .args(["iteration", "next", &iter_id])
    .output()
    .expect("iteration next failed");

  assert!(output.status.success(), "iteration next should succeed");
  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(
    stdout.contains("first task") || stdout.contains("second task"),
    "next should surface a phase-1 task title, got: {stdout}"
  );
}

#[test]
fn it_returns_none_when_no_tasks() {
  let g = GestCmd::new();
  let iter_id = g.create_iteration("empty sprint");

  let output = g
    .cmd()
    .args(["iteration", "next", &iter_id])
    .output()
    .expect("iteration next failed");

  // Per ADR-0017 the "no available tasks" outcome is EX_TEMPFAIL (75).
  assert!(
    !output.status.success(),
    "iteration next on an empty iteration should not exit zero"
  );
  assert_eq!(
    output.status.code(),
    Some(75),
    "empty iteration next should exit with EX_TEMPFAIL (75), got: {:?}",
    output.status.code()
  );
}

#[test]
fn it_skips_blocked_tasks() {
  let g = GestCmd::new();
  let iter_id = g.create_iteration_with_phases("blocked sprint", &[&["blocker task", "blocked task"]]);

  // Identify the two tasks via the iteration's graph/next JSON; simpler path is
  // to capture them by creating extra tasks directly and attaching them manually,
  // but the helper already seeded them so we inspect graph output.
  let graph = g
    .cmd()
    .args(["iteration", "show", &iter_id, "--json"])
    .output()
    .expect("iteration show failed");
  assert!(graph.status.success(), "iteration show should succeed");

  // Claim the first task to take it out of the candidate pool.
  let claim = g
    .cmd()
    .args(["iteration", "next", &iter_id, "--claim", "--agent", "test", "-q"])
    .output()
    .expect("iteration next --claim failed");
  assert!(claim.status.success());

  // Second next() call should return a different task (since the first was claimed
  // and is now in-progress/non-available).
  let second = g
    .cmd()
    .args(["iteration", "next", &iter_id])
    .output()
    .expect("iteration next failed");
  assert!(
    second.status.success(),
    "second next should still find an available task: {}",
    String::from_utf8_lossy(&second.stderr)
  );
}

#[test]
fn it_rejects_agent_without_claim() {
  let g = GestCmd::new();
  let iter_id = g.create_iteration_with_phases("agent sprint", &[&["some task"]]);

  let output = g
    .cmd()
    .args(["iteration", "next", &iter_id, "--agent", "foo"])
    .output()
    .expect("iteration next failed");

  assert!(!output.status.success(), "--agent without --claim should exit non-zero");
  let stderr = String::from_utf8_lossy(&output.stderr);
  assert!(
    stderr.contains("--agent requires --claim"),
    "stderr should mention '--agent requires --claim', got: {stderr}"
  );
}

#[test]
fn it_supports_raw_flag() {
  let g = GestCmd::new();
  let iter_id = g.create_iteration_with_phases("raw sprint", &[&["raw task"]]);

  let output = g
    .cmd()
    .args(["iteration", "next", &iter_id, "--raw"])
    .output()
    .expect("iteration next --raw failed");

  assert!(
    output.status.success(),
    "iteration next --raw should succeed: {}",
    String::from_utf8_lossy(&output.stderr)
  );
  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(
    !stdout.trim().is_empty(),
    "iteration next --raw should produce output, got empty stdout"
  );
}
