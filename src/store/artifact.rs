use std::{
  fs,
  path::{Path, PathBuf},
};

use chrono::Utc;

use super::fs::{ensure_dirs, read_dir_files, resolve_id};
use crate::model::{Artifact, ArtifactFilter, ArtifactPatch, Id, NewArtifact};

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

pub fn list_artifacts(data_dir: &Path, filter: &ArtifactFilter) -> crate::Result<Vec<Artifact>> {
  let mut artifacts = Vec::new();

  if !filter.only_archived {
    for path in read_dir_files(&data_dir.join("artifacts"), "md")? {
      let content = fs::read_to_string(&path)?;
      let artifact = parse_artifact_file(&content)?;
      artifacts.push(artifact);
    }
  }

  if filter.show_all || filter.only_archived {
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

pub fn resolve_artifact_id(data_dir: &Path, prefix: &str, show_all: bool) -> crate::Result<Id> {
  log::debug!("resolving artifact ID prefix '{prefix}'");
  resolve_id(
    &data_dir.join("artifacts"),
    Some(&data_dir.join("artifacts/archive")),
    "md",
    prefix,
    show_all,
    "Artifact",
  )
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

pub fn write_artifact(data_dir: &Path, artifact: &Artifact) -> crate::Result<()> {
  ensure_dirs(data_dir)?;
  let content = serialize_artifact(artifact)?;
  let path = data_dir.join(format!("artifacts/{}.md", artifact.id));
  log::trace!("writing artifact {} to {}", artifact.id, path.display());
  fs::write(path, content)?;
  Ok(())
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
  use crate::model::{Artifact, ArtifactFilter};

  fn make_test_artifact(id: &str, title: &str, body: &str) -> Artifact {
    Artifact {
      title: title.to_string(),
      body: body.to_string(),
      ..crate::test_helpers::make_test_artifact(id)
    }
  }

  mod artifact_io {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_handles_empty_body() {
      let dir = tempfile::tempdir().unwrap();
      let artifact = make_test_artifact("kkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkk", "Empty Body", "");

      crate::store::write_artifact(dir.path(), &artifact).unwrap();
      let loaded = crate::store::read_artifact(dir.path(), &artifact.id).unwrap();

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

      crate::store::write_artifact(dir.path(), &artifact).unwrap();
      let loaded = crate::store::read_artifact(dir.path(), &artifact.id).unwrap();

      assert_eq!(artifact.title, loaded.title);
      assert_eq!(artifact.body, loaded.body);
      assert_eq!(artifact.id, loaded.id);
    }
  }

  mod parse_artifact_file {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_parses_frontmatter_and_body() {
      let dir = tempfile::tempdir().unwrap();
      let artifact = make_test_artifact("zyxwvutsrqponmlkzyxwvutsrqponmlk", "Test Artifact", "Body text here");

      crate::store::write_artifact(dir.path(), &artifact).unwrap();

      let content = std::fs::read_to_string(dir.path().join("artifacts/zyxwvutsrqponmlkzyxwvutsrqponmlk.md")).unwrap();
      let parsed = super::super::parse_artifact_file(&content).unwrap();

      assert_eq!(parsed.title, "Test Artifact");
      assert_eq!(parsed.body, "Body text here");
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
      crate::store::write_artifact(dir.path(), &a1).unwrap();
      crate::store::write_artifact(dir.path(), &a2).unwrap();

      let filter = ArtifactFilter::default();
      let artifacts = crate::store::list_artifacts(dir.path(), &filter).unwrap();
      assert_eq!(artifacts.len(), 2);
    }

    #[test]
    fn it_excludes_archived_by_default() {
      let dir = tempfile::tempdir().unwrap();
      let a = make_test_artifact("zyxwvutsrqponmlkzyxwvutsrqponmlk", "To Archive", "");
      crate::store::write_artifact(dir.path(), &a).unwrap();
      crate::store::archive_artifact(dir.path(), &a.id).unwrap();

      let filter = ArtifactFilter::default();
      let artifacts = crate::store::list_artifacts(dir.path(), &filter).unwrap();
      assert_eq!(artifacts.len(), 0);
    }

    #[test]
    fn it_includes_archived_when_requested() {
      let dir = tempfile::tempdir().unwrap();
      let active = make_test_artifact("zyxwvutsrqponmlkzyxwvutsrqponmlk", "Active", "");
      let to_archive = make_test_artifact("kkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkk", "Archived", "");
      crate::store::write_artifact(dir.path(), &active).unwrap();
      crate::store::write_artifact(dir.path(), &to_archive).unwrap();
      crate::store::archive_artifact(dir.path(), &to_archive.id).unwrap();

      let filter = ArtifactFilter {
        show_all: true,
        ..Default::default()
      };
      let artifacts = crate::store::list_artifacts(dir.path(), &filter).unwrap();
      assert_eq!(artifacts.len(), 2);
    }

    #[test]
    fn it_returns_only_archived_when_only_archived() {
      let dir = tempfile::tempdir().unwrap();
      let active = make_test_artifact("zyxwvutsrqponmlkzyxwvutsrqponmlk", "Active", "");
      let to_archive = make_test_artifact("kkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkk", "Archived", "");
      crate::store::write_artifact(dir.path(), &active).unwrap();
      crate::store::write_artifact(dir.path(), &to_archive).unwrap();
      crate::store::archive_artifact(dir.path(), &to_archive.id).unwrap();

      let filter = ArtifactFilter {
        only_archived: true,
        ..Default::default()
      };
      let artifacts = crate::store::list_artifacts(dir.path(), &filter).unwrap();
      assert_eq!(artifacts.len(), 1);
      assert_eq!(artifacts[0].title, "Archived");
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
      crate::store::write_artifact(dir.path(), &active).unwrap();
      crate::store::write_artifact(dir.path(), &to_archive).unwrap();
      crate::store::archive_artifact(dir.path(), &to_archive.id).unwrap();

      let resolved = crate::store::resolve_artifact_id(dir.path(), "zyxw", true).unwrap();
      assert_eq!(resolved.to_string(), "zyxwvutsrqponmlkzyxwvutsrqponmlk");
    }

    #[test]
    fn it_falls_back_to_archived_when_no_active_match() {
      let dir = tempfile::tempdir().unwrap();
      let artifact = make_test_artifact("zyxwvutsrqponmlkzyxwvutsrqponmlk", "Archived", "");
      crate::store::write_artifact(dir.path(), &artifact).unwrap();
      crate::store::archive_artifact(dir.path(), &artifact.id).unwrap();

      let resolved = crate::store::resolve_artifact_id(dir.path(), "zyxw", true).unwrap();
      assert_eq!(resolved.to_string(), "zyxwvutsrqponmlkzyxwvutsrqponmlk");
    }

    #[test]
    fn it_errors_not_found_for_archived_when_not_included() {
      let dir = tempfile::tempdir().unwrap();
      let artifact = make_test_artifact("zyxwvutsrqponmlkzyxwvutsrqponmlk", "Archived", "");
      crate::store::write_artifact(dir.path(), &artifact).unwrap();
      crate::store::archive_artifact(dir.path(), &artifact.id).unwrap();

      let result = crate::store::resolve_artifact_id(dir.path(), "zyxw", false);
      assert!(result.is_err());
      let err = result.unwrap_err().to_string();
      assert!(err.contains("not found"), "Expected not found error, got: {err}");
      assert!(err.contains("--all"), "Expected archive hint, got: {err}");
    }
  }
}
