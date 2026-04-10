//! Integration tests covering the `iteration graph` command output.

use crate::support::helpers::GestCmd;

/// Run `iteration graph` with color disabled and normalize dynamic IDs so
/// snapshot output is deterministic across runs.
fn graph(g: &GestCmd, iter_id: &str) -> String {
  let mut cmd = g.raw_cmd();
  cmd.env("NO_COLOR", "1");
  cmd.args(["iteration", "graph", iter_id]);
  let output = cmd.output().expect("iteration graph failed to run");
  assert!(
    output.status.success(),
    "iteration graph exited non-zero: {}",
    String::from_utf8_lossy(&output.stderr)
  );
  let raw = String::from_utf8_lossy(&output.stdout).into_owned();
  normalize_ids(&raw)
}

/// Replace 8-char gest ID sequences (chars `k`–`z`) with `[ID]`.
fn normalize_ids(s: &str) -> String {
  let is_id_char = |c: char| matches!(c, 'k'..='z');

  let mut result = String::with_capacity(s.len());
  let chars: Vec<char> = s.chars().collect();
  let mut i = 0;

  while i < chars.len() {
    if i + 8 <= chars.len() && chars[i..i + 8].iter().all(|&c| is_id_char(c)) {
      let before_ok = i == 0 || !is_id_char(chars[i - 1]);
      let after_ok = i + 8 >= chars.len() || !is_id_char(chars[i + 8]);
      if before_ok && after_ok {
        result.push_str("[ID]");
        i += 8;
        continue;
      }
    }
    result.push(chars[i]);
    i += 1;
  }

  result
}

#[test]
fn it_renders_a_blocked_task_with_blocked_and_blocking_indicators() {
  let g = GestCmd::new();
  let iter_id = g.create_iteration("Sprint");
  let blocker = g.create_task("blocker task");
  let blocked = g.create_task("blocked task");
  g.cmd().args(["task", "block", &blocker, &blocked]).assert().success();
  g.cmd()
    .args(["iteration", "add", &iter_id, &blocker])
    .assert()
    .success();
  g.cmd()
    .args(["iteration", "add", &iter_id, &blocked])
    .assert()
    .success();

  let out = graph(&g, &iter_id);

  insta::assert_snapshot!(out);
}

#[test]
fn it_renders_a_single_task_phase_without_branches() {
  let g = GestCmd::new();
  let iter_id = g.create_iteration_with_phases("Sprint", &[&["solo task"]]);

  let out = graph(&g, &iter_id);

  insta::assert_snapshot!(out);
}

#[test]
fn it_renders_branch_connectors_for_multi_task_phases() {
  let g = GestCmd::new();
  let iter_id = g.create_iteration_with_phases("Sprint", &[&["first task", "second task", "third task"]]);

  let out = graph(&g, &iter_id);

  insta::assert_snapshot!(out);
}

#[test]
fn it_renders_continuation_lines_between_phases() {
  let g = GestCmd::new();
  let iter_id = g.create_iteration_with_phases(
    "Sprint",
    &[&["phase one task"], &["phase two task"], &["phase three task"]],
  );

  let out = graph(&g, &iter_id);

  insta::assert_snapshot!(out);
}

#[test]
fn it_renders_phase_header_and_summary_line() {
  let g = GestCmd::new();
  let iter_id = g.create_iteration_with_phases("Sprint", &[&["a task"]]);

  let out = graph(&g, &iter_id);

  insta::assert_snapshot!(out);
}

#[test]
fn it_renders_priority_badge_when_task_has_priority() {
  let g = GestCmd::new();
  let iter_id = g.create_iteration("Sprint");
  let task_id = g.create_task("priority task");
  g.cmd()
    .args(["task", "update", &task_id, "--priority", "1"])
    .assert()
    .success();
  g.cmd()
    .args(["iteration", "add", &iter_id, &task_id])
    .assert()
    .success();

  let out = graph(&g, &iter_id);

  insta::assert_snapshot!(out);
}

#[test]
fn it_renders_without_priority_column_when_absent() {
  let g = GestCmd::new();
  let iter_id = g.create_iteration_with_phases("Sprint", &[&["no priority"]]);

  let out = graph(&g, &iter_id);

  insta::assert_snapshot!(out);
}
