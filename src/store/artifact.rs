use std::fs;

use chrono::Utc;

use super::{
  Error,
  fs::{ensure_dirs, move_entity_file, next_id, resolve_id},
  helpers::{load_entities_from_dirs, read_entity_file},
};
use crate::{
  config::Settings,
  model::{Artifact, ArtifactFilter, ArtifactPatch, Id, NewArtifact},
};

/// Move an artifact to the archive, setting its `archived_at` timestamp.
pub fn archive_artifact(config: &Settings, id: &Id) -> super::Result<()> {
  let mut artifact = read_artifact(config, id)?;
  let now = Utc::now();
  artifact.archived_at = Some(now);
  artifact.updated_at = now;

  let content = serialize_artifact(&artifact)?;
  move_entity_file(
    config,
    &content,
    &config.storage().artifact_dir().join(format!("archive/{id}.md")),
    &config.storage().artifact_dir().join(format!("{id}.md")),
  )?;

  Ok(())
}

/// Persist a new artifact and return the fully-populated record.
pub fn create_artifact(config: &Settings, new: NewArtifact) -> super::Result<Artifact> {
  let now = Utc::now();
  let artifact = Artifact {
    archived_at: None,
    body: new.body,
    created_at: now,
    id: next_id(config)?,
    kind: new.kind,
    metadata: new.metadata,
    tags: new.tags,
    title: new.title,
    updated_at: now,
  };

  write_artifact(config, &artifact)?;
  Ok(artifact)
}

/// List artifacts matching the given filter criteria.
pub fn list_artifacts(config: &Settings, filter: &ArtifactFilter) -> super::Result<Vec<Artifact>> {
  let mut artifacts = load_entities_from_dirs(
    config.storage().artifact_dir(),
    &config.storage().artifact_dir().join("archive"),
    "md",
    filter.only_archived,
    filter.all || filter.only_archived,
    parse_artifact_file,
  )?;

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

/// Load a single artifact by exact ID, checking both active and archived directories.
pub fn read_artifact(config: &Settings, id: &Id) -> super::Result<Artifact> {
  let active = config.storage().artifact_dir().join(format!("{id}.md"));
  let archived = config.storage().artifact_dir().join(format!("archive/{id}.md"));

  read_entity_file(&active, &archived, "archived", "Artifact", id, parse_artifact_file)
}

/// Resolve a short ID prefix to a full artifact [`Id`].
pub fn resolve_artifact_id(config: &Settings, prefix: &str, show_all: bool) -> super::Result<Id> {
  log::debug!("resolving artifact ID prefix '{prefix}'");
  resolve_id(
    config.storage().artifact_dir(),
    Some(&config.storage().artifact_dir().join("archive")),
    "md",
    prefix,
    show_all,
    "Artifact",
  )
}

/// Apply a partial update to an existing artifact.
pub fn update_artifact(config: &Settings, id: &Id, patch: ArtifactPatch) -> super::Result<Artifact> {
  let mut artifact = read_artifact(config, id)?;

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
  write_artifact(config, &artifact)?;
  Ok(artifact)
}

/// Serialize and write an artifact, respecting its current location on disk.
///
/// If the artifact already exists in the archive directory, it is written there
/// to avoid creating a duplicate in the active directory.
pub fn write_artifact(config: &Settings, artifact: &Artifact) -> super::Result<()> {
  ensure_dirs(config)?;
  let content = serialize_artifact(artifact)?;
  let archived_path = config
    .storage()
    .artifact_dir()
    .join(format!("archive/{}.md", artifact.id));
  let path = if archived_path.exists() {
    archived_path
  } else {
    config.storage().artifact_dir().join(format!("{}.md", artifact.id))
  };
  log::trace!("writing artifact {} to {}", artifact.id, path.display());
  fs::write(path, content)?;
  Ok(())
}

/// Parse a YAML-frontmatter + markdown body artifact file into an [`Artifact`].
fn parse_artifact_file(content: &str) -> super::Result<Artifact> {
  let content = content
    .strip_prefix("---\n")
    .ok_or_else(|| Error::generic("Artifact file missing opening frontmatter delimiter"))?;

  let end = content
    .find("\n---\n")
    .ok_or_else(|| Error::generic("Artifact file missing closing frontmatter delimiter"))?;

  let yaml = &content[..end];
  let rest = &content[end + 5..];

  let mut artifact: Artifact = yaml_serde::from_str(yaml)?;

  let body = rest.strip_prefix('\n').unwrap_or(rest);
  artifact.body = body.to_string();

  Ok(artifact)
}

/// Serialize an artifact into the YAML-frontmatter + markdown body format.
fn serialize_artifact(artifact: &Artifact) -> super::Result<String> {
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
  use crate::{
    config::Settings,
    model::{Artifact, ArtifactFilter},
  };

  fn make_config(base: &std::path::Path) -> Settings {
    crate::test_helpers::make_test_config(base.to_path_buf())
  }

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

      crate::store::write_artifact(&make_config(dir.path()), &artifact).unwrap();
      let loaded = crate::store::read_artifact(&make_config(dir.path()), &artifact.id).unwrap();

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

      crate::store::write_artifact(&make_config(dir.path()), &artifact).unwrap();
      let loaded = crate::store::read_artifact(&make_config(dir.path()), &artifact.id).unwrap();

      assert_eq!(artifact.title, loaded.title);
      assert_eq!(artifact.body, loaded.body);
      assert_eq!(artifact.id, loaded.id);
    }
  }

  mod list_artifacts {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_excludes_archived_by_default() {
      let dir = tempfile::tempdir().unwrap();
      let a = make_test_artifact("zyxwvutsrqponmlkzyxwvutsrqponmlk", "To Archive", "");
      crate::store::write_artifact(&make_config(dir.path()), &a).unwrap();
      crate::store::archive_artifact(&make_config(dir.path()), &a.id).unwrap();

      let filter = ArtifactFilter::default();
      let artifacts = crate::store::list_artifacts(&make_config(dir.path()), &filter).unwrap();

      assert_eq!(artifacts.len(), 0);
    }

    #[test]
    fn it_includes_archived_when_requested() {
      let dir = tempfile::tempdir().unwrap();
      let active = make_test_artifact("zyxwvutsrqponmlkzyxwvutsrqponmlk", "Active", "");
      let to_archive = make_test_artifact("kkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkk", "Archived", "");
      crate::store::write_artifact(&make_config(dir.path()), &active).unwrap();
      crate::store::write_artifact(&make_config(dir.path()), &to_archive).unwrap();
      crate::store::archive_artifact(&make_config(dir.path()), &to_archive.id).unwrap();

      let filter = ArtifactFilter {
        all: true,
        ..Default::default()
      };
      let artifacts = crate::store::list_artifacts(&make_config(dir.path()), &filter).unwrap();
      assert_eq!(artifacts.len(), 2);
    }

    #[test]
    fn it_returns_active_artifacts() {
      let dir = tempfile::tempdir().unwrap();
      let a1 = make_test_artifact("zyxwvutsrqponmlkzyxwvutsrqponmlk", "Active One", "");
      let a2 = make_test_artifact("kkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkk", "Active Two", "");
      crate::store::write_artifact(&make_config(dir.path()), &a1).unwrap();
      crate::store::write_artifact(&make_config(dir.path()), &a2).unwrap();

      let filter = ArtifactFilter::default();
      let artifacts = crate::store::list_artifacts(&make_config(dir.path()), &filter).unwrap();

      assert_eq!(artifacts.len(), 2);
    }

    #[test]
    fn it_returns_only_archived_when_only_archived() {
      let dir = tempfile::tempdir().unwrap();
      let active = make_test_artifact("zyxwvutsrqponmlkzyxwvutsrqponmlk", "Active", "");
      let to_archive = make_test_artifact("kkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkk", "Archived", "");
      crate::store::write_artifact(&make_config(dir.path()), &active).unwrap();
      crate::store::write_artifact(&make_config(dir.path()), &to_archive).unwrap();
      crate::store::archive_artifact(&make_config(dir.path()), &to_archive.id).unwrap();

      let filter = ArtifactFilter {
        only_archived: true,
        ..Default::default()
      };
      let artifacts = crate::store::list_artifacts(&make_config(dir.path()), &filter).unwrap();

      assert_eq!(artifacts.len(), 1);
      assert_eq!(artifacts[0].title, "Archived");
    }
  }

  mod parse_artifact_file {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_parses_frontmatter_and_body() {
      let dir = tempfile::tempdir().unwrap();
      let artifact = make_test_artifact("zyxwvutsrqponmlkzyxwvutsrqponmlk", "Test Artifact", "Body text here");

      crate::store::write_artifact(&make_config(dir.path()), &artifact).unwrap();

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
    fn it_errors_not_found_for_archived_when_not_included() {
      let dir = tempfile::tempdir().unwrap();
      let artifact = make_test_artifact("zyxwvutsrqponmlkzyxwvutsrqponmlk", "Archived", "");
      crate::store::write_artifact(&make_config(dir.path()), &artifact).unwrap();
      crate::store::archive_artifact(&make_config(dir.path()), &artifact.id).unwrap();

      let result = crate::store::resolve_artifact_id(&make_config(dir.path()), "zyxw", false);

      assert!(result.is_err());
      let err = result.unwrap_err().to_string();
      assert!(err.contains("not found"), "Expected not found error, got: {err}");
      assert!(err.contains("--all"), "Expected archive hint, got: {err}");
    }

    #[test]
    fn it_falls_back_to_archived_when_no_active_match() {
      let dir = tempfile::tempdir().unwrap();
      let artifact = make_test_artifact("zyxwvutsrqponmlkzyxwvutsrqponmlk", "Archived", "");
      crate::store::write_artifact(&make_config(dir.path()), &artifact).unwrap();
      crate::store::archive_artifact(&make_config(dir.path()), &artifact.id).unwrap();

      let resolved = crate::store::resolve_artifact_id(&make_config(dir.path()), "zyxw", true).unwrap();

      assert_eq!(resolved.to_string(), "zyxwvutsrqponmlkzyxwvutsrqponmlk");
    }

    #[test]
    fn it_prefers_active_over_archived_with_shared_prefix() {
      let dir = tempfile::tempdir().unwrap();
      let active = make_test_artifact("zyxwvutsrqponmlkzyxwvutsrqponmlk", "Active", "");
      let to_archive = make_test_artifact("zyxwkkkkkkkkkkkkkkkkkkkkkkkkkkkk", "Archived", "");
      crate::store::write_artifact(&make_config(dir.path()), &active).unwrap();
      crate::store::write_artifact(&make_config(dir.path()), &to_archive).unwrap();
      crate::store::archive_artifact(&make_config(dir.path()), &to_archive.id).unwrap();

      let resolved = crate::store::resolve_artifact_id(&make_config(dir.path()), "zyxw", true).unwrap();

      assert_eq!(resolved.to_string(), "zyxwvutsrqponmlkzyxwvutsrqponmlk");
    }
  }

  mod update_artifact {
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::model::ArtifactPatch;

    #[test]
    fn it_keeps_active_artifact_in_active_dir() {
      let dir = tempfile::tempdir().unwrap();
      let a = make_test_artifact("zyxwvutsrqponmlkzyxwvutsrqponmlk", "Original", "body");
      crate::store::write_artifact(&make_config(dir.path()), &a).unwrap();

      let patch = ArtifactPatch {
        title: Some("Updated".to_string()),
        ..Default::default()
      };
      crate::store::update_artifact(&make_config(dir.path()), &a.id, patch).unwrap();

      assert!(
        dir
          .path()
          .join("artifacts/zyxwvutsrqponmlkzyxwvutsrqponmlk.md")
          .exists()
      );
      assert!(
        !dir
          .path()
          .join("artifacts/archive/zyxwvutsrqponmlkzyxwvutsrqponmlk.md")
          .exists()
      );
    }

    #[test]
    fn it_writes_to_archive_not_active() {
      let dir = tempfile::tempdir().unwrap();
      let a = make_test_artifact("zyxwvutsrqponmlkzyxwvutsrqponmlk", "Original", "body");
      crate::store::write_artifact(&make_config(dir.path()), &a).unwrap();
      crate::store::archive_artifact(&make_config(dir.path()), &a.id).unwrap();

      let patch = ArtifactPatch {
        title: Some("Updated".to_string()),
        ..Default::default()
      };
      crate::store::update_artifact(&make_config(dir.path()), &a.id, patch).unwrap();

      assert!(
        !dir
          .path()
          .join("artifacts/zyxwvutsrqponmlkzyxwvutsrqponmlk.md")
          .exists()
      );
      assert!(
        dir
          .path()
          .join("artifacts/archive/zyxwvutsrqponmlkzyxwvutsrqponmlk.md")
          .exists()
      );

      let loaded = crate::store::read_artifact(&make_config(dir.path()), &a.id).unwrap();
      assert_eq!(loaded.title, "Updated");
    }
  }
}
