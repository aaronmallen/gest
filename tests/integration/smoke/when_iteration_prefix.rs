//! Integration tests covering iteration ID prefix highlighting across list,
//! show, graph, and the two-phase resolver.
//!
//! ANSI color is forced on via `CLICOLOR_FORCE=1` so the rendered
//! `Id` component emits distinct style escapes for the highlighted prefix and
//! the trailing "rest" portion. Each test inspects the escape stream to infer
//! the actual prefix length rendered for a given short ID.

use crate::support::helpers::GestCmd;

/// Strip ANSI escape sequences from a string.
fn strip_ansi(s: &str) -> String {
  let mut out = String::new();
  let bytes = s.as_bytes();
  let mut i = 0;
  while i < bytes.len() {
    if bytes[i] == 0x1b && i + 1 < bytes.len() && bytes[i + 1] == b'[' {
      i += 2;
      while i < bytes.len() && bytes[i] != b'm' {
        i += 1;
      }
      if i < bytes.len() {
        i += 1;
      }
      continue;
    }
    out.push(bytes[i] as char);
    i += 1;
  }
  out
}

/// Given an output stream containing a styled short ID, return the number of
/// characters that were rendered with the "prefix" style. The `Id` component
/// emits `<prefix-style>PREFIX<reset><rest-style>REST<reset>`, so the prefix
/// length is the number of plain characters between the first escape sequence
/// starting with `\x1b[1m` (bold) and the next escape sequence.
fn styled_prefix_len(output: &str, short_id: &str) -> usize {
  // Locate the bold-prefix marker for the short id inside the stream.
  // The prefix style is bold + primary color; the rest style is foreground
  // muted. Both are preceded by an escape sequence, but only the prefix style
  // starts with `\x1b[1m` (the bold attribute).
  let needle_byte = short_id.as_bytes()[0];
  let bytes = output.as_bytes();
  let mut i = 0;
  while i < bytes.len() {
    // Look for an escape opener `\x1b[1m` followed eventually by the first
    // character of the short id before any further non-escape text.
    if bytes[i] == 0x1b && i + 2 < bytes.len() && bytes[i + 1] == b'[' && bytes[i + 2] == b'1' {
      // Skip until `m`.
      let mut j = i + 2;
      while j < bytes.len() && bytes[j] != b'm' {
        j += 1;
      }
      if j >= bytes.len() {
        break;
      }
      j += 1;
      // Skip any additional escape sequences immediately following
      // (`\x1b[Nm` color codes are often emitted as separate SGR sequences).
      while j + 1 < bytes.len() && bytes[j] == 0x1b && bytes[j + 1] == b'[' {
        let mut k = j + 2;
        while k < bytes.len() && bytes[k] != b'm' {
          k += 1;
        }
        if k >= bytes.len() {
          break;
        }
        j = k + 1;
      }
      if j < bytes.len() && bytes[j] == needle_byte {
        // Count plain chars until the next escape.
        let mut count = 0;
        let mut k = j;
        while k < bytes.len() && bytes[k] != 0x1b {
          count += 1;
          k += 1;
        }
        // Ensure the following chars in the short id also match.
        if count >= 1 && output[j..k].starts_with(&short_id[..count.min(short_id.len())]) {
          return count;
        }
      }
    }
    i += 1;
  }
  panic!("could not find styled prefix for id {short_id} in output:\n{output:?}");
}

/// Run an iteration list command with color forced on and return raw stdout.
fn list_raw(g: &GestCmd, all: bool) -> String {
  let mut cmd = g.raw_cmd();
  cmd.env("CLICOLOR_FORCE", "1");
  cmd.args(["iteration", "list"]);
  if all {
    cmd.arg("--all");
  }
  let output = cmd.output().expect("iteration list failed to run");
  assert!(
    output.status.success(),
    "iteration list exited non-zero: {}",
    String::from_utf8_lossy(&output.stderr)
  );
  String::from_utf8_lossy(&output.stdout).to_string()
}

/// Create `count` iterations and return their short IDs.
fn create_iterations(g: &GestCmd, count: usize) -> Vec<String> {
  (0..count).map(|i| g.create_iteration(&format!("Sprint {i}"))).collect()
}

#[test]
fn it_highlights_active_pool_prefix_in_list() {
  let g = GestCmd::new();
  let ids = create_iterations(&g, 3);

  // Complete one iteration so it leaves the active pool.
  g.cmd().args(["iteration", "complete", &ids[0]]).assert().success();

  let raw = list_raw(&g, false);
  let plain = strip_ansi(&raw);

  // The active-pool list should still contain the two active iterations and
  // both should be styled using the active-pool prefix length.
  assert!(plain.contains(&ids[1]), "active list missing id {}", ids[1]);
  assert!(plain.contains(&ids[2]), "active list missing id {}", ids[2]);

  let len_a = styled_prefix_len(&raw, &ids[1]);
  let len_b = styled_prefix_len(&raw, &ids[2]);
  assert_eq!(len_a, len_b, "all rows in a list should share the same prefix length");
  assert!(
    (1..=8).contains(&len_a),
    "prefix length should be clamped to [1, 8], got {len_a}"
  );
}

#[test]
fn it_highlights_all_pool_prefix_with_all_flag() {
  let g = GestCmd::new();
  let ids = create_iterations(&g, 3);

  // Complete one so the --all list spans both pools.
  g.cmd().args(["iteration", "complete", &ids[0]]).assert().success();

  let raw_active = list_raw(&g, false);
  let raw_all = list_raw(&g, true);

  let plain_all = strip_ansi(&raw_all);
  for id in &ids {
    assert!(plain_all.contains(id), "--all list missing id {id}");
  }

  // Both listings pick a shared prefix length across their visible ids. The
  // all-pool length must be at least as large as the active-pool length
  // because the all pool is a superset of the active pool.
  let active_len = styled_prefix_len(&raw_active, &ids[1]);
  let all_len = styled_prefix_len(&raw_all, &ids[1]);
  assert!(
    all_len >= active_len,
    "all-pool prefix ({all_len}) should not be shorter than active-pool prefix ({active_len})"
  );
}

#[test]
fn it_resolves_active_match_over_completed() {
  let g = GestCmd::new();
  let ids = create_iterations(&g, 2);

  // Cancel the first iteration; it leaves the active pool but is still
  // resolvable via the all pool.
  g.cmd().args(["iteration", "cancel", &ids[0]]).assert().success();

  // A two-character prefix of the surviving active iteration must resolve to
  // that iteration even though a cancelled iteration also exists in the all
  // pool.
  let short_prefix = &ids[1][..2];
  let output = g
    .cmd()
    .args(["iteration", "show", short_prefix])
    .output()
    .expect("iteration show failed");
  assert!(
    output.status.success(),
    "iteration show failed: {}",
    String::from_utf8_lossy(&output.stderr)
  );
  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(
    stdout.contains(&ids[1]),
    "active resolve should return active iteration, got: {stdout}"
  );
}

#[test]
fn it_graph_highlights_task_ids_through_id_component() {
  let g = GestCmd::new();
  let iter_id = g.create_iteration("Graph sprint");

  // Create several tasks and attach them to the iteration across two phases.
  let tasks: Vec<String> = (0..3).map(|i| g.create_task(&format!("task {i}"))).collect();
  for (i, tid) in tasks.iter().enumerate() {
    let phase = if i == 0 { "1" } else { "2" };
    g.cmd()
      .args(["iteration", "add", &iter_id, tid, "--phase", phase])
      .assert()
      .success();
  }

  let mut cmd = g.raw_cmd();
  cmd.env("CLICOLOR_FORCE", "1");
  cmd.args(["iteration", "graph", &iter_id]);
  let output = cmd.output().expect("iteration graph failed to run");
  assert!(
    output.status.success(),
    "iteration graph failed: {}",
    String::from_utf8_lossy(&output.stderr)
  );
  let raw = String::from_utf8_lossy(&output.stdout).to_string();
  let plain = strip_ansi(&raw);

  for tid in &tasks {
    assert!(plain.contains(tid), "graph missing task id {tid}");
  }

  // Each task id should be routed through the `Id` atom, which emits a bold
  // prefix style. Verify the styled prefix length is within [1, 8] for every
  // task id in the graph.
  for tid in &tasks {
    let len = styled_prefix_len(&raw, tid);
    assert!(
      (1..=8).contains(&len),
      "task id {tid} rendered with prefix len {len}; expected 1..=8"
    );
  }
}
