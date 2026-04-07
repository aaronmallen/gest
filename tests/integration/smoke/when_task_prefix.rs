//! Smoke tests for task `prefix_len` plumbing across list/show and the
//! active-first ID resolver.

use crate::support::helpers::GestCmd;

#[test]
fn it_highlights_active_pool_prefix_in_list() {
  let g = GestCmd::new();

  // Two open tasks plus a done task. The default `task list` should only
  // see the active set (two rows) and resolve prefixes against that pool.
  let open_a = g.create_task("alpha task");
  let open_b = g.create_task("bravo task");
  let done = g.create_task("done task");
  g.cmd()
    .args(["task", "update", &done, "--status", "done"])
    .assert()
    .success();

  let output = g.cmd().args(["task", "list"]).output().expect("task list failed");
  assert!(output.status.success(), "task list exited non-zero");

  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(stdout.contains("alpha task"), "should list open task alpha: {stdout}");
  assert!(stdout.contains("bravo task"), "should list open task bravo: {stdout}");
  assert!(
    !stdout.contains("done task"),
    "should hide terminal task by default: {stdout}"
  );

  // The visible 8-char shorts of the active pool tasks should round-trip
  // through the active-first resolver via `task show`.
  let short_a = &open_a[..8];
  let short_b = &open_b[..8];
  let show_a = g.cmd().args(["task", "show", short_a]).assert().success();
  let show_b = g.cmd().args(["task", "show", short_b]).assert().success();
  let out_a = String::from_utf8_lossy(&show_a.get_output().stdout).to_string();
  let out_b = String::from_utf8_lossy(&show_b.get_output().stdout).to_string();
  assert!(out_a.contains("alpha task"), "expected alpha shown via active prefix");
  assert!(out_b.contains("bravo task"), "expected bravo shown via active prefix");
}

#[test]
fn it_highlights_all_pool_prefix_with_all_flag() {
  let g = GestCmd::new();

  let _open = g.create_task("still open");
  let done = g.create_task("already done");
  g.cmd()
    .args(["task", "update", &done, "--status", "done"])
    .assert()
    .success();

  // With `--all`, the listing should include the terminal row and use the
  // all-rows pool for prefix highlighting.
  let output = g
    .cmd()
    .args(["task", "list", "--all"])
    .output()
    .expect("task list --all failed");
  assert!(output.status.success(), "task list --all exited non-zero");

  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(stdout.contains("still open"), "should list open task: {stdout}");
  assert!(stdout.contains("already done"), "should list done task: {stdout}");
}

#[test]
fn it_resolves_active_match_over_terminal() {
  let g = GestCmd::new();

  // Create an open and a done task. Both have unique short IDs, but we
  // verify that `task show <full-id>` continues to work for the open task
  // even when a done task with the same status pool exists. The two-phase
  // resolver should silently prefer active matches.
  let open = g.create_task("active candidate");
  let done = g.create_task("terminal candidate");
  g.cmd()
    .args(["task", "update", &done, "--status", "done"])
    .assert()
    .success();

  let short = &open[..8];
  let output = g
    .cmd()
    .args(["task", "show", short])
    .output()
    .expect("task show failed");
  assert!(output.status.success(), "task show exited non-zero");
  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(
    stdout.contains("active candidate"),
    "should resolve to active task: {stdout}"
  );
  assert!(
    !stdout.contains("terminal candidate"),
    "should not include terminal task: {stdout}"
  );
}

#[test]
fn it_falls_back_to_done_when_no_active_match() {
  let g = GestCmd::new();

  let done = g.create_task("archived work");
  g.cmd()
    .args(["task", "update", &done, "--status", "done"])
    .assert()
    .success();

  // `task show` against the done task's prefix should fall back to the
  // all-rows pool and resolve successfully.
  let short = &done[..8];
  let output = g
    .cmd()
    .args(["task", "show", short])
    .output()
    .expect("task show failed");
  assert!(
    output.status.success(),
    "task show should fall back to terminal pool, stderr={}",
    String::from_utf8_lossy(&output.stderr)
  );
  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(stdout.contains("archived work"), "should show archived task: {stdout}");
  assert!(stdout.contains("done"), "should show done status: {stdout}");
}
