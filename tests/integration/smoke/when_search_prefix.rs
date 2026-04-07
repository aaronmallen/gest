use std::collections::HashSet;

use crate::support::helpers::GestCmd;

/// Compute the minimum unique prefix length over a set of IDs (each truncated
/// to at most 8 characters), matching `ui::components::atoms::id::min_unique_prefix`.
fn min_unique_prefix(ids: &[String]) -> usize {
  let shorts: Vec<String> = ids.iter().map(|id| id.chars().take(8).collect()).collect();
  for len in 1..=8 {
    let prefixes: HashSet<&str> = shorts.iter().map(|s| &s[..len.min(s.len())]).collect();
    if prefixes.len() == shorts.len() {
      return len;
    }
  }
  8
}

/// Strip ANSI escape sequences for length-based comparison.
fn strip_ansi(s: &str) -> String {
  let mut out = String::with_capacity(s.len());
  let bytes = s.as_bytes();
  let mut i = 0;
  while i < bytes.len() {
    if bytes[i] == 0x1b && i + 1 < bytes.len() && bytes[i + 1] == b'[' {
      // Skip until we hit a final byte in 0x40..=0x7E.
      i += 2;
      while i < bytes.len() && !(0x40..=0x7e).contains(&bytes[i]) {
        i += 1;
      }
      if i < bytes.len() {
        i += 1;
      }
    } else {
      out.push(bytes[i] as char);
      i += 1;
    }
  }
  out
}

/// Extract the rendered prefix length for `short_id` from a colored output
/// buffer.
///
/// IDs are displayed as `<CSI>...m{prefix}<CSI>0m<CSI>...m{rest}<CSI>0m`. We
/// scan for the id as a contiguous run of visible characters, allowing escape
/// sequences to interleave; the first interleaved escape after at least one
/// visible character marks the prefix→rest boundary.
fn rendered_prefix_len(output: &str, short_id: &str) -> Option<usize> {
  let bytes = output.as_bytes();
  let target = short_id.as_bytes();
  let mut i = 0;
  while i < bytes.len() {
    let mut j = i;
    let mut t = 0;
    let mut prefix_len: Option<usize> = None;
    let mut visible_seen = 0usize;
    let mut last_was_visible = true;
    while t < target.len() && j < bytes.len() {
      if bytes[j] == 0x1b && j + 1 < bytes.len() && bytes[j + 1] == b'[' {
        if t > 0 && prefix_len.is_none() && last_was_visible {
          prefix_len = Some(visible_seen);
        }
        j += 2;
        while j < bytes.len() && !(0x40..=0x7e).contains(&bytes[j]) {
          j += 1;
        }
        if j < bytes.len() {
          j += 1;
        }
        last_was_visible = false;
        continue;
      }
      if bytes[j] == target[t] {
        t += 1;
        j += 1;
        visible_seen += 1;
        last_was_visible = true;
      } else {
        break;
      }
    }
    if t == target.len() {
      return Some(prefix_len.unwrap_or(visible_seen));
    }
    i += 1;
  }
  None
}

#[test]
fn it_highlights_per_entity_pool_prefixes_in_search() {
  let g = GestCmd::new();

  // Seed several entities of each type, all containing the search term.
  let task_ids: Vec<String> = (0..6).map(|i| g.create_task(&format!("needle task {i}"))).collect();
  let artifact_ids: Vec<String> = (0..5)
    .map(|i| g.create_artifact(&format!("needle artifact {i}"), &format!("body {i}")))
    .collect();
  let iteration_ids: Vec<String> = (0..4)
    .map(|i| g.create_iteration(&format!("needle iteration {i}")))
    .collect();

  let expected_task_prefix = min_unique_prefix(&task_ids);
  let expected_artifact_prefix = min_unique_prefix(&artifact_ids);
  let expected_iteration_prefix = min_unique_prefix(&iteration_ids);

  // Run search with colors forced on so prefix highlighting is observable.
  let output = g
    .raw_cmd()
    .env("CLICOLOR_FORCE", "1")
    .args(["search", "needle"])
    .output()
    .expect("search failed to run");

  assert!(
    output.status.success(),
    "search exited non-zero: {}",
    String::from_utf8_lossy(&output.stderr)
  );

  let stdout = String::from_utf8_lossy(&output.stdout).to_string();
  let plain = strip_ansi(&stdout);

  // Sanity: every seeded entity should appear in stripped output.
  for id in task_ids.iter().chain(artifact_ids.iter()).chain(iteration_ids.iter()) {
    let short: String = id.chars().take(8).collect();
    assert!(plain.contains(&short), "missing id {short} in search output:\n{plain}");
  }

  // Verify per-entity prefix lengths in colored output.
  for id in &task_ids {
    let short: String = id.chars().take(8).collect();
    let got = rendered_prefix_len(&stdout, &short)
      .unwrap_or_else(|| panic!("could not find rendered task id {short} in:\n{stdout}"));
    assert_eq!(
      got, expected_task_prefix,
      "task id {short}: expected prefix_len {expected_task_prefix}, got {got}"
    );
  }

  for id in &artifact_ids {
    let short: String = id.chars().take(8).collect();
    let got = rendered_prefix_len(&stdout, &short)
      .unwrap_or_else(|| panic!("could not find rendered artifact id {short} in:\n{stdout}"));
    assert_eq!(
      got, expected_artifact_prefix,
      "artifact id {short}: expected prefix_len {expected_artifact_prefix}, got {got}"
    );
  }

  for id in &iteration_ids {
    let short: String = id.chars().take(8).collect();
    let got = rendered_prefix_len(&stdout, &short)
      .unwrap_or_else(|| panic!("could not find rendered iteration id {short} in:\n{stdout}"));
    assert_eq!(
      got, expected_iteration_prefix,
      "iteration id {short}: expected prefix_len {expected_iteration_prefix}, got {got}"
    );
  }
}
