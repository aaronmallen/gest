mod search;

use std::{
  fs,
  path::{Path, PathBuf},
};

use chrono::Utc;
pub use search::{SearchResults, search};

use crate::model::{Artifact, ArtifactFilter, ArtifactPatch, Id, NewArtifact, NewTask, Task, TaskFilter, TaskPatch};

pub fn archive_artifact(data_dir: &Path, id: &Id) -> crate::Result<()> {
  let mut artifact = read_artifact(data_dir, id)?;
  let now = Utc::now();
  artifact.archived_at = Some(now);
  artifact.updated_at = now;

  ensure_dirs(data_dir)?;
  let content = serialize_artifact(&artifact)?;
  let archive_path = data_dir.join(format!("artifacts/archive/{id}.md"));
  fs::write(archive_path, content)?;

  let active_path = data_dir.join(format!("artifacts/{id}.md"));
  if active_path.exists() {
    fs::remove_file(active_path)?;
  }

  Ok(())
}

pub fn artifact_path(data_dir: &Path, id: &Id) -> PathBuf {
  let active = data_dir.join(format!("artifacts/{id}.md"));
  let archived = data_dir.join(format!("artifacts/archive/{id}.md"));
  if archived.exists() && !active.exists() {
    archived
  } else {
    active
  }
}

pub fn create_artifact(data_dir: &Path, new: NewArtifact) -> crate::Result<Artifact> {
  let now = Utc::now();
  let artifact = Artifact {
    archived_at: None,
    body: new.body,
    created_at: now,
    id: Id::new(),
    kind: new.kind,
    metadata: new.metadata,
    tags: new.tags,
    title: new.title,
    updated_at: now,
  };

  write_artifact(data_dir, &artifact)?;
  Ok(artifact)
}

pub fn create_task(data_dir: &Path, new: NewTask) -> crate::Result<Task> {
  let now = Utc::now();
  let task = Task {
    resolved_at: None,
    created_at: now,
    description: new.description,
    id: Id::new(),
    links: new.links,
    metadata: new.metadata,
    status: new.status,
    tags: new.tags,
    title: new.title,
    updated_at: now,
  };

  write_task(data_dir, &task)?;

  if task.status.is_terminal() {
    resolve_task(data_dir, &task.id)?;
    return read_task(data_dir, &task.id);
  }

  Ok(task)
}

pub fn ensure_dirs(data_dir: &Path) -> crate::Result<()> {
  fs::create_dir_all(data_dir.join("artifacts"))?;
  fs::create_dir_all(data_dir.join("artifacts/archive"))?;
  fs::create_dir_all(data_dir.join("tasks"))?;
  fs::create_dir_all(data_dir.join("tasks/resolved"))?;
  Ok(())
}

pub fn is_task_resolved(data_dir: &Path, id: &Id) -> bool {
  let resolved_path = data_dir.join(format!("tasks/resolved/{id}.toml"));
  let active_path = data_dir.join(format!("tasks/{id}.toml"));
  resolved_path.exists() && !active_path.exists()
}

pub fn list_artifacts(data_dir: &Path, filter: &ArtifactFilter) -> crate::Result<Vec<Artifact>> {
  let mut artifacts = Vec::new();

  if !filter.only_archived {
    for path in read_dir_files(&data_dir.join("artifacts"), "md")? {
      let content = fs::read_to_string(&path)?;
      let artifact = parse_artifact_file(&content)?;
      artifacts.push(artifact);
    }
  }

  if filter.include_archived || filter.only_archived {
    for path in read_dir_files(&data_dir.join("artifacts/archive"), "md")? {
      let content = fs::read_to_string(&path)?;
      let artifact = parse_artifact_file(&content)?;
      artifacts.push(artifact);
    }
  }

  artifacts.retain(|artifact| {
    if let Some(ref kind) = filter.kind
      && artifact.kind.as_deref() != Some(kind.as_str())
    {
      return false;
    }
    if let Some(ref tag) = filter.tag
      && !artifact.tags.contains(tag)
    {
      return false;
    }
    true
  });

  Ok(artifacts)
}

pub fn list_tasks(data_dir: &Path, filter: &TaskFilter) -> crate::Result<Vec<Task>> {
  let mut tasks = Vec::new();

  for path in read_dir_files(&data_dir.join("tasks"), "toml")? {
    let content = fs::read_to_string(&path)?;
    let task: Task = toml::from_str(&content)?;
    tasks.push(task);
  }

  if filter.all {
    for path in read_dir_files(&data_dir.join("tasks/resolved"), "toml")? {
      let content = fs::read_to_string(&path)?;
      let task: Task = toml::from_str(&content)?;
      tasks.push(task);
    }
  }

  tasks.retain(|task| {
    if let Some(ref status) = filter.status
      && &task.status != status
    {
      return false;
    }
    if let Some(ref tag) = filter.tag
      && !task.tags.contains(tag)
    {
      return false;
    }
    true
  });

  Ok(tasks)
}

pub fn read_artifact(data_dir: &Path, id: &Id) -> crate::Result<Artifact> {
  let active = data_dir.join(format!("artifacts/{id}.md"));
  let archived = data_dir.join(format!("artifacts/archive/{id}.md"));

  let path = if active.exists() {
    active
  } else if archived.exists() {
    log::debug!("reading archived artifact {id}");
    archived
  } else {
    return Err(crate::Error::generic(format!("Artifact not found: '{id}'")));
  };

  log::trace!("reading artifact from {}", path.display());
  let content = fs::read_to_string(path)?;
  parse_artifact_file(&content)
}

pub fn read_task(data_dir: &Path, id: &Id) -> crate::Result<Task> {
  let active = data_dir.join(format!("tasks/{id}.toml"));
  let resolved = data_dir.join(format!("tasks/resolved/{id}.toml"));

  let path = if active.exists() {
    active
  } else if resolved.exists() {
    log::debug!("reading resolved task {id}");
    resolved
  } else {
    return Err(crate::Error::generic(format!("Task not found: '{id}'")));
  };

  log::trace!("reading task from {}", path.display());
  let content = fs::read_to_string(path)?;
  let task: Task = toml::from_str(&content)?;
  Ok(task)
}

pub fn resolve_artifact_id(data_dir: &Path, prefix: &str, include_archived: bool) -> crate::Result<Id> {
  log::debug!("resolving artifact ID prefix '{prefix}'");
  let active_matches = collect_prefix_matches(&data_dir.join("artifacts"), "md", prefix)?;

  match active_matches.len() {
    1 => {
      return active_matches[0].parse().map_err(|e: String| crate::Error::generic(e));
    }
    n if n > 1 => {
      let ids = active_matches.join(", ");
      return Err(crate::Error::generic(format!(
        "Ambiguous ID prefix '{prefix}', matches: {ids}"
      )));
    }
    _ => {}
  }

  if include_archived {
    let archived_matches = collect_prefix_matches(&data_dir.join("artifacts/archive"), "md", prefix)?;
    match archived_matches.len() {
      0 => {}
      1 => {
        return archived_matches[0]
          .parse()
          .map_err(|e: String| crate::Error::generic(e));
      }
      _ => {
        let ids = archived_matches.join(", ");
        return Err(crate::Error::generic(format!(
          "Ambiguous ID prefix '{prefix}', matches: {ids}"
        )));
      }
    }
  }

  let mut msg = format!("Artifact not found: '{prefix}'");
  if !include_archived {
    msg.push_str(" (try --include-archived)");
  }
  Err(crate::Error::generic(msg))
}

pub fn resolve_task(data_dir: &Path, id: &Id) -> crate::Result<()> {
  let mut task = read_task(data_dir, id)?;
  let now = Utc::now();
  task.resolved_at = Some(now);
  task.updated_at = now;

  ensure_dirs(data_dir)?;
  let content = toml::to_string(&task)?;
  let resolved_path = data_dir.join(format!("tasks/resolved/{id}.toml"));
  fs::write(resolved_path, content)?;

  let active_path = data_dir.join(format!("tasks/{id}.toml"));
  if active_path.exists() {
    fs::remove_file(active_path)?;
  }

  Ok(())
}

pub fn resolve_task_id(data_dir: &Path, prefix: &str, include_resolved: bool) -> crate::Result<Id> {
  log::debug!("resolving task ID prefix '{prefix}'");
  let active_matches = collect_prefix_matches(&data_dir.join("tasks"), "toml", prefix)?;

  match active_matches.len() {
    1 => {
      return active_matches[0].parse().map_err(|e: String| crate::Error::generic(e));
    }
    n if n > 1 => {
      let ids = active_matches.join(", ");
      return Err(crate::Error::generic(format!(
        "Ambiguous ID prefix '{prefix}', matches: {ids}"
      )));
    }
    _ => {}
  }

  if include_resolved {
    let resolved_matches = collect_prefix_matches(&data_dir.join("tasks/resolved"), "toml", prefix)?;
    match resolved_matches.len() {
      0 => {}
      1 => {
        return resolved_matches[0]
          .parse()
          .map_err(|e: String| crate::Error::generic(e));
      }
      _ => {
        let ids = resolved_matches.join(", ");
        return Err(crate::Error::generic(format!(
          "Ambiguous ID prefix '{prefix}', matches: {ids}"
        )));
      }
    }
  }

  let mut msg = format!("Task not found: '{prefix}'");
  if !include_resolved {
    msg.push_str(" (try --all)");
  }
  Err(crate::Error::generic(msg))
}

pub fn update_artifact(data_dir: &Path, id: &Id, patch: ArtifactPatch) -> crate::Result<Artifact> {
  let mut artifact = read_artifact(data_dir, id)?;

  if let Some(body) = patch.body {
    artifact.body = body;
  }
  if let Some(kind) = patch.kind {
    artifact.kind = Some(kind);
  }
  if let Some(metadata) = patch.metadata {
    artifact.metadata = metadata;
  }
  if let Some(tags) = patch.tags {
    artifact.tags = tags;
  }
  if let Some(title) = patch.title {
    artifact.title = title;
  }

  artifact.updated_at = Utc::now();
  write_artifact(data_dir, &artifact)?;
  Ok(artifact)
}

pub fn update_task(data_dir: &Path, id: &Id, patch: TaskPatch) -> crate::Result<Task> {
  let mut task = read_task(data_dir, id)?;
  let was_resolved = is_task_resolved(data_dir, id);

  if let Some(description) = patch.description {
    task.description = description;
  }
  if let Some(metadata) = patch.metadata {
    task.metadata = metadata;
  }
  if let Some(status) = patch.status {
    task.status = status;
  }
  if let Some(tags) = patch.tags {
    task.tags = tags;
  }
  if let Some(title) = patch.title {
    task.title = title;
  }

  task.updated_at = Utc::now();

  if task.status.is_terminal() && !was_resolved {
    write_task(data_dir, &task)?;
    resolve_task(data_dir, id)?;
    return read_task(data_dir, id);
  } else if !task.status.is_terminal() && was_resolved {
    unresolve_task(data_dir, id)?;
    task.resolved_at = None;
    write_task(data_dir, &task)?;
  } else {
    write_task(data_dir, &task)?;
  }

  Ok(task)
}

pub fn unresolve_task(data_dir: &Path, id: &Id) -> crate::Result<()> {
  let mut task = read_task(data_dir, id)?;
  task.resolved_at = None;
  task.updated_at = Utc::now();

  ensure_dirs(data_dir)?;
  let content = toml::to_string(&task)?;
  let active_path = data_dir.join(format!("tasks/{id}.toml"));
  fs::write(active_path, content)?;

  let resolved_path = data_dir.join(format!("tasks/resolved/{id}.toml"));
  if resolved_path.exists() {
    fs::remove_file(resolved_path)?;
  }

  Ok(())
}

pub fn write_artifact(data_dir: &Path, artifact: &Artifact) -> crate::Result<()> {
  ensure_dirs(data_dir)?;
  let content = serialize_artifact(artifact)?;
  let path = data_dir.join(format!("artifacts/{}.md", artifact.id));
  log::debug!("writing artifact {} to {}", artifact.id, path.display());
  fs::write(path, content)?;
  Ok(())
}

pub fn write_task(data_dir: &Path, task: &Task) -> crate::Result<()> {
  ensure_dirs(data_dir)?;
  let content = toml::to_string(task)?;
  let path = data_dir.join(format!("tasks/{}.toml", task.id));
  log::debug!("writing task {} to {}", task.id, path.display());
  fs::write(path, content)?;
  Ok(())
}

fn collect_prefix_matches(dir: &Path, extension: &str, prefix: &str) -> crate::Result<Vec<String>> {
  let mut matches = Vec::new();
  for path in read_dir_files(dir, extension)? {
    if let Some(stem) = path.file_stem().and_then(|s| s.to_str())
      && stem.starts_with(prefix)
    {
      matches.push(stem.to_string());
    }
  }
  Ok(matches)
}

fn parse_artifact_file(content: &str) -> crate::Result<Artifact> {
  let content = content
    .strip_prefix("---\n")
    .ok_or_else(|| crate::Error::generic("Artifact file missing opening frontmatter delimiter"))?;

  let end = content
    .find("\n---\n")
    .ok_or_else(|| crate::Error::generic("Artifact file missing closing frontmatter delimiter"))?;

  let yaml = &content[..end];
  let rest = &content[end + 5..];

  let mut artifact: Artifact = yaml_serde::from_str(yaml)?;

  let body = rest.strip_prefix('\n').unwrap_or(rest);
  artifact.body = body.to_string();

  Ok(artifact)
}

fn read_dir_files(dir: &Path, extension: &str) -> crate::Result<Vec<PathBuf>> {
  if !dir.exists() {
    return Ok(Vec::new());
  }

  let mut paths = Vec::new();
  for entry in fs::read_dir(dir)? {
    let entry = entry?;
    let path = entry.path();
    if path.is_file()
      && let Some(ext) = path.extension().and_then(|e| e.to_str())
      && ext == extension
    {
      paths.push(path);
    }
  }
  paths.sort();
  Ok(paths)
}

fn serialize_artifact(artifact: &Artifact) -> crate::Result<String> {
  let yaml = yaml_serde::to_string(artifact)?;
  let mut output = String::from("---\n");
  output.push_str(&yaml);
  output.push_str("---\n");
  if !artifact.body.is_empty() {
    output.push('\n');
    output.push_str(&artifact.body);
  }
  Ok(output)
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::model::{link::Link, task::Status};

  fn make_test_artifact(id: &str, title: &str, body: &str) -> Artifact {
    let mut a = crate::test_helpers::make_test_artifact(id);
    a.title = title.to_string();
    a.body = body.to_string();
    a
  }

  fn make_test_task(id: &str, title: &str) -> Task {
    let mut t = crate::test_helpers::make_test_task(id);
    t.title = title.to_string();
    t
  }

  mod resolve_task {
    use super::*;

    #[test]
    fn it_moves_file_to_resolved() {
      let dir = tempfile::tempdir().unwrap();
      let task = make_test_task("zyxwvutsrqponmlkzyxwvutsrqponmlk", "To Resolve");
      super::super::write_task(dir.path(), &task).unwrap();

      assert!(dir.path().join("tasks/zyxwvutsrqponmlkzyxwvutsrqponmlk.toml").exists());
      super::super::resolve_task(dir.path(), &task.id).unwrap();

      assert!(!dir.path().join("tasks/zyxwvutsrqponmlkzyxwvutsrqponmlk.toml").exists());
      assert!(
        dir
          .path()
          .join("tasks/resolved/zyxwvutsrqponmlkzyxwvutsrqponmlk.toml")
          .exists()
      );
    }

    #[test]
    fn it_sets_resolved_at() {
      let dir = tempfile::tempdir().unwrap();
      let task = make_test_task("zyxwvutsrqponmlkzyxwvutsrqponmlk", "To Resolve");
      super::super::write_task(dir.path(), &task).unwrap();

      super::super::resolve_task(dir.path(), &task.id).unwrap();

      let loaded = super::super::read_task(dir.path(), &task.id).unwrap();
      assert!(loaded.resolved_at.is_some());
    }
  }

  mod artifact_io {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_handles_empty_body() {
      let dir = tempfile::tempdir().unwrap();
      let artifact = make_test_artifact("kkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkk", "Empty Body", "");

      super::super::write_artifact(dir.path(), &artifact).unwrap();
      let loaded = super::super::read_artifact(dir.path(), &artifact.id).unwrap();

      assert_eq!(loaded.body, "");
      assert_eq!(loaded.title, "Empty Body");
    }

    #[test]
    fn it_roundtrips_artifact_with_frontmatter_and_body() {
      let dir = tempfile::tempdir().unwrap();
      let artifact = make_test_artifact(
        "zyxwvutsrqponmlkzyxwvutsrqponmlk",
        "My Artifact",
        "# Hello\n\nSome content here.",
      );

      super::super::write_artifact(dir.path(), &artifact).unwrap();
      let loaded = super::super::read_artifact(dir.path(), &artifact.id).unwrap();

      assert_eq!(artifact.title, loaded.title);
      assert_eq!(artifact.body, loaded.body);
      assert_eq!(artifact.id, loaded.id);
    }
  }

  mod ensure_dirs {

    #[test]
    fn it_creates_all_subdirectories() {
      let dir = tempfile::tempdir().unwrap();
      super::super::ensure_dirs(dir.path()).unwrap();

      assert!(dir.path().join("tasks").is_dir());
      assert!(dir.path().join("tasks/resolved").is_dir());
      assert!(dir.path().join("artifacts").is_dir());
      assert!(dir.path().join("artifacts/archive").is_dir());
    }

    #[test]
    fn it_is_idempotent() {
      let dir = tempfile::tempdir().unwrap();
      super::super::ensure_dirs(dir.path()).unwrap();
      super::super::ensure_dirs(dir.path()).unwrap();

      assert!(dir.path().join("tasks").is_dir());
    }
  }

  mod is_task_resolved {
    use super::*;

    #[test]
    fn it_returns_false_for_active_task() {
      let dir = tempfile::tempdir().unwrap();
      let task = make_test_task("zyxwvutsrqponmlkzyxwvutsrqponmlk", "Active");
      super::super::write_task(dir.path(), &task).unwrap();

      assert!(!super::super::is_task_resolved(dir.path(), &task.id));
    }

    #[test]
    fn it_returns_true_for_resolved_task() {
      let dir = tempfile::tempdir().unwrap();
      let task = make_test_task("zyxwvutsrqponmlkzyxwvutsrqponmlk", "Resolved");
      super::super::write_task(dir.path(), &task).unwrap();
      super::super::resolve_task(dir.path(), &task.id).unwrap();

      assert!(super::super::is_task_resolved(dir.path(), &task.id));
    }
  }

  mod list_artifacts {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_returns_active_artifacts() {
      let dir = tempfile::tempdir().unwrap();
      let a1 = make_test_artifact("zyxwvutsrqponmlkzyxwvutsrqponmlk", "Active One", "");
      let a2 = make_test_artifact("kkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkk", "Active Two", "");
      super::super::write_artifact(dir.path(), &a1).unwrap();
      super::super::write_artifact(dir.path(), &a2).unwrap();

      let filter = ArtifactFilter::default();
      let artifacts = super::super::list_artifacts(dir.path(), &filter).unwrap();
      assert_eq!(artifacts.len(), 2);
    }

    #[test]
    fn it_excludes_archived_by_default() {
      let dir = tempfile::tempdir().unwrap();
      let a = make_test_artifact("zyxwvutsrqponmlkzyxwvutsrqponmlk", "To Archive", "");
      super::super::write_artifact(dir.path(), &a).unwrap();
      super::super::archive_artifact(dir.path(), &a.id).unwrap();

      let filter = ArtifactFilter::default();
      let artifacts = super::super::list_artifacts(dir.path(), &filter).unwrap();
      assert_eq!(artifacts.len(), 0);
    }

    #[test]
    fn it_includes_archived_when_requested() {
      let dir = tempfile::tempdir().unwrap();
      let active = make_test_artifact("zyxwvutsrqponmlkzyxwvutsrqponmlk", "Active", "");
      let to_archive = make_test_artifact("kkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkk", "Archived", "");
      super::super::write_artifact(dir.path(), &active).unwrap();
      super::super::write_artifact(dir.path(), &to_archive).unwrap();
      super::super::archive_artifact(dir.path(), &to_archive.id).unwrap();

      let filter = ArtifactFilter {
        include_archived: true,
        ..Default::default()
      };
      let artifacts = super::super::list_artifacts(dir.path(), &filter).unwrap();
      assert_eq!(artifacts.len(), 2);
    }

    #[test]
    fn it_returns_only_archived_when_only_archived() {
      let dir = tempfile::tempdir().unwrap();
      let active = make_test_artifact("zyxwvutsrqponmlkzyxwvutsrqponmlk", "Active", "");
      let to_archive = make_test_artifact("kkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkk", "Archived", "");
      super::super::write_artifact(dir.path(), &active).unwrap();
      super::super::write_artifact(dir.path(), &to_archive).unwrap();
      super::super::archive_artifact(dir.path(), &to_archive.id).unwrap();

      let filter = ArtifactFilter {
        only_archived: true,
        ..Default::default()
      };
      let artifacts = super::super::list_artifacts(dir.path(), &filter).unwrap();
      assert_eq!(artifacts.len(), 1);
      assert_eq!(artifacts[0].title, "Archived");
    }
  }

  mod list_tasks {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_excludes_resolved_by_default() {
      let dir = tempfile::tempdir().unwrap();
      let task = make_test_task("zyxwvutsrqponmlkzyxwvutsrqponmlk", "Active Task");
      super::super::write_task(dir.path(), &task).unwrap();
      super::super::resolve_task(dir.path(), &task.id).unwrap();

      let filter = TaskFilter::default();
      let tasks = super::super::list_tasks(dir.path(), &filter).unwrap();
      assert_eq!(tasks.len(), 0);
    }

    #[test]
    fn it_filters_by_status() {
      let dir = tempfile::tempdir().unwrap();
      let mut task1 = make_test_task("zyxwvutsrqponmlkzyxwvutsrqponmlk", "Open Task");
      task1.status = Status::Open;
      let mut task2 = make_test_task("kkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkk", "Done Task");
      task2.status = Status::Done;
      super::super::write_task(dir.path(), &task1).unwrap();
      super::super::write_task(dir.path(), &task2).unwrap();

      let filter = TaskFilter {
        status: Some(Status::Done),
        ..Default::default()
      };
      let tasks = super::super::list_tasks(dir.path(), &filter).unwrap();
      assert_eq!(tasks.len(), 1);
      assert_eq!(tasks[0].title, "Done Task");
    }

    #[test]
    fn it_filters_by_tag() {
      let dir = tempfile::tempdir().unwrap();
      let mut task1 = make_test_task("zyxwvutsrqponmlkzyxwvutsrqponmlk", "Tagged");
      task1.tags = vec!["rust".to_string()];
      let task2 = make_test_task("kkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkk", "Untagged");
      super::super::write_task(dir.path(), &task1).unwrap();
      super::super::write_task(dir.path(), &task2).unwrap();

      let filter = TaskFilter {
        tag: Some("rust".to_string()),
        ..Default::default()
      };
      let tasks = super::super::list_tasks(dir.path(), &filter).unwrap();
      assert_eq!(tasks.len(), 1);
      assert_eq!(tasks[0].title, "Tagged");
    }

    #[test]
    fn it_includes_resolved_when_all() {
      let dir = tempfile::tempdir().unwrap();
      let task = make_test_task("zyxwvutsrqponmlkzyxwvutsrqponmlk", "Active Task");
      super::super::write_task(dir.path(), &task).unwrap();
      super::super::resolve_task(dir.path(), &task.id).unwrap();

      let filter = TaskFilter {
        all: true,
        ..Default::default()
      };
      let tasks = super::super::list_tasks(dir.path(), &filter).unwrap();
      assert_eq!(tasks.len(), 1);
    }

    #[test]
    fn it_returns_active_tasks() {
      let dir = tempfile::tempdir().unwrap();
      let task1 = make_test_task("zyxwvutsrqponmlkzyxwvutsrqponmlk", "Task One");
      let task2 = make_test_task("kkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkk", "Task Two");
      super::super::write_task(dir.path(), &task1).unwrap();
      super::super::write_task(dir.path(), &task2).unwrap();

      let filter = TaskFilter::default();
      let tasks = super::super::list_tasks(dir.path(), &filter).unwrap();
      assert_eq!(tasks.len(), 2);
    }
  }

  mod parse_artifact_file {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_parses_frontmatter_and_body() {
      let dir = tempfile::tempdir().unwrap();
      let artifact = make_test_artifact("zyxwvutsrqponmlkzyxwvutsrqponmlk", "Test Artifact", "Body text here");

      super::super::write_artifact(dir.path(), &artifact).unwrap();

      let content = std::fs::read_to_string(dir.path().join("artifacts/zyxwvutsrqponmlkzyxwvutsrqponmlk.md")).unwrap();
      let parsed = super::super::parse_artifact_file(&content).unwrap();

      assert_eq!(parsed.title, "Test Artifact");
      assert_eq!(parsed.body, "Body text here");
    }
  }

  mod resolve_artifact_id {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_prefers_active_over_archived_with_shared_prefix() {
      let dir = tempfile::tempdir().unwrap();
      let active = make_test_artifact("zyxwvutsrqponmlkzyxwvutsrqponmlk", "Active", "");
      let to_archive = make_test_artifact("zyxwkkkkkkkkkkkkkkkkkkkkkkkkkkkk", "Archived", "");
      super::super::write_artifact(dir.path(), &active).unwrap();
      super::super::write_artifact(dir.path(), &to_archive).unwrap();
      super::super::archive_artifact(dir.path(), &to_archive.id).unwrap();

      let resolved = super::super::resolve_artifact_id(dir.path(), "zyxw", true).unwrap();
      assert_eq!(resolved.to_string(), "zyxwvutsrqponmlkzyxwvutsrqponmlk");
    }

    #[test]
    fn it_falls_back_to_archived_when_no_active_match() {
      let dir = tempfile::tempdir().unwrap();
      let artifact = make_test_artifact("zyxwvutsrqponmlkzyxwvutsrqponmlk", "Archived", "");
      super::super::write_artifact(dir.path(), &artifact).unwrap();
      super::super::archive_artifact(dir.path(), &artifact.id).unwrap();

      let resolved = super::super::resolve_artifact_id(dir.path(), "zyxw", true).unwrap();
      assert_eq!(resolved.to_string(), "zyxwvutsrqponmlkzyxwvutsrqponmlk");
    }

    #[test]
    fn it_errors_not_found_for_archived_when_not_included() {
      let dir = tempfile::tempdir().unwrap();
      let artifact = make_test_artifact("zyxwvutsrqponmlkzyxwvutsrqponmlk", "Archived", "");
      super::super::write_artifact(dir.path(), &artifact).unwrap();
      super::super::archive_artifact(dir.path(), &artifact.id).unwrap();

      let result = super::super::resolve_artifact_id(dir.path(), "zyxw", false);
      assert!(result.is_err());
      let err = result.unwrap_err().to_string();
      assert!(err.contains("not found"), "Expected not found error, got: {err}");
      assert!(err.contains("--include-archived"), "Expected archive hint, got: {err}");
    }
  }

  mod resolve_task_id {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_errors_on_ambiguous_prefix() {
      let dir = tempfile::tempdir().unwrap();
      let task1 = make_test_task("zyxwvutsrqponmlkzyxwvutsrqponmlk", "Task 1");
      let task2 = make_test_task("zyxwkkkkkkkkkkkkkkkkkkkkkkkkkkkk", "Task 2");
      super::super::write_task(dir.path(), &task1).unwrap();
      super::super::write_task(dir.path(), &task2).unwrap();

      let result = super::super::resolve_task_id(dir.path(), "zyxw", false);
      assert!(result.is_err());
      let err = result.unwrap_err().to_string();
      assert!(err.contains("Ambiguous"), "Expected ambiguous error, got: {err}");
    }

    #[test]
    fn it_errors_on_not_found() {
      let dir = tempfile::tempdir().unwrap();
      super::super::ensure_dirs(dir.path()).unwrap();

      let result = super::super::resolve_task_id(dir.path(), "nnnn", false);
      assert!(result.is_err());
      let err = result.unwrap_err().to_string();
      assert!(err.contains("not found"), "Expected not found error, got: {err}");
      assert!(err.contains("--all"), "Expected --all hint, got: {err}");
    }

    #[test]
    fn it_resolves_unique_prefix() {
      let dir = tempfile::tempdir().unwrap();
      let task = make_test_task("zyxwvutsrqponmlkzyxwvutsrqponmlk", "Test");
      super::super::write_task(dir.path(), &task).unwrap();

      let resolved = super::super::resolve_task_id(dir.path(), "zyxw", false).unwrap();
      assert_eq!(resolved.to_string(), "zyxwvutsrqponmlkzyxwvutsrqponmlk");
    }

    #[test]
    fn it_prefers_active_over_resolved_with_shared_prefix() {
      let dir = tempfile::tempdir().unwrap();
      let active = make_test_task("zyxwvutsrqponmlkzyxwvutsrqponmlk", "Active");
      let to_resolve = make_test_task("zyxwkkkkkkkkkkkkkkkkkkkkkkkkkkkk", "Resolved");
      super::super::write_task(dir.path(), &active).unwrap();
      super::super::write_task(dir.path(), &to_resolve).unwrap();
      super::super::resolve_task(dir.path(), &to_resolve.id).unwrap();

      let resolved = super::super::resolve_task_id(dir.path(), "zyxw", true).unwrap();
      assert_eq!(resolved.to_string(), "zyxwvutsrqponmlkzyxwvutsrqponmlk");
    }

    #[test]
    fn it_falls_back_to_resolved_when_no_active_match() {
      let dir = tempfile::tempdir().unwrap();
      let task = make_test_task("zyxwvutsrqponmlkzyxwvutsrqponmlk", "Resolved");
      super::super::write_task(dir.path(), &task).unwrap();
      super::super::resolve_task(dir.path(), &task.id).unwrap();

      let resolved = super::super::resolve_task_id(dir.path(), "zyxw", true).unwrap();
      assert_eq!(resolved.to_string(), "zyxwvutsrqponmlkzyxwvutsrqponmlk");
    }

    #[test]
    fn it_errors_not_found_for_resolved_when_not_included() {
      let dir = tempfile::tempdir().unwrap();
      let task = make_test_task("zyxwvutsrqponmlkzyxwvutsrqponmlk", "Resolved");
      super::super::write_task(dir.path(), &task).unwrap();
      super::super::resolve_task(dir.path(), &task.id).unwrap();

      let result = super::super::resolve_task_id(dir.path(), "zyxw", false);
      assert!(result.is_err());
      let err = result.unwrap_err().to_string();
      assert!(err.contains("not found"), "Expected not found error, got: {err}");
      assert!(err.contains("--all"), "Expected --all hint, got: {err}");
    }

    #[test]
    fn it_errors_ambiguous_for_multiple_active_without_checking_resolved() {
      let dir = tempfile::tempdir().unwrap();
      let task1 = make_test_task("zyxwvutsrqponmlkzyxwvutsrqponmlk", "Active 1");
      let task2 = make_test_task("zyxwkkkkkkkkkkkkkkkkkkkkkkkkkkkk", "Active 2");
      let task3 = make_test_task("zyxwmmmmmmmmmmmmmmmmmmmmmmmmmmmm", "Resolved");
      super::super::write_task(dir.path(), &task1).unwrap();
      super::super::write_task(dir.path(), &task2).unwrap();
      super::super::write_task(dir.path(), &task3).unwrap();
      super::super::resolve_task(dir.path(), &task3.id).unwrap();

      let result = super::super::resolve_task_id(dir.path(), "zyxw", true);
      assert!(result.is_err());
      let err = result.unwrap_err().to_string();
      assert!(err.contains("Ambiguous"), "Expected ambiguous error, got: {err}");
      assert!(
        !err.contains("zyxwmmmmmmmmmmmmmmmmmmmmmmmmmmmm"),
        "Should not include archived ID in error: {err}"
      );
    }

    #[test]
    fn it_errors_ambiguous_for_multiple_resolved_matches() {
      let dir = tempfile::tempdir().unwrap();
      let task1 = make_test_task("zyxwvutsrqponmlkzyxwvutsrqponmlk", "Resolved 1");
      let task2 = make_test_task("zyxwkkkkkkkkkkkkkkkkkkkkkkkkkkkk", "Resolved 2");
      super::super::write_task(dir.path(), &task1).unwrap();
      super::super::write_task(dir.path(), &task2).unwrap();
      super::super::resolve_task(dir.path(), &task1.id).unwrap();
      super::super::resolve_task(dir.path(), &task2.id).unwrap();

      let result = super::super::resolve_task_id(dir.path(), "zyxw", true);
      assert!(result.is_err());
      let err = result.unwrap_err().to_string();
      assert!(err.contains("Ambiguous"), "Expected ambiguous error, got: {err}");
    }
  }

  mod task_io {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_roundtrips_task_with_links_and_metadata() {
      let dir = tempfile::tempdir().unwrap();
      let mut task = make_test_task("zyxwvutsrqponmlkzyxwvutsrqponmlk", "Test Task");
      task.description = "A description".to_string();
      task.links = vec![Link {
        ref_: "https://example.com".to_string(),
        rel: crate::model::RelationshipType::RelatesTo,
      }];
      task.metadata = {
        let mut table = toml::Table::new();
        table.insert("priority".to_string(), toml::Value::String("high".to_string()));
        table
      };
      task.tags = vec!["rust".to_string(), "test".to_string()];

      super::super::write_task(dir.path(), &task).unwrap();
      let loaded = super::super::read_task(dir.path(), &task.id).unwrap();

      assert_eq!(task, loaded);
    }
  }

  mod unresolve_task {
    use super::*;

    #[test]
    fn it_clears_resolved_at() {
      let dir = tempfile::tempdir().unwrap();
      let task = make_test_task("zyxwvutsrqponmlkzyxwvutsrqponmlk", "To Unresolve");
      super::super::write_task(dir.path(), &task).unwrap();
      super::super::resolve_task(dir.path(), &task.id).unwrap();

      let resolved = super::super::read_task(dir.path(), &task.id).unwrap();
      assert!(resolved.resolved_at.is_some());

      super::super::unresolve_task(dir.path(), &task.id).unwrap();

      let unresolved = super::super::read_task(dir.path(), &task.id).unwrap();
      assert!(unresolved.resolved_at.is_none());
    }

    #[test]
    fn it_moves_file_from_resolved_to_active() {
      let dir = tempfile::tempdir().unwrap();
      let task = make_test_task("zyxwvutsrqponmlkzyxwvutsrqponmlk", "To Unresolve");
      super::super::write_task(dir.path(), &task).unwrap();
      super::super::resolve_task(dir.path(), &task.id).unwrap();

      assert!(!dir.path().join("tasks/zyxwvutsrqponmlkzyxwvutsrqponmlk.toml").exists());
      assert!(
        dir
          .path()
          .join("tasks/resolved/zyxwvutsrqponmlkzyxwvutsrqponmlk.toml")
          .exists()
      );

      super::super::unresolve_task(dir.path(), &task.id).unwrap();

      assert!(dir.path().join("tasks/zyxwvutsrqponmlkzyxwvutsrqponmlk.toml").exists());
      assert!(
        !dir
          .path()
          .join("tasks/resolved/zyxwvutsrqponmlkzyxwvutsrqponmlk.toml")
          .exists()
      );
    }
  }
}
