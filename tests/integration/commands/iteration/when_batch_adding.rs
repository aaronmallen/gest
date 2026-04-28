use crate::support::helpers::GestCmd;

/// Build an NDJSON payload from a slice of `(task_id, phase)` pairs where
/// `phase == 0` means "omit the phase field" (auto-assign).
fn ndjson(records: &[(&str, u32)]) -> String {
  records
    .iter()
    .map(|(task, phase)| {
      if *phase == 0 {
        format!("{{\"task\":\"{task}\"}}\n")
      } else {
        format!("{{\"task\":\"{task}\",\"phase\":{phase}}}\n")
      }
    })
    .collect()
}

#[test]
fn it_adds_tasks_with_explicit_phases() {
  let g = GestCmd::new();
  let iter_id = g.create_iteration("explicit phase batch");
  let task_a = g.create_task("batch task a");
  let task_b = g.create_task("batch task b");

  let input = ndjson(&[(&task_a, 3), (&task_b, 5)]);

  let output = g
    .cmd()
    .args(["iteration", "add", &iter_id, "--batch", "--json"])
    .write_stdin(input)
    .output()
    .expect("iteration add --batch failed to run");

  assert!(
    output.status.success(),
    "batch add with explicit phases should succeed: {}",
    String::from_utf8_lossy(&output.stderr)
  );
  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(
    stdout.contains("\"count\": 2") || stdout.contains("\"count\":2"),
    "expected count 2 in output, got: {stdout}"
  );
}

#[test]
fn it_adds_tasks_with_mixed_explicit_and_auto_phases() {
  let g = GestCmd::new();
  let iter_id = g.create_iteration("mixed phase batch");
  let task_a = g.create_task("mixed explicit task");
  let task_b = g.create_task("mixed auto task");

  // task_a gets explicit phase 2; task_b has no phase (should auto to 3).
  let input = format!("{{\"task\":\"{task_a}\",\"phase\":2}}\n{{\"task\":\"{task_b}\"}}\n");

  let output = g
    .cmd()
    .args(["iteration", "add", &iter_id, "--batch", "--json"])
    .write_stdin(input)
    .output()
    .expect("iteration add --batch (mixed) failed to run");

  assert!(
    output.status.success(),
    "mixed batch add should succeed: {}",
    String::from_utf8_lossy(&output.stderr)
  );
  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(
    stdout.contains("\"count\": 2") || stdout.contains("\"count\":2"),
    "expected count 2 in output, got: {stdout}"
  );

  // Confirm the batch landed tasks: a subsequent single add without a phase should
  // use max_phase + 1.  The explicit phase was 2; the auto-assigned task got
  // next_phase=1 (auto counter starts at 1 and is independent of explicit phases),
  // so the max phase in the iteration is 2.  The probe therefore lands on phase 3.
  let task_c = g.create_task("phase probe task");
  let probe = g
    .cmd()
    .args(["iteration", "add", &iter_id, &task_c, "--json"])
    .output()
    .expect("probe iteration add failed to run");

  assert!(probe.status.success(), "probe add should succeed");
  let probe_stdout = String::from_utf8_lossy(&probe.stdout);
  assert!(
    probe_stdout.contains("\"phase\": 3") || probe_stdout.contains("\"phase\":3"),
    "probe task should land on phase 3 (max of explicit 2 and auto 1 is 2), got: {probe_stdout}"
  );
}

#[test]
fn it_auto_increments_phases_when_phase_is_omitted() {
  let g = GestCmd::new();
  let iter_id = g.create_iteration("auto-phase batch");
  let task_a = g.create_task("auto phase task a");
  let task_b = g.create_task("auto phase task b");

  // No phase fields — both should be auto-assigned starting from phase 1.
  let input = ndjson(&[(&task_a, 0), (&task_b, 0)]);

  let output = g
    .cmd()
    .args(["iteration", "add", &iter_id, "--batch", "--json"])
    .write_stdin(input)
    .output()
    .expect("iteration add --batch failed to run");

  assert!(
    output.status.success(),
    "batch add without phases should succeed: {}",
    String::from_utf8_lossy(&output.stderr)
  );
  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(
    stdout.contains("\"count\": 2") || stdout.contains("\"count\":2"),
    "expected count 2 in output, got: {stdout}"
  );
}

#[test]
fn it_rejects_batch_flag_combined_with_phase_flag() {
  let g = GestCmd::new();
  let iter_id = g.create_iteration("conflict test batch+phase");

  let output = g
    .cmd()
    .args(["iteration", "add", &iter_id, "--batch", "--phase", "1"])
    .output()
    .expect("iteration add --batch --phase failed to run");

  assert!(!output.status.success(), "--batch combined with --phase should fail");
}

#[test]
fn it_rejects_batch_flag_combined_with_positional_task() {
  let g = GestCmd::new();
  let iter_id = g.create_iteration("conflict test batch+task");
  let task_id = g.create_task("conflicting task");

  let output = g
    .cmd()
    .args(["iteration", "add", &iter_id, &task_id, "--batch"])
    .output()
    .expect("iteration add --batch with task failed to run");

  assert!(
    !output.status.success(),
    "--batch combined with a positional task argument should fail"
  );
}

#[test]
fn it_prints_iteration_short_id_in_quiet_mode() {
  let g = GestCmd::new();
  let iter_id = g.create_iteration("quiet batch iteration");
  let task_a = g.create_task("quiet batch task");

  let input = ndjson(&[(&task_a, 1)]);

  let output = g
    .cmd()
    .args(["iteration", "add", &iter_id, "--batch", "-q"])
    .write_stdin(input)
    .output()
    .expect("iteration add --batch -q failed to run");

  assert!(
    output.status.success(),
    "iteration add --batch -q should succeed: {}",
    String::from_utf8_lossy(&output.stderr)
  );
  let stdout = String::from_utf8_lossy(&output.stdout);
  let lines: Vec<&str> = stdout.lines().filter(|l| !l.is_empty()).collect();

  assert_eq!(lines.len(), 1, "expected exactly one non-blank line, got: {stdout}");
  assert!(
    iter_id.starts_with(lines[0]),
    "expected line to be a short prefix of the iteration ID {iter_id:?}, got: {:?}",
    lines[0]
  );
}

#[test]
fn it_rolls_back_entire_batch_on_invalid_task_reference() {
  let g = GestCmd::new();
  let iter_id = g.create_iteration("rollback batch");
  let task_a = g.create_task("valid task for rollback test");

  // First line is valid; second references a non-existent task.
  let input = format!("{{\"task\":\"{task_a}\",\"phase\":1}}\n{{\"task\":\"zzzzzzzzzz\",\"phase\":2}}\n");

  let output = g
    .cmd()
    .args(["iteration", "add", &iter_id, "--batch"])
    .write_stdin(input)
    .output()
    .expect("iteration add --batch (rollback) failed to run");

  assert!(
    !output.status.success(),
    "batch add with invalid task reference should exit non-zero"
  );

  // Verify no tasks were added: a subsequent single add of task_a should land on
  // phase 1 (i.e. max_phase returns None, meaning the iteration is still empty).
  let probe = g
    .cmd()
    .args(["iteration", "add", &iter_id, &task_a, "--json"])
    .output()
    .expect("probe add after rollback failed to run");

  assert!(probe.status.success(), "probe add after failed batch should succeed");
  let probe_stdout = String::from_utf8_lossy(&probe.stdout);
  assert!(
    probe_stdout.contains("\"phase\": 1") || probe_stdout.contains("\"phase\":1"),
    "iteration should be empty after rollback (task lands on phase 1), got: {probe_stdout}"
  );
}
