use std::path::Path;

use rayon::prelude::*;

use crate::model::{Artifact, ArtifactFilter, Task, TaskFilter};

pub struct SearchResults {
  pub artifacts: Vec<Artifact>,
  pub tasks: Vec<Task>,
}

pub fn search(data_dir: &Path, query: &str, show_all: bool) -> crate::Result<SearchResults> {
  let query_lower = query.to_lowercase();

  let task_filter = TaskFilter {
    all: show_all,
    ..Default::default()
  };
  let all_tasks = super::list_tasks(data_dir, &task_filter)?;

  let artifact_filter = ArtifactFilter {
    show_all,
    ..Default::default()
  };
  let all_artifacts = super::list_artifacts(data_dir, &artifact_filter)?;

  let tasks: Vec<Task> = all_tasks
    .into_par_iter()
    .filter(|task| {
      let searchable = format!(
        "{} {} {} {} {}",
        task.title,
        task.description,
        task.tags.join(" "),
        task.status,
        toml::to_string(&task.metadata).unwrap_or_default(),
      );
      searchable.to_lowercase().contains(&query_lower)
    })
    .collect();

  let artifacts: Vec<Artifact> = all_artifacts
    .into_par_iter()
    .filter(|artifact| {
      let searchable = format!(
        "{} {} {} {} {}",
        artifact.title,
        artifact.body,
        artifact.tags.join(" "),
        artifact.kind.as_deref().unwrap_or(""),
        yaml_serde::to_string(&artifact.metadata).unwrap_or_default(),
      );
      searchable.to_lowercase().contains(&query_lower)
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
      crate::store::write_artifact(dir.path(), &artifact).unwrap();

      let results = super::super::search(dir.path(), "secret", false).unwrap();
      assert_eq!(results.artifacts.len(), 1);
      assert_eq!(results.artifacts[0].title, "Notes");
    }

    #[test]
    fn it_finds_tasks_by_title() {
      let dir = tempfile::tempdir().unwrap();
      let task = make_test_task("zyxwvutsrqponmlkzyxwvutsrqponmlk", "Important Feature");
      crate::store::write_task(dir.path(), &task).unwrap();

      let results = super::super::search(dir.path(), "important", false).unwrap();
      assert_eq!(results.tasks.len(), 1);
      assert_eq!(results.tasks[0].title, "Important Feature");
    }

    #[test]
    fn it_is_case_insensitive() {
      let dir = tempfile::tempdir().unwrap();
      let task = make_test_task("zyxwvutsrqponmlkzyxwvutsrqponmlk", "UPPERCASE Title");
      crate::store::write_task(dir.path(), &task).unwrap();

      let results = super::super::search(dir.path(), "uppercase", false).unwrap();
      assert_eq!(results.tasks.len(), 1);
    }

    #[test]
    fn it_returns_empty_when_no_match() {
      let dir = tempfile::tempdir().unwrap();
      let task = make_test_task("zyxwvutsrqponmlkzyxwvutsrqponmlk", "Some Task");
      crate::store::write_task(dir.path(), &task).unwrap();

      let results = super::super::search(dir.path(), "nonexistent", false).unwrap();
      assert_eq!(results.tasks.len(), 0);
      assert_eq!(results.artifacts.len(), 0);
    }
  }
}
