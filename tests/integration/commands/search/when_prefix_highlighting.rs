use crate::support::helpers::{GestCmd, rendered_prefix_len, strip_ansi};

/// Compute per-ID unique prefix lengths over a set of IDs (each truncated to at
/// most 8 characters), matching `ui::components::atoms::id::unique_prefix_lengths`.
fn unique_prefix_lengths(ids: &[String]) -> Vec<usize> {
  let shorts: Vec<String> = ids.iter().map(|id| id.chars().take(8).collect()).collect();
  shorts
    .iter()
    .enumerate()
    .map(|(i, s)| {
      let mut len = 1usize;
      for (j, other) in shorts.iter().enumerate() {
        if i == j {
          continue;
        }
        let common = s.chars().zip(other.chars()).take_while(|(a, b)| a == b).count();
        len = len.max((common + 1).min(8));
      }
      len
    })
    .collect()
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

  let task_prefix_lengths = unique_prefix_lengths(&task_ids);
  let artifact_prefix_lengths = unique_prefix_lengths(&artifact_ids);
  let iteration_prefix_lengths = unique_prefix_lengths(&iteration_ids);

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
  for (i, id) in task_ids.iter().enumerate() {
    let short: String = id.chars().take(8).collect();
    let got = rendered_prefix_len(&stdout, &short)
      .unwrap_or_else(|| panic!("could not find rendered task id {short} in:\n{stdout}"));
    assert_eq!(
      got, task_prefix_lengths[i],
      "task id {short}: expected prefix_len {}, got {got}",
      task_prefix_lengths[i]
    );
  }

  for (i, id) in artifact_ids.iter().enumerate() {
    let short: String = id.chars().take(8).collect();
    let got = rendered_prefix_len(&stdout, &short)
      .unwrap_or_else(|| panic!("could not find rendered artifact id {short} in:\n{stdout}"));
    assert_eq!(
      got, artifact_prefix_lengths[i],
      "artifact id {short}: expected prefix_len {}, got {got}",
      artifact_prefix_lengths[i]
    );
  }

  for (i, id) in iteration_ids.iter().enumerate() {
    let short: String = id.chars().take(8).collect();
    let got = rendered_prefix_len(&stdout, &short)
      .unwrap_or_else(|| panic!("could not find rendered iteration id {short} in:\n{stdout}"));
    assert_eq!(
      got, iteration_prefix_lengths[i],
      "iteration id {short}: expected prefix_len {}, got {got}",
      iteration_prefix_lengths[i]
    );
  }
}
