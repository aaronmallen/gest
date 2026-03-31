use rayon::prelude::*;

use crate::{
  config::storage::DataLayout,
  model::{Artifact, ArtifactFilter, Task, TaskFilter},
};

/// Collected search results across entity types.
pub struct SearchResults {
  pub artifacts: Vec<Artifact>,
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
pub fn search(layout: &DataLayout, query: &str, show_all: bool) -> super::Result<SearchResults> {
  let query_lower = query.to_lowercase();

  let task_filter = TaskFilter {
    all: show_all,
    ..Default::default()
  };
  let all_tasks = super::list_tasks(layout, &task_filter)?;

  let artifact_filter = ArtifactFilter {
    show_all,
    ..Default::default()
  };
  let all_artifacts = super::list_artifacts(layout, &artifact_filter)?;

  let tasks: Vec<Task> = all_tasks
    .into_par_iter()
    .filter(|task| {
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

  Ok(SearchResults {
    artifacts,
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
        &crate::config::storage::DataLayout::new(&crate::config::storage::Settings::default(), dir.path()),
        &artifact,
      )
      .unwrap();

      let results = super::super::search(
        &crate::config::storage::DataLayout::new(&crate::config::storage::Settings::default(), dir.path()),
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
      crate::store::write_task(
        &crate::config::storage::DataLayout::new(&crate::config::storage::Settings::default(), dir.path()),
        &task,
      )
      .unwrap();

      let results = super::super::search(
        &crate::config::storage::DataLayout::new(&crate::config::storage::Settings::default(), dir.path()),
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
      crate::store::write_task(
        &crate::config::storage::DataLayout::new(&crate::config::storage::Settings::default(), dir.path()),
        &task,
      )
      .unwrap();

      let results = super::super::search(
        &crate::config::storage::DataLayout::new(&crate::config::storage::Settings::default(), dir.path()),
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
      crate::store::write_task(
        &crate::config::storage::DataLayout::new(&crate::config::storage::Settings::default(), dir.path()),
        &task,
      )
      .unwrap();

      let results = super::super::search(
        &crate::config::storage::DataLayout::new(&crate::config::storage::Settings::default(), dir.path()),
        "nonexistent",
        false,
      )
      .unwrap();
      assert_eq!(results.tasks.len(), 0);
      assert_eq!(results.artifacts.len(), 0);
    }
  }
}
