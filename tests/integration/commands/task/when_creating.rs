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
fn it_drops_blank_entries_from_stray_commas_in_tags() {
  let g = GestCmd::new();

  let create = g
    .cmd()
    .args(["task", "create", "stray commas", "--tag", "x,,y,"])
    .output()
    .expect("task create failed to run");
  assert!(
    create.status.success(),
    "task create exited non-zero: {}",
    String::from_utf8_lossy(&create.stderr)
  );

  let list = g
    .cmd()
    .args(["tag", "list", "--task"])
    .output()
    .expect("tag list failed to run");
  let stdout = String::from_utf8_lossy(&list.stdout);

  assert!(stdout.contains("x"), "expected x tag, got: {stdout}");
  assert!(stdout.contains("y"), "expected y tag, got: {stdout}");
  // No blank-labeled tag line (rendered as `#` with no label) should appear.
  assert!(
    !stdout.lines().any(|line| line.trim() == "#"),
    "expected no blank tag line, got: {stdout}"
  );
}

#[test]
fn it_rejects_phase_without_iteration() {
  let g = GestCmd::new();
  let output = g
    .cmd()
    .args(["task", "create", "Hello task", "--phase", "2"])
    .output()
    .expect("task create failed to run");

  assert!(
    !output.status.success(),
    "task create --phase without --iteration should error"
  );
  let stderr = String::from_utf8_lossy(&output.stderr);
  assert!(
    stderr.contains("--iteration"),
    "expected error to mention --iteration, got: {stderr}"
  );
}

#[test]
fn it_splits_comma_separated_tag_values() {
  let g = GestCmd::new();

  let create = g
    .cmd()
    .args(["task", "create", "comma tags", "--tag", "alpha,beta"])
    .output()
    .expect("task create failed to run");
  assert!(
    create.status.success(),
    "task create exited non-zero: {}",
    String::from_utf8_lossy(&create.stderr)
  );

  let list = g
    .cmd()
    .args(["tag", "list", "--task"])
    .output()
    .expect("tag list failed to run");
  let stdout = String::from_utf8_lossy(&list.stdout);

  assert!(stdout.contains("alpha"), "expected alpha tag, got: {stdout}");
  assert!(stdout.contains("beta"), "expected beta tag, got: {stdout}");
  assert!(
    !stdout.contains("alpha,beta"),
    "expected no literal comma-joined tag, got: {stdout}"
  );
}

#[test]
fn it_trims_whitespace_around_comma_split_tags() {
  let g = GestCmd::new();

  let create = g
    .cmd()
    .args(["task", "create", "spaced tags", "--tag", "  first , second  "])
    .output()
    .expect("task create failed to run");
  assert!(
    create.status.success(),
    "task create exited non-zero: {}",
    String::from_utf8_lossy(&create.stderr)
  );

  let list = g
    .cmd()
    .args(["tag", "list", "--task"])
    .output()
    .expect("tag list failed to run");
  let stdout = String::from_utf8_lossy(&list.stdout);

  assert!(stdout.contains("first"), "expected first tag, got: {stdout}");
  assert!(stdout.contains("second"), "expected second tag, got: {stdout}");
  assert!(
    !stdout.contains(" first"),
    "expected no leading whitespace in tag, got: {stdout}"
  );
  assert!(
    !stdout.contains("second "),
    "expected no trailing whitespace in tag, got: {stdout}"
  );
}
