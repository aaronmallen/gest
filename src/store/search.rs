use rayon::prelude::*;

use crate::{
  config::Settings,
  model::{Artifact, ArtifactFilter, Iteration, IterationFilter, Task, TaskFilter},
  store::search_query::{Filter, ParsedQuery},
};

/// Collected search results across entity types.
pub struct SearchResults {
  pub artifacts: Vec<Artifact>,
  pub iterations: Vec<Iteration>,
  pub tasks: Vec<Task>,
}

/// Perform a case-insensitive full-text search across tasks, artifacts, and iterations.
///
/// Supports expressive query syntax via [`super::search_query::parse`]:
/// - `is:<type>` to scope by entity type
/// - `tag:<name>` for exact tag match
/// - `status:<status>` for status match
/// - `type:<kind>` for artifact kind match
/// - `-<filter>` to negate any filter
pub fn search(config: &Settings, query: &str, show_all: bool) -> super::Result<SearchResults> {
  let parsed = super::search_query::parse(query);
  let text_lower = parsed.text.to_lowercase();

  let want_tasks = wants_entity_type(&parsed, "task");
  let want_artifacts = wants_entity_type(&parsed, "artifact");
  let want_iterations = wants_entity_type(&parsed, "iteration");

  let tasks = if want_tasks {
    let filter = TaskFilter {
      all: show_all,
      ..Default::default()
    };
    super::list_tasks(config, &filter)?
      .into_par_iter()
      .filter(|task| {
        if !matches_tag_filters(&parsed, &task.tags) {
          return false;
        }
        if !matches_status_filters(&parsed, &task.status.to_string()) {
          return false;
        }
        // Tasks have no kind — exclude if a type: include filter is present.
        if has_include_of_kind(&parsed, FilterKind::Type) {
          return false;
        }
        if has_exclude_of_kind(&parsed, FilterKind::Type) && !matches_type_exclude(&parsed) {
          return false;
        }
        matches_text(task, &text_lower)
      })
      .collect()
  } else {
    Vec::new()
  };

  let artifacts = if want_artifacts {
    let filter = ArtifactFilter {
      all: show_all,
      ..Default::default()
    };
    super::list_artifacts(config, &filter)?
      .into_par_iter()
      .filter(|artifact| {
        if !matches_tag_filters(&parsed, &artifact.tags) {
          return false;
        }
        // Artifacts have no status — exclude if a status: include filter is present.
        if has_include_of_kind(&parsed, FilterKind::Status) {
          return false;
        }
        if !matches_type_filters(&parsed, artifact.kind.as_deref().unwrap_or("")) {
          return false;
        }
        matches_text_artifact(artifact, &text_lower)
      })
      .collect()
  } else {
    Vec::new()
  };

  let iterations = if want_iterations {
    let filter = IterationFilter {
      all: show_all,
      ..Default::default()
    };
    super::list_iterations(config, &filter)?
      .into_par_iter()
      .filter(|iteration| {
        if !matches_tag_filters(&parsed, &iteration.tags) {
          return false;
        }
        if !matches_status_filters(&parsed, &iteration.status.to_string()) {
          return false;
        }
        // Iterations have no kind — exclude if a type: include filter is present.
        if has_include_of_kind(&parsed, FilterKind::Type) {
          return false;
        }
        if has_exclude_of_kind(&parsed, FilterKind::Type) && !matches_type_exclude(&parsed) {
          return false;
        }
        matches_text_iteration(iteration, &text_lower)
      })
      .collect()
  } else {
    Vec::new()
  };

  Ok(SearchResults {
    artifacts,
    iterations,
    tasks,
  })
}

// ---------------------------------------------------------------------------
// Filter helpers
// ---------------------------------------------------------------------------

/// Discriminant for filter kinds (used to check presence without matching values).
enum FilterKind {
  Status,
  Type,
}

/// Determine whether a given entity type should be included based on `is:` filters.
///
/// - No `is:` include filters → include all types (unless excluded).
/// - `is:` include filters present → only include listed types.
/// - `is:` exclude filters → exclude listed types.
fn wants_entity_type(parsed: &ParsedQuery, entity: &str) -> bool {
  let has_is_include = parsed.include.iter().any(|f| matches!(f, Filter::Is(_)));

  if has_is_include {
    // Must be in the include list.
    if !parsed.include.iter().any(|f| matches!(f, Filter::Is(v) if v == entity)) {
      return false;
    }
  }

  // Must not be in the exclude list.
  !parsed.exclude.iter().any(|f| matches!(f, Filter::Is(v) if v == entity))
}

/// Check tag filters (include OR-combines, exclude rejects any match).
fn matches_tag_filters(parsed: &ParsedQuery, tags: &[String]) -> bool {
  let tag_includes: Vec<&str> = parsed
    .include
    .iter()
    .filter_map(|f| match f {
      Filter::Tag(v) => Some(v.as_str()),
      _ => None,
    })
    .collect();

  if !tag_includes.is_empty()
    && !tag_includes
      .iter()
      .any(|tv| tags.iter().any(|t| t.to_lowercase() == *tv))
  {
    return false;
  }

  let tag_excludes: Vec<&str> = parsed
    .exclude
    .iter()
    .filter_map(|f| match f {
      Filter::Tag(v) => Some(v.as_str()),
      _ => None,
    })
    .collect();

  if tag_excludes
    .iter()
    .any(|tv| tags.iter().any(|t| t.to_lowercase() == *tv))
  {
    return false;
  }

  true
}

/// Check status filters (include OR-combines, exclude rejects any match).
fn matches_status_filters(parsed: &ParsedQuery, status: &str) -> bool {
  let status_lower = status.to_lowercase();

  let status_includes: Vec<&str> = parsed
    .include
    .iter()
    .filter_map(|f| match f {
      Filter::Status(v) => Some(v.as_str()),
      _ => None,
    })
    .collect();

  if !status_includes.is_empty() && !status_includes.iter().any(|sv| *sv == status_lower) {
    return false;
  }

  let status_excludes: Vec<&str> = parsed
    .exclude
    .iter()
    .filter_map(|f| match f {
      Filter::Status(v) => Some(v.as_str()),
      _ => None,
    })
    .collect();

  if status_excludes.iter().any(|sv| *sv == status_lower) {
    return false;
  }

  true
}

/// Check type (artifact kind) filters.
fn matches_type_filters(parsed: &ParsedQuery, kind: &str) -> bool {
  let kind_lower = kind.to_lowercase();

  let type_includes: Vec<&str> = parsed
    .include
    .iter()
    .filter_map(|f| match f {
      Filter::Type(v) => Some(v.as_str()),
      _ => None,
    })
    .collect();

  if !type_includes.is_empty() && !type_includes.iter().any(|tv| *tv == kind_lower) {
    return false;
  }

  let type_excludes: Vec<&str> = parsed
    .exclude
    .iter()
    .filter_map(|f| match f {
      Filter::Type(v) => Some(v.as_str()),
      _ => None,
    })
    .collect();

  if type_excludes.iter().any(|tv| *tv == kind_lower) {
    return false;
  }

  true
}

/// Whether any include filter of the given kind exists.
fn has_include_of_kind(parsed: &ParsedQuery, kind: FilterKind) -> bool {
  parsed.include.iter().any(|f| match kind {
    FilterKind::Status => matches!(f, Filter::Status(_)),
    FilterKind::Type => matches!(f, Filter::Type(_)),
  })
}

/// Whether any exclude filter of the given kind exists.
fn has_exclude_of_kind(parsed: &ParsedQuery, kind: FilterKind) -> bool {
  parsed.exclude.iter().any(|f| match kind {
    FilterKind::Status => matches!(f, Filter::Status(_)),
    FilterKind::Type => matches!(f, Filter::Type(_)),
  })
}

/// For entities without a kind field: if only exclude type filters exist, they
/// pass (because they can't match the excluded kind).
fn matches_type_exclude(_parsed: &ParsedQuery) -> bool {
  // Entities without a kind can never match -type:X, so they always pass.
  true
}

// ---------------------------------------------------------------------------
// Free-text matching helpers
// ---------------------------------------------------------------------------

fn matches_text(task: &Task, text_lower: &str) -> bool {
  if text_lower.is_empty() {
    return true;
  }
  contains_ignore_case(&task.title, text_lower)
    || contains_ignore_case(&task.description, text_lower)
    || task.tags.iter().any(|t| contains_ignore_case(t, text_lower))
    || contains_ignore_case(&task.status.to_string(), text_lower)
    || (!task.metadata.is_empty()
      && contains_ignore_case(&toml::to_string(&task.metadata).unwrap_or_default(), text_lower))
}

fn matches_text_artifact(artifact: &Artifact, text_lower: &str) -> bool {
  if text_lower.is_empty() {
    return true;
  }
  contains_ignore_case(&artifact.title, text_lower)
    || contains_ignore_case(&artifact.body, text_lower)
    || artifact.tags.iter().any(|t| contains_ignore_case(t, text_lower))
    || contains_ignore_case(artifact.kind.as_deref().unwrap_or(""), text_lower)
    || (!artifact.metadata.is_empty()
      && contains_ignore_case(
        &yaml_serde::to_string(&artifact.metadata).unwrap_or_default(),
        text_lower,
      ))
}

fn matches_text_iteration(iteration: &Iteration, text_lower: &str) -> bool {
  if text_lower.is_empty() {
    return true;
  }
  contains_ignore_case(&iteration.title, text_lower)
    || contains_ignore_case(&iteration.description, text_lower)
    || iteration.tags.iter().any(|t| contains_ignore_case(t, text_lower))
    || (!iteration.metadata.is_empty()
      && contains_ignore_case(&toml::to_string(&iteration.metadata).unwrap_or_default(), text_lower))
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
    fn it_excludes_entities_with_negated_is_filter() {
      let dir = tempfile::tempdir().unwrap();
      let cfg = crate::test_helpers::make_test_config(dir.path().to_path_buf());
      let task = make_test_task("kkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkk", "My Task");
      crate::store::write_task(&cfg, &task).unwrap();
      let artifact = make_test_artifact("llllllllllllllllllllllllllllllll", "My Artifact", "body");
      crate::store::write_artifact(&cfg, &artifact).unwrap();

      let results = super::super::search(&cfg, "-is:artifact My", false).unwrap();

      assert_eq!(results.tasks.len(), 1);
      assert_eq!(results.artifacts.len(), 0);
    }

    #[test]
    fn it_excludes_entities_with_negated_tag_filter() {
      let dir = tempfile::tempdir().unwrap();
      let cfg = crate::test_helpers::make_test_config(dir.path().to_path_buf());
      let mut task = make_test_task("kkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkk", "Tagged Task");
      task.tags = vec!["wip".to_string()];
      crate::store::write_task(&cfg, &task).unwrap();
      let task2 = make_test_task("nnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnn", "Other Task");
      crate::store::write_task(&cfg, &task2).unwrap();

      let results = super::super::search(&cfg, "-tag:wip Task", false).unwrap();

      assert_eq!(results.tasks.len(), 1);
      assert_eq!(results.tasks[0].title, "Other Task");
    }

    #[test]
    fn it_filters_artifacts_by_type() {
      let dir = tempfile::tempdir().unwrap();
      let cfg = crate::test_helpers::make_test_config(dir.path().to_path_buf());
      let mut artifact = make_test_artifact("llllllllllllllllllllllllllllllll", "My Spec", "body");
      artifact.kind = Some("spec".to_string());
      crate::store::write_artifact(&cfg, &artifact).unwrap();
      let mut artifact2 = make_test_artifact("nnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnn", "My RFC", "body");
      artifact2.kind = Some("rfc".to_string());
      crate::store::write_artifact(&cfg, &artifact2).unwrap();

      let results = super::super::search(&cfg, "type:spec", false).unwrap();

      assert_eq!(results.artifacts.len(), 1);
      assert_eq!(results.artifacts[0].title, "My Spec");
    }

    #[test]
    fn it_filters_by_status() {
      let dir = tempfile::tempdir().unwrap();
      let cfg = crate::test_helpers::make_test_config(dir.path().to_path_buf());
      let mut task = make_test_task("kkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkk", "Open Task");
      task.status = crate::model::task::Status::Open;
      crate::store::write_task(&cfg, &task).unwrap();
      let mut task2 = make_test_task("nnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnn", "Done Task");
      task2.status = crate::model::task::Status::Done;
      crate::store::write_task(&cfg, &task2).unwrap();

      let results = super::super::search(&cfg, "status:open", false).unwrap();

      assert_eq!(results.tasks.len(), 1);
      assert_eq!(results.tasks[0].title, "Open Task");
      // Artifacts have no status — should be excluded when status: filter is present.
      assert_eq!(results.artifacts.len(), 0);
    }

    #[test]
    fn it_filters_tasks_only_with_is_task() {
      let dir = tempfile::tempdir().unwrap();
      let cfg = crate::test_helpers::make_test_config(dir.path().to_path_buf());
      let task = make_test_task("kkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkk", "My Task");
      crate::store::write_task(&cfg, &task).unwrap();
      let artifact = make_test_artifact("llllllllllllllllllllllllllllllll", "My Artifact", "body");
      crate::store::write_artifact(&cfg, &artifact).unwrap();

      let results = super::super::search(&cfg, "is:task My", false).unwrap();

      assert_eq!(results.tasks.len(), 1);
      assert_eq!(results.artifacts.len(), 0);
      assert_eq!(results.iterations.len(), 0);
    }

    #[test]
    fn it_or_combines_multiple_is_filters() {
      let dir = tempfile::tempdir().unwrap();
      let cfg = crate::test_helpers::make_test_config(dir.path().to_path_buf());
      let task = make_test_task("kkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkk", "My Task");
      crate::store::write_task(&cfg, &task).unwrap();
      let artifact = make_test_artifact("llllllllllllllllllllllllllllllll", "My Artifact", "body");
      crate::store::write_artifact(&cfg, &artifact).unwrap();
      let mut iteration = make_test_iteration("mmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmm", "My Iteration");
      iteration.description = "body".to_string();
      crate::store::write_iteration(&cfg, &iteration).unwrap();

      let results = super::super::search(&cfg, "is:task is:artifact My", false).unwrap();

      assert_eq!(results.tasks.len(), 1);
      assert_eq!(results.artifacts.len(), 1);
      assert_eq!(results.iterations.len(), 0);
    }

    #[test]
    fn it_or_combines_multiple_tag_filters() {
      let dir = tempfile::tempdir().unwrap();
      let cfg = crate::test_helpers::make_test_config(dir.path().to_path_buf());
      let mut task1 = make_test_task("kkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkk", "Task A");
      task1.tags = vec!["alpha".to_string()];
      crate::store::write_task(&cfg, &task1).unwrap();
      let mut task2 = make_test_task("nnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnn", "Task B");
      task2.tags = vec!["beta".to_string()];
      crate::store::write_task(&cfg, &task2).unwrap();
      let task3 = make_test_task("pppppppppppppppppppppppppppppppp", "Task C");
      crate::store::write_task(&cfg, &task3).unwrap();

      let results = super::super::search(&cfg, "tag:alpha tag:beta", false).unwrap();

      assert_eq!(results.tasks.len(), 2);
    }

    #[test]
    fn it_and_combines_different_filter_types() {
      let dir = tempfile::tempdir().unwrap();
      let cfg = crate::test_helpers::make_test_config(dir.path().to_path_buf());
      let mut task = make_test_task("kkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkk", "Tagged Open");
      task.tags = vec!["urgent".to_string()];
      task.status = crate::model::task::Status::Open;
      crate::store::write_task(&cfg, &task).unwrap();
      let mut task2 = make_test_task("nnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnn", "Tagged Done");
      task2.tags = vec!["urgent".to_string()];
      task2.status = crate::model::task::Status::Done;
      crate::store::write_task(&cfg, &task2).unwrap();

      let results = super::super::search(&cfg, "is:task tag:urgent status:open", false).unwrap();

      assert_eq!(results.tasks.len(), 1);
      assert_eq!(results.tasks[0].title, "Tagged Open");
    }

    #[test]
    fn it_returns_all_types_with_no_filters() {
      let dir = tempfile::tempdir().unwrap();
      let cfg = crate::test_helpers::make_test_config(dir.path().to_path_buf());
      let task = make_test_task("kkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkk", "Shared Word");
      crate::store::write_task(&cfg, &task).unwrap();
      let artifact = make_test_artifact("llllllllllllllllllllllllllllllll", "Shared Word", "body");
      crate::store::write_artifact(&cfg, &artifact).unwrap();
      let mut iteration = make_test_iteration("mmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmm", "Shared Word");
      iteration.description = "".to_string();
      crate::store::write_iteration(&cfg, &iteration).unwrap();

      let results = super::super::search(&cfg, "Shared", false).unwrap();

      assert_eq!(results.tasks.len(), 1);
      assert_eq!(results.artifacts.len(), 1);
      assert_eq!(results.iterations.len(), 1);
    }
  }
}
