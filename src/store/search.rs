use rayon::prelude::*;

use crate::{
  config::Settings,
  model::{Artifact, ArtifactFilter, Iteration, IterationFilter, Task, TaskFilter},
};

/// Collected search results across entity types.
pub struct SearchResults {
  pub artifacts: Vec<Artifact>,
  pub iterations: Vec<Iteration>,
  pub tasks: Vec<Task>,
}

/// Check whether `haystack` contains `needle` case-insensitively, without
/// allocating a lowercase copy of the haystack.  `needle` **must** already be
/// lowercase.
fn contains_ignore_case(haystack: &str, needle: &str) -> bool {
  if needle.is_empty() {
    return true;
  }

  let needle_len = needle.len();
  if haystack.len() < needle_len {
    return false;
  }

  // Slide a byte-length window over the haystack.  Because `to_lowercase` can
  // change the byte-length of a char, we iterate by character boundaries and
  // compare lowercased chars from the haystack against the needle chars.
  'outer: for (byte_offset, _) in haystack.char_indices() {
    let remaining = &haystack[byte_offset..];
    let mut hay_chars = remaining.chars();
    let mut needle_chars = needle.chars();

    loop {
      match needle_chars.next() {
        None => return true, // entire needle matched
        Some(nc) => match hay_chars.next() {
          None => break, // haystack exhausted for this starting position
          Some(hc) => {
            // Compare lowercased chars.  `to_lowercase` returns an iterator
            // (multi-char mappings exist, e.g. 'ß' -> "ss"), so we must
            // compare element-by-element.
            let mut h_lower = hc.to_lowercase();
            let mut n_lower = nc.to_lowercase(); // needle is already lowercase, but keeps correctness
            loop {
              match (h_lower.next(), n_lower.next()) {
                (Some(a), Some(b)) if a == b => continue,
                (None, None) => break, // this char matched
                _ => continue 'outer,  // mismatch
              }
            }
          }
        },
      }
    }
  }

  false
}

/// Perform a case-insensitive full-text search across tasks and artifacts.
///
/// Supports the `tag:<name>` prefix to filter by exact tag match.
pub fn search(config: &Settings, query: &str, show_all: bool) -> super::Result<SearchResults> {
  let query_lower = query.to_lowercase();

  // Check for tag: prefix filter.
  let tag_filter = query_lower.strip_prefix("tag:").map(|t| t.to_owned());

  let task_filter = TaskFilter {
    all: show_all,
    ..Default::default()
  };
  let all_tasks = super::list_tasks(config, &task_filter)?;

  let artifact_filter = ArtifactFilter {
    all: show_all,
    ..Default::default()
  };
  let all_artifacts = super::list_artifacts(config, &artifact_filter)?;

  let iteration_filter = IterationFilter {
    all: show_all,
    ..Default::default()
  };
  let all_iterations = super::list_iterations(config, &iteration_filter)?;

  let tasks: Vec<Task> = all_tasks
    .into_par_iter()
    .filter(|task| {
      if let Some(ref tag) = tag_filter {
        return task.tags.iter().any(|t| t.to_lowercase() == *tag);
      }
      contains_ignore_case(&task.title, &query_lower)
        || contains_ignore_case(&task.description, &query_lower)
        || task.tags.iter().any(|t| contains_ignore_case(t, &query_lower))
        || contains_ignore_case(&task.status.to_string(), &query_lower)
        || (!task.metadata.is_empty()
          && contains_ignore_case(&toml::to_string(&task.metadata).unwrap_or_default(), &query_lower))
    })
    .collect();

  let artifacts: Vec<Artifact> = all_artifacts
    .into_par_iter()
    .filter(|artifact| {
      if let Some(ref tag) = tag_filter {
        return artifact.tags.iter().any(|t| t.to_lowercase() == *tag);
      }
      contains_ignore_case(&artifact.title, &query_lower)
        || contains_ignore_case(&artifact.body, &query_lower)
        || artifact.tags.iter().any(|t| contains_ignore_case(t, &query_lower))
        || contains_ignore_case(artifact.kind.as_deref().unwrap_or(""), &query_lower)
        || (!artifact.metadata.is_empty()
          && contains_ignore_case(
            &yaml_serde::to_string(&artifact.metadata).unwrap_or_default(),
            &query_lower,
          ))
    })
    .collect();

  let iterations: Vec<Iteration> = all_iterations
    .into_par_iter()
    .filter(|iteration| {
      if let Some(ref tag) = tag_filter {
        return iteration.tags.iter().any(|t| t.to_lowercase() == *tag);
      }
      contains_ignore_case(&iteration.title, &query_lower)
        || contains_ignore_case(&iteration.description, &query_lower)
        || iteration.tags.iter().any(|t| contains_ignore_case(t, &query_lower))
        || (!iteration.metadata.is_empty()
          && contains_ignore_case(&toml::to_string(&iteration.metadata).unwrap_or_default(), &query_lower))
    })
    .collect();

  Ok(SearchResults {
    artifacts,
    iterations,
    tasks,
  })
}

#[cfg(test)]
mod tests {
  use super::*;

  fn make_test_artifact(id: &str, title: &str, body: &str) -> Artifact {
    Artifact {
      title: title.to_string(),
      body: body.to_string(),
      ..crate::test_helpers::make_test_artifact(id)
    }
  }

  fn make_test_iteration(id: &str, title: &str) -> Iteration {
    Iteration {
      title: title.to_string(),
      ..crate::test_helpers::make_test_iteration(id)
    }
  }

  fn make_test_task(id: &str, title: &str) -> Task {
    Task {
      title: title.to_string(),
      ..crate::test_helpers::make_test_task(id)
    }
  }

  mod search {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_finds_artifacts_by_body() {
      let dir = tempfile::tempdir().unwrap();
      let artifact = make_test_artifact(
        "kkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkk",
        "Notes",
        "This contains a secret keyword",
      );
      crate::store::write_artifact(
        &crate::test_helpers::make_test_config(dir.path().to_path_buf()),
        &artifact,
      )
      .unwrap();

      let results = super::super::search(
        &crate::test_helpers::make_test_config(dir.path().to_path_buf()),
        "secret",
        false,
      )
      .unwrap();
      assert_eq!(results.artifacts.len(), 1);
      assert_eq!(results.artifacts[0].title, "Notes");
    }

    #[test]
    fn it_finds_tasks_by_title() {
      let dir = tempfile::tempdir().unwrap();
      let task = make_test_task("zyxwvutsrqponmlkzyxwvutsrqponmlk", "Important Feature");
      crate::store::write_task(&crate::test_helpers::make_test_config(dir.path().to_path_buf()), &task).unwrap();

      let results = super::super::search(
        &crate::test_helpers::make_test_config(dir.path().to_path_buf()),
        "important",
        false,
      )
      .unwrap();
      assert_eq!(results.tasks.len(), 1);
      assert_eq!(results.tasks[0].title, "Important Feature");
    }

    #[test]
    fn it_is_case_insensitive() {
      let dir = tempfile::tempdir().unwrap();
      let task = make_test_task("zyxwvutsrqponmlkzyxwvutsrqponmlk", "UPPERCASE Title");
      crate::store::write_task(&crate::test_helpers::make_test_config(dir.path().to_path_buf()), &task).unwrap();

      let results = super::super::search(
        &crate::test_helpers::make_test_config(dir.path().to_path_buf()),
        "uppercase",
        false,
      )
      .unwrap();
      assert_eq!(results.tasks.len(), 1);
    }

    #[test]
    fn it_returns_empty_when_no_match() {
      let dir = tempfile::tempdir().unwrap();
      let task = make_test_task("zyxwvutsrqponmlkzyxwvutsrqponmlk", "Some Task");
      crate::store::write_task(&crate::test_helpers::make_test_config(dir.path().to_path_buf()), &task).unwrap();

      let results = super::super::search(
        &crate::test_helpers::make_test_config(dir.path().to_path_buf()),
        "nonexistent",
        false,
      )
      .unwrap();
      assert_eq!(results.tasks.len(), 0);
      assert_eq!(results.artifacts.len(), 0);
      assert_eq!(results.iterations.len(), 0);
    }

    #[test]
    fn it_finds_iterations_by_title() {
      let dir = tempfile::tempdir().unwrap();
      let iteration = make_test_iteration("zyxwvutsrqponmlkzyxwvutsrqponmlk", "Sprint Alpha");
      crate::store::write_iteration(
        &crate::test_helpers::make_test_config(dir.path().to_path_buf()),
        &iteration,
      )
      .unwrap();

      let results = super::super::search(
        &crate::test_helpers::make_test_config(dir.path().to_path_buf()),
        "sprint",
        false,
      )
      .unwrap();
      assert_eq!(results.iterations.len(), 1);
      assert_eq!(results.iterations[0].title, "Sprint Alpha");
    }

    #[test]
    fn it_finds_iterations_by_description() {
      let dir = tempfile::tempdir().unwrap();
      let mut iteration = make_test_iteration("zyxwvutsrqponmlkzyxwvutsrqponmlk", "Sprint One");
      iteration.description = "This iteration focuses on the backend refactor".to_string();
      crate::store::write_iteration(
        &crate::test_helpers::make_test_config(dir.path().to_path_buf()),
        &iteration,
      )
      .unwrap();

      let results = super::super::search(
        &crate::test_helpers::make_test_config(dir.path().to_path_buf()),
        "refactor",
        false,
      )
      .unwrap();
      assert_eq!(results.iterations.len(), 1);
      assert_eq!(results.iterations[0].title, "Sprint One");
    }

    #[test]
    fn it_finds_iterations_by_tag_prefix() {
      let dir = tempfile::tempdir().unwrap();
      let mut iteration = make_test_iteration("zyxwvutsrqponmlkzyxwvutsrqponmlk", "Tagged Iteration");
      iteration.tags = vec!["release".to_string()];
      crate::store::write_iteration(
        &crate::test_helpers::make_test_config(dir.path().to_path_buf()),
        &iteration,
      )
      .unwrap();

      let results = super::super::search(
        &crate::test_helpers::make_test_config(dir.path().to_path_buf()),
        "tag:release",
        false,
      )
      .unwrap();
      assert_eq!(results.iterations.len(), 1);
      assert_eq!(results.iterations[0].title, "Tagged Iteration");
    }
  }
}
