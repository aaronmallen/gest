//! v0.4.x flat-file migration logic.

use std::{
  collections::HashMap,
  io::{Error as IoError, ErrorKind},
  path::{Path, PathBuf},
};

use serde::Deserialize;
use serde_json::Value as JsonValue;

use crate::{
  AppContext,
  cli::Error,
  store::{
    model::primitives::{AuthorType, EntityType, Id, RelationshipType, TaskStatus},
    repo,
  },
};

/// Counters for reporting migration results.
#[derive(Default)]
struct MigrationCounts {
  tasks: usize,
  artifacts: usize,
  iterations: usize,
  notes: usize,
  relationships: usize,
  skipped: usize,
}

/// Run the v0.4 migration from flat-file `.gest/` directory into SQLite.
pub async fn run(context: &AppContext, source: &Path) -> Result<(), Error> {
  if !source.is_dir() {
    return Err(Error::Io(IoError::new(
      ErrorKind::NotFound,
      format!("source directory does not exist: {}", source.display()),
    )));
  }

  let cwd = std::env::current_dir()?;
  let conn = context.store().connect().await?;

  let project_id = match context.project_id() {
    Some(id) => id.clone(),
    None => {
      let project = repo::project::create(&conn, &cwd).await?;
      project.id().clone()
    }
  };

  let mut counts = MigrationCounts::default();

  // Tasks first — build old→new ID mapping for iteration task associations.
  let task_id_map = migrate_tasks(&conn, &project_id, source, &mut counts).await?;
  migrate_artifacts(&conn, &project_id, source, &mut counts).await?;
  migrate_iterations(&conn, &project_id, source, &mut counts, &task_id_map).await?;

  log::info!(
    "migrated {} tasks, {} artifacts, {} iterations, {} notes, {} relationships",
    counts.tasks,
    counts.artifacts,
    counts.iterations,
    counts.notes,
    counts.relationships,
  );
  println!(
    "migrated {} tasks, {} artifacts, {} iterations, {} notes, {} relationships",
    counts.tasks, counts.artifacts, counts.iterations, counts.notes, counts.relationships,
  );
  if counts.skipped > 0 {
    println!("skipped {} files (run with RUST_LOG=warn for details)", counts.skipped);
  }
  Ok(())
}

// ── Entity migration ────────────────────────────────────────────────────────────

/// Migrate tasks and return a mapping of old ID string → new Id.
async fn migrate_tasks(
  conn: &libsql::Connection,
  project_id: &Id,
  source: &Path,
  counts: &mut MigrationCounts,
) -> Result<HashMap<String, Id>, Error> {
  let mut id_map = HashMap::new();

  for dir in [source.join("tasks"), source.join("tasks/resolved")] {
    if !dir.is_dir() {
      continue;
    }
    for path in read_files_with_ext(&dir, "toml")? {
      let content = std::fs::read_to_string(&path)?;
      match toml::from_str::<LegacyTask>(&content) {
        Ok(task) => {
          let old_id = task.id.clone();
          let assigned_to = match &task.assigned_to {
            Some(name) => {
              let author = repo::author::find_or_create(conn, name, None, AuthorType::Human).await?;
              Some(author.id().clone())
            }
            None => None,
          };

          let status: Option<TaskStatus> = task.status.parse().ok();
          let metadata = toml_table_to_json(&task.metadata);

          let created = repo::task::create(
            conn,
            project_id,
            &crate::store::model::task::New {
              assigned_to,
              description: task.description.unwrap_or_default(),
              metadata,
              priority: task.priority,
              status,
              title: task.title,
            },
          )
          .await?;

          let new_id = created.id().clone();

          // Tags
          for tag in task.tags.unwrap_or_default() {
            repo::tag::attach(conn, EntityType::Task, &new_id, &tag).await?;
          }

          // Notes
          for note in task.notes.unwrap_or_default() {
            let author_id = match &note.author {
              Some(name) => {
                let author_type = match note.author_type.as_deref() {
                  Some("agent") => AuthorType::Agent,
                  _ => AuthorType::Human,
                };
                let author =
                  repo::author::find_or_create(conn, name, note.author_email.as_deref(), author_type).await?;
                Some(author.id().clone())
              }
              None => None,
            };
            repo::note::create(
              conn,
              EntityType::Task,
              &new_id,
              &crate::store::model::note::New {
                author_id,
                body: note.body,
              },
            )
            .await?;
            counts.notes += 1;
          }

          // Links/relationships
          for link in task.links.unwrap_or_default() {
            import_link(conn, EntityType::Task, &new_id, &link, &id_map).await?;
            counts.relationships += 1;
          }

          id_map.insert(old_id, new_id);
          counts.tasks += 1;
        }
        Err(e) => {
          log::warn!("skipping {}: {e}", path.display());
          counts.skipped += 1;
        }
      }
    }
  }

  Ok(id_map)
}

async fn migrate_artifacts(
  conn: &libsql::Connection,
  project_id: &Id,
  source: &Path,
  counts: &mut MigrationCounts,
) -> Result<(), Error> {
  for (dir, archived) in [
    (source.join("artifacts"), false),
    (source.join("artifacts/archive"), true),
  ] {
    if !dir.is_dir() {
      continue;
    }
    for path in read_files_with_ext(&dir, "md")? {
      let content = std::fs::read_to_string(&path)?;
      match parse_legacy_artifact(&content, archived) {
        Ok(artifact) => {
          let created = repo::artifact::create(
            conn,
            project_id,
            &crate::store::model::artifact::New {
              body: artifact.body,
              metadata: None,
              title: artifact.title,
            },
          )
          .await?;

          let new_id = created.id().clone();

          // Archive if needed
          if artifact.archived_at.is_some() {
            repo::artifact::archive(conn, &new_id).await?;
          }

          // Tags (including kind as a tag)
          let mut tags = artifact.tags;
          if let Some(kind) = artifact.kind
            && !tags.contains(&kind)
          {
            tags.push(kind);
          }
          for tag in &tags {
            repo::tag::attach(conn, EntityType::Artifact, &new_id, tag).await?;
          }

          counts.artifacts += 1;
        }
        Err(e) => {
          log::warn!("skipping {}: {e}", path.display());
          counts.skipped += 1;
        }
      }
    }
  }

  Ok(())
}

async fn migrate_iterations(
  conn: &libsql::Connection,
  project_id: &Id,
  source: &Path,
  counts: &mut MigrationCounts,
  task_id_map: &HashMap<String, Id>,
) -> Result<(), Error> {
  for dir in [source.join("iterations"), source.join("iterations/resolved")] {
    if !dir.is_dir() {
      continue;
    }
    for path in read_files_with_ext(&dir, "toml")? {
      let content = std::fs::read_to_string(&path)?;
      match toml::from_str::<LegacyIteration>(&content) {
        Ok(iteration) => {
          let metadata = toml_table_to_json(&iteration.metadata);

          let created = repo::iteration::create(
            conn,
            project_id,
            &crate::store::model::iteration::New {
              description: iteration.description.unwrap_or_default(),
              metadata,
              title: iteration.title,
            },
          )
          .await?;

          let new_id = created.id().clone();

          // Tags
          for tag in iteration.tags.unwrap_or_default() {
            repo::tag::attach(conn, EntityType::Iteration, &new_id, &tag).await?;
          }

          // Task associations
          for task_ref in iteration.tasks.unwrap_or_default() {
            let task_id_str = task_ref.strip_prefix("tasks/").unwrap_or(&task_ref);
            if let Some(new_task_id) = task_id_map.get(task_id_str) {
              repo::iteration::add_task(conn, &new_id, new_task_id, 1).await?;
            } else {
              log::warn!("iteration task ref {task_ref} not found in migrated tasks");
            }
          }

          // Notes
          for note in iteration.notes.unwrap_or_default() {
            let author_id = match &note.author {
              Some(name) => {
                let author_type = match note.author_type.as_deref() {
                  Some("agent") => AuthorType::Agent,
                  _ => AuthorType::Human,
                };
                let author =
                  repo::author::find_or_create(conn, name, note.author_email.as_deref(), author_type).await?;
                Some(author.id().clone())
              }
              None => None,
            };
            repo::note::create(
              conn,
              EntityType::Iteration,
              &new_id,
              &crate::store::model::note::New {
                author_id,
                body: note.body,
              },
            )
            .await?;
            counts.notes += 1;
          }

          // Links/relationships
          for link in iteration.links.unwrap_or_default() {
            import_link(conn, EntityType::Iteration, &new_id, &link, task_id_map).await?;
            counts.relationships += 1;
          }

          counts.iterations += 1;
        }
        Err(e) => {
          log::warn!("skipping {}: {e}", path.display());
          counts.skipped += 1;
        }
      }
    }
  }

  Ok(())
}

// ── Link import ─────────────────────────────────────────────────────────────────

/// Import a v0.4 link as a relationship. Links reference entities by path like
/// "tasks/<id>" or "artifacts/<id>".
async fn import_link(
  conn: &libsql::Connection,
  source_type: EntityType,
  source_id: &Id,
  link: &LegacyLink,
  task_id_map: &HashMap<String, Id>,
) -> Result<(), Error> {
  let rel_type: RelationshipType = match link.rel.parse() {
    Ok(r) => r,
    Err(_) => {
      log::warn!("unknown relationship type: {}", link.rel);
      return Ok(());
    }
  };

  let (target_type, target_id) = match resolve_link_ref(&link.ref_, task_id_map) {
    Some(pair) => pair,
    None => {
      log::warn!("could not resolve link ref: {}", link.ref_);
      return Ok(());
    }
  };

  repo::relationship::create(conn, rel_type, source_type, source_id, target_type, &target_id)
    .await
    .ok();

  Ok(())
}

/// Resolve a v0.4 link ref (e.g. "tasks/<id>", "artifacts/<id>") to an entity type and new ID.
fn resolve_link_ref(ref_: &str, task_id_map: &HashMap<String, Id>) -> Option<(EntityType, Id)> {
  if let Some(id_str) = ref_.strip_prefix("tasks/") {
    let new_id = task_id_map.get(id_str)?;
    Some((EntityType::Task, new_id.clone()))
  } else if let Some(id_str) = ref_.strip_prefix("artifacts/") {
    // Artifacts don't have an ID map (created with new IDs), but try to parse as-is
    let id: Id = id_str.parse().ok()?;
    Some((EntityType::Artifact, id))
  } else if let Some(id_str) = ref_.strip_prefix("iterations/") {
    let id: Id = id_str.parse().ok()?;
    Some((EntityType::Iteration, id))
  } else {
    None
  }
}

// ── Helpers ─────────────────────────────────────────────────────────────────────

/// Convert a TOML table to a JSON value for metadata storage.
fn toml_table_to_json(table: &toml::Table) -> Option<JsonValue> {
  if table.is_empty() {
    return None;
  }
  // TOML Value -> JSON Value via serde
  let toml_value = toml::Value::Table(table.clone());
  serde_json::to_value(toml_value).ok()
}

fn read_files_with_ext(dir: &Path, ext: &str) -> std::io::Result<Vec<PathBuf>> {
  let mut paths = Vec::new();
  if !dir.exists() {
    return Ok(paths);
  }
  for entry in std::fs::read_dir(dir)? {
    let entry = entry?;
    let path = entry.path();
    if path.is_file() && path.extension().and_then(|e| e.to_str()) == Some(ext) {
      paths.push(path);
    }
  }
  paths.sort();
  Ok(paths)
}

// ── Legacy format parsing ───────────────────────────────────────────────────────

/// Parse a legacy artifact file (YAML frontmatter + markdown body).
fn parse_legacy_artifact(content: &str, archived: bool) -> Result<LegacyArtifact, String> {
  let stripped = content
    .strip_prefix("---\n")
    .ok_or("missing opening frontmatter delimiter")?;
  let end = stripped
    .find("\n---\n")
    .ok_or("missing closing frontmatter delimiter")?;
  let yaml = &stripped[..end];
  let rest = &stripped[end + 5..];
  let body = rest.strip_prefix('\n').unwrap_or(rest);

  let fields = parse_yaml_frontmatter(yaml);

  let title = fields.get("title").ok_or("missing title")?.clone();
  let created_at = fields.get("created_at").ok_or("missing created_at")?.clone();
  let updated_at = fields.get("updated_at").ok_or("missing updated_at")?.clone();
  let kind = fields.get("type").cloned();
  let tags = parse_yaml_list_field(&fields, "tags");

  let archived_at = if archived {
    fields
      .get("archived_at")
      .filter(|s| !s.is_empty())
      .cloned()
      .or_else(|| Some(updated_at.clone()))
  } else {
    None
  };

  Ok(LegacyArtifact {
    archived_at,
    body: body.to_string(),
    _created_at: created_at,
    kind,
    tags,
    title,
    _updated_at: updated_at,
  })
}

fn parse_yaml_frontmatter(yaml: &str) -> HashMap<String, String> {
  let mut fields = HashMap::new();
  let mut current_key: Option<String> = None;
  let mut list_values: Vec<String> = Vec::new();

  for line in yaml.lines() {
    let trimmed = line.trim();
    if trimmed.is_empty() {
      continue;
    }

    if let Some(item) = trimmed.strip_prefix("- ") {
      if let Some(ref key) = current_key {
        list_values.push(item.trim().to_string());
        fields.insert(key.clone(), list_values.join(","));
      }
      continue;
    }

    current_key = None;
    list_values.clear();

    if let Some((key, value)) = trimmed.split_once(':') {
      let key = key.trim().to_string();
      let value = value.trim();

      if value.is_empty() {
        current_key = Some(key);
        list_values.clear();
        continue;
      }

      let value = value
        .strip_prefix('"')
        .and_then(|s| s.strip_suffix('"'))
        .or_else(|| value.strip_prefix('\'').and_then(|s| s.strip_suffix('\'')))
        .unwrap_or(value);

      fields.insert(key, value.to_string());
    }
  }

  fields
}

fn parse_yaml_list_field(fields: &HashMap<String, String>, key: &str) -> Vec<String> {
  fields
    .get(key)
    .map(|s| {
      s.split(',')
        .map(|t| t.trim().to_string())
        .filter(|t| !t.is_empty())
        .collect()
    })
    .unwrap_or_default()
}

// ── Legacy format structs ───────────────────────────────────────────────────────

/// Parsed legacy artifact data.
struct LegacyArtifact {
  archived_at: Option<String>,
  body: String,
  _created_at: String,
  kind: Option<String>,
  tags: Vec<String>,
  title: String,
  _updated_at: String,
}

/// A note embedded in a v0.4 task or iteration TOML file.
#[derive(Debug, Deserialize)]
struct LegacyNote {
  #[serde(default)]
  author: Option<String>,
  #[serde(default)]
  author_email: Option<String>,
  #[serde(default)]
  author_type: Option<String>,
  body: String,
}

/// A link/relationship embedded in a v0.4 task or iteration TOML file.
#[derive(Debug, Deserialize)]
struct LegacyLink {
  #[serde(rename = "ref")]
  ref_: String,
  rel: String,
}

/// Deserialization target for legacy task TOML files.
#[derive(Debug, Deserialize)]
struct LegacyTask {
  #[serde(default)]
  assigned_to: Option<String>,
  description: Option<String>,
  id: String,
  #[serde(default)]
  links: Option<Vec<LegacyLink>>,
  #[serde(default)]
  metadata: toml::Table,
  #[serde(default)]
  notes: Option<Vec<LegacyNote>>,
  #[serde(default)]
  priority: Option<u8>,
  #[serde(default)]
  resolved_at: Option<String>,
  status: String,
  #[serde(default)]
  tags: Option<Vec<String>>,
  title: String,
}

/// Deserialization target for legacy iteration TOML files.
#[derive(Debug, Deserialize)]
struct LegacyIteration {
  description: Option<String>,
  #[serde(default)]
  links: Option<Vec<LegacyLink>>,
  #[serde(default)]
  metadata: toml::Table,
  #[serde(default)]
  notes: Option<Vec<LegacyNote>>,
  #[serde(default)]
  tags: Option<Vec<String>>,
  #[serde(default)]
  tasks: Option<Vec<String>>,
  title: String,
}

#[cfg(test)]
mod tests {
  use super::*;

  mod migrate_integration {
    use std::sync::Arc;

    use pretty_assertions::assert_eq;

    use super::*;

    fn setup_legacy_dir(dir: &Path) {
      for sub in [
        "tasks",
        "tasks/resolved",
        "artifacts",
        "artifacts/archive",
        "iterations",
        "iterations/resolved",
      ] {
        std::fs::create_dir_all(dir.join(sub)).unwrap();
      }
    }

    async fn setup_db() -> (Arc<crate::store::Db>, Id, tempfile::TempDir) {
      let (store, tmp) = crate::store::open_temp().await.unwrap();
      let conn = store.connect().await.unwrap();
      let cwd = std::env::current_dir().unwrap();
      let project = repo::project::create(&conn, &cwd).await.unwrap();
      (store, project.id().clone(), tmp)
    }

    #[tokio::test]
    async fn it_imports_artifacts() {
      let tmp = tempfile::tempdir().unwrap();
      let source = tmp.path().join("legacy");
      setup_legacy_dir(&source);

      let artifact_md = "---\n\
        id: zyxwvutsrqponmlkzyxwvutsrqponmlk\n\
        title: My Artifact\n\
        type: adr\n\
        tags:\n  - design\n\
        created_at: \"2026-01-01T00:00:00Z\"\n\
        updated_at: \"2026-01-02T00:00:00Z\"\n\
        ---\n\n\
        # Hello\n\nThis is the body.\n";
      std::fs::write(
        source.join("artifacts/zyxwvutsrqponmlkzyxwvutsrqponmlk.md"),
        artifact_md,
      )
      .unwrap();

      let (store, project_id, _db_tmp) = setup_db().await;
      let conn = store.connect().await.unwrap();
      let mut counts = MigrationCounts::default();
      migrate_artifacts(&conn, &project_id, &source, &mut counts)
        .await
        .unwrap();

      assert_eq!(counts.artifacts, 1);

      // Verify via repo layer
      let artifacts = repo::artifact::all(
        &conn,
        &project_id,
        &crate::store::model::artifact::Filter {
          all: true,
          ..Default::default()
        },
      )
      .await
      .unwrap();
      assert_eq!(artifacts.len(), 1);
      assert_eq!(artifacts[0].title(), "My Artifact");

      // Tags
      let tags = repo::tag::for_entity(&conn, EntityType::Artifact, artifacts[0].id())
        .await
        .unwrap();
      assert!(tags.contains(&"adr".to_string()));
      assert!(tags.contains(&"design".to_string()));
    }

    #[tokio::test]
    async fn it_imports_tasks_with_notes_and_metadata() {
      let tmp = tempfile::tempdir().unwrap();
      let source = tmp.path().join("legacy");
      setup_legacy_dir(&source);

      let task_toml = r#"
        id = "zyxwvutsrqponmlkzyxwvutsrqponmlk"
        title = "Test task"
        description = "A description"
        status = "open"
        priority = 1
        assigned_to = "agent-1"
        tags = ["bug", "urgent"]
        created_at = "2026-01-01T00:00:00Z"
        updated_at = "2026-01-02T00:00:00Z"

        [metadata]
        custom_key = "value"

        [[notes]]
        author = "alice"
        author_email = "alice@example.com"
        body = "A note on this task"
      "#;
      std::fs::write(source.join("tasks/zyxwvutsrqponmlkzyxwvutsrqponmlk.toml"), task_toml).unwrap();

      let (store, project_id, _db_tmp) = setup_db().await;
      let conn = store.connect().await.unwrap();
      let mut counts = MigrationCounts::default();
      let id_map = migrate_tasks(&conn, &project_id, &source, &mut counts).await.unwrap();

      assert_eq!(counts.tasks, 1);
      assert_eq!(counts.notes, 1);
      assert_eq!(id_map.len(), 1);

      let new_id = id_map.get("zyxwvutsrqponmlkzyxwvutsrqponmlk").unwrap();
      let task = repo::task::find_by_id(&conn, new_id.clone()).await.unwrap().unwrap();
      assert_eq!(task.title(), "Test task");
      assert_eq!(task.description(), "A description");
      assert_eq!(task.priority(), Some(1));

      // Verify metadata
      assert_eq!(task.metadata()["custom_key"], "value");

      // Verify notes
      let notes = repo::note::for_entity(&conn, EntityType::Task, new_id).await.unwrap();
      assert_eq!(notes.len(), 1);
      assert_eq!(notes[0].body(), "A note on this task");
    }

    #[tokio::test]
    async fn it_imports_iterations_with_task_associations() {
      let tmp = tempfile::tempdir().unwrap();
      let source = tmp.path().join("legacy");
      setup_legacy_dir(&source);

      // Create a task first
      let task_toml = r#"
        id = "kkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkk"
        title = "Linked task"
        status = "open"
        created_at = "2026-01-01T00:00:00Z"
        updated_at = "2026-01-01T00:00:00Z"
      "#;
      std::fs::write(source.join("tasks/kkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkk.toml"), task_toml).unwrap();

      let iter_toml = r#"
        id = "zyxwvutsrqponmlkzyxwvutsrqponmlk"
        title = "Sprint 1"
        description = "First sprint"
        status = "active"
        tags = ["sprint"]
        tasks = ["tasks/kkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkk"]
        created_at = "2026-01-01T00:00:00Z"
        updated_at = "2026-01-02T00:00:00Z"
      "#;
      std::fs::write(
        source.join("iterations/zyxwvutsrqponmlkzyxwvutsrqponmlk.toml"),
        iter_toml,
      )
      .unwrap();

      let (store, project_id, _db_tmp) = setup_db().await;
      let conn = store.connect().await.unwrap();
      let mut counts = MigrationCounts::default();

      let task_id_map = migrate_tasks(&conn, &project_id, &source, &mut counts).await.unwrap();
      migrate_iterations(&conn, &project_id, &source, &mut counts, &task_id_map)
        .await
        .unwrap();

      assert_eq!(counts.tasks, 1);
      assert_eq!(counts.iterations, 1);

      // Verify iteration was created
      let iterations = repo::iteration::all(
        &conn,
        &project_id,
        &crate::store::model::iteration::Filter {
          all: true,
          ..Default::default()
        },
      )
      .await
      .unwrap();
      assert_eq!(iterations.len(), 1);
      assert_eq!(iterations[0].title(), "Sprint 1");
      assert_eq!(iterations[0].description(), "First sprint");

      // Verify task association
      let tasks = repo::iteration::tasks_with_phase(&conn, iterations[0].id())
        .await
        .unwrap();
      assert_eq!(tasks.len(), 1);
    }
  }

  mod parse_legacy_artifact {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_falls_back_to_updated_at_when_archived_at_missing() {
      let content = "---\n\
        id: zyxwvutsrqponmlkzyxwvutsrqponmlk\n\
        title: Archived\n\
        created_at: \"2026-01-01T00:00:00Z\"\n\
        updated_at: \"2026-01-02T00:00:00Z\"\n\
        ---\n\n\
        Old.\n";
      let artifact = parse_legacy_artifact(content, true).unwrap();

      assert_eq!(artifact.archived_at, Some("2026-01-02T00:00:00Z".to_string()));
    }

    #[test]
    fn it_parses_artifact_with_body() {
      let content = "---\n\
        id: zyxwvutsrqponmlkzyxwvutsrqponmlk\n\
        title: Test\n\
        created_at: \"2026-01-01T00:00:00Z\"\n\
        updated_at: \"2026-01-01T00:00:00Z\"\n\
        ---\n\n\
        Body content here.\n";
      let artifact = parse_legacy_artifact(content, false).unwrap();

      assert_eq!(artifact.title, "Test");
      assert_eq!(artifact.body, "Body content here.\n");
      assert_eq!(artifact.archived_at, None);
    }

    #[test]
    fn it_parses_artifact_with_kind() {
      let content = "---\n\
        id: zyxwvutsrqponmlkzyxwvutsrqponmlk\n\
        title: ADR\n\
        type: adr\n\
        created_at: \"2026-01-01T00:00:00Z\"\n\
        updated_at: \"2026-01-01T00:00:00Z\"\n\
        ---\n\n\
        Decision.\n";
      let artifact = parse_legacy_artifact(content, false).unwrap();

      assert_eq!(artifact.kind.as_deref(), Some("adr"));
    }

    #[test]
    fn it_parses_artifact_with_tags() {
      let content = "---\n\
        id: zyxwvutsrqponmlkzyxwvutsrqponmlk\n\
        title: Tagged\n\
        tags:\n  - design\n  - adr\n\
        created_at: \"2026-01-01T00:00:00Z\"\n\
        updated_at: \"2026-01-01T00:00:00Z\"\n\
        ---\n\n\
        Content.\n";
      let artifact = parse_legacy_artifact(content, false).unwrap();

      assert_eq!(artifact.tags, vec!["design", "adr"]);
    }

    #[test]
    fn it_sets_archived_at_for_archived_artifacts() {
      let content = "---\n\
        id: zyxwvutsrqponmlkzyxwvutsrqponmlk\n\
        title: Archived\n\
        created_at: \"2026-01-01T00:00:00Z\"\n\
        updated_at: \"2026-01-02T00:00:00Z\"\n\
        archived_at: \"2026-01-03T00:00:00Z\"\n\
        ---\n\n\
        Old.\n";
      let artifact = parse_legacy_artifact(content, true).unwrap();

      assert_eq!(artifact.archived_at, Some("2026-01-03T00:00:00Z".to_string()));
    }
  }

  mod parse_legacy_task {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_parses_minimal_task() {
      let toml_str = r#"
        id = "zyxwvutsrqponmlkzyxwvutsrqponmlk"
        title = "Minimal"
        status = "open"
        created_at = "2026-01-01T00:00:00Z"
        updated_at = "2026-01-01T00:00:00Z"
      "#;
      let task: LegacyTask = toml::from_str(toml_str).unwrap();

      assert_eq!(task.title, "Minimal");
      assert_eq!(task.description, None);
      assert_eq!(task.assigned_to, None);
      assert_eq!(task.priority, None);
      assert!(task.tags.unwrap_or_default().is_empty());
    }

    #[test]
    fn it_parses_task_with_all_fields() {
      let toml_str = r#"
        id = "zyxwvutsrqponmlkzyxwvutsrqponmlk"
        title = "Full task"
        description = "A description"
        status = "done"
        priority = 1
        assigned_to = "agent-1"
        tags = ["bug", "urgent"]
        created_at = "2026-01-01T00:00:00Z"
        updated_at = "2026-01-02T00:00:00Z"
        resolved_at = "2026-01-02T00:00:00Z"

        [[notes]]
        author = "alice"
        body = "A note"

        [[links]]
        ref = "tasks/kkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkk"
        rel = "blocked-by"
      "#;
      let task: LegacyTask = toml::from_str(toml_str).unwrap();

      assert_eq!(task.title, "Full task");
      assert_eq!(task.description.as_deref(), Some("A description"));
      assert_eq!(task.assigned_to.as_deref(), Some("agent-1"));
      assert_eq!(task.priority, Some(1));
      assert_eq!(task.tags.unwrap(), vec!["bug", "urgent"]);
      assert_eq!(task.resolved_at, Some("2026-01-02T00:00:00Z".to_string()));
      assert_eq!(task.notes.as_ref().unwrap().len(), 1);
      assert_eq!(task.links.as_ref().unwrap().len(), 1);
    }
  }

  mod parse_legacy_iteration {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_parses_iteration_with_tasks() {
      let toml_str = r#"
        id = "zyxwvutsrqponmlkzyxwvutsrqponmlk"
        title = "Sprint 1"
        description = "First sprint"
        status = "active"
        tags = ["sprint"]
        tasks = ["tasks/kkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkk"]
        created_at = "2026-01-01T00:00:00Z"
        updated_at = "2026-01-02T00:00:00Z"
      "#;
      let iteration: LegacyIteration = toml::from_str(toml_str).unwrap();

      assert_eq!(iteration.tasks.unwrap(), vec!["tasks/kkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkk"]);
      assert_eq!(iteration.tags.unwrap(), vec!["sprint"]);
    }

    #[test]
    fn it_parses_minimal_iteration() {
      let toml_str = r#"
        id = "zyxwvutsrqponmlkzyxwvutsrqponmlk"
        title = "Sprint"
        status = "active"
        created_at = "2026-01-01T00:00:00Z"
        updated_at = "2026-01-01T00:00:00Z"
      "#;
      let iteration: LegacyIteration = toml::from_str(toml_str).unwrap();

      assert_eq!(iteration.title, "Sprint");
      assert_eq!(iteration.description, None);
      assert!(iteration.tasks.unwrap_or_default().is_empty());
      assert!(iteration.tags.unwrap_or_default().is_empty());
    }
  }

  mod parse_yaml_frontmatter {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_parses_list_values() {
      let yaml = "tags:\n  - bug\n  - urgent";
      let fields = parse_yaml_frontmatter(yaml);
      let tags = parse_yaml_list_field(&fields, "tags");

      assert_eq!(tags, vec!["bug", "urgent"]);
    }

    #[test]
    fn it_parses_simple_key_value_pairs() {
      let yaml = "id: zyxwvutsrqponmlkzyxwvutsrqponmlk\ntitle: Test";
      let fields = parse_yaml_frontmatter(yaml);

      assert_eq!(fields.get("id").unwrap(), "zyxwvutsrqponmlkzyxwvutsrqponmlk");
      assert_eq!(fields.get("title").unwrap(), "Test");
    }

    #[test]
    fn it_strips_quotes() {
      let yaml = "created_at: \"2026-01-01T00:00:00Z\"";
      let fields = parse_yaml_frontmatter(yaml);

      assert_eq!(fields.get("created_at").unwrap(), "2026-01-01T00:00:00Z");
    }
  }

  mod toml_table_to_json_fn {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_returns_none_for_empty_table() {
      let table = toml::Table::new();
      assert_eq!(toml_table_to_json(&table), None);
    }

    #[test]
    fn it_converts_toml_table_to_json() {
      let mut table = toml::Table::new();
      table.insert("key".to_string(), toml::Value::String("value".to_string()));
      let json = toml_table_to_json(&table).unwrap();
      assert_eq!(json["key"], "value");
    }
  }
}
