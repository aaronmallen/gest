use std::io::IsTerminal;

use clap::Args;
use libsql::Connection;

use crate::{
  AppContext,
  cli::{Error, meta_args},
  store::{
    model::{
      primitives::{AuthorType, EntityType, Id, RelationshipType, TaskStatus},
      task::New,
    },
    repo,
  },
  ui::{components::SuccessMessage, json},
};

/// Create a new task.
///
/// When stdin is piped, the first markdown heading (`# …`) is used as the title
/// and the full input becomes the description. A title can also be given as a
/// positional argument, in which case stdin (if piped) is used as the description only.
#[derive(Args, Debug)]
pub struct Command {
  /// The task title (extracted from the first `# heading` when piping stdin).
  title: Option<String>,
  /// Assign the task to an author by name.
  #[arg(long)]
  assign: Option<String>,
  /// Read NDJSON task objects from stdin (one per line).
  #[arg(long)]
  batch: bool,
  /// The task description (opens `$EDITOR` if omitted and stdin is a terminal).
  #[arg(long, short)]
  description: Option<String>,
  /// Add the task to an iteration (ID or prefix).
  #[arg(long, short)]
  iteration: Option<String>,
  /// Link the task to another entity (format: `rel:target`). Repeatable.
  #[arg(long, short)]
  link: Vec<String>,
  /// Set a metadata key=value pair (repeatable; supports dot-paths and scalar inference).
  #[arg(long = "metadata", short = 'm', value_name = "KEY=VALUE")]
  metadata: Vec<String>,
  /// Merge a JSON object into metadata (repeatable; applied after --metadata pairs).
  #[arg(long = "metadata-json", value_name = "JSON")]
  metadata_json: Vec<String>,
  /// The phase within the iteration (defaults to 1 or max existing + 1).
  #[arg(long)]
  phase: Option<u32>,
  /// The task priority (0-4, lower is higher).
  #[arg(long, short)]
  priority: Option<u8>,
  /// The initial task status.
  #[arg(long, short)]
  status: Option<TaskStatus>,
  /// Add a tag to the task. Repeatable.
  #[arg(long)]
  tag: Vec<String>,
  #[command(flatten)]
  output: json::Flags,
}

impl Command {
  pub async fn call(&self, context: &AppContext) -> Result<(), Error> {
    let project_id = context.project_id().as_ref().ok_or(Error::UninitializedProject)?;
    let conn = context.store().connect().await?;

    if self.batch {
      return self.batch_create(context, project_id, &conn).await;
    }

    let assigned_to = if let Some(name) = &self.assign {
      let author = repo::author::find_or_create(&conn, name, None, AuthorType::Human).await?;
      Some(author.id().clone())
    } else {
      None
    };

    let (title, description) = self.resolve_title_and_description()?;
    let metadata = meta_args::build_metadata(None, &self.metadata, &self.metadata_json)?;

    let new = New {
      assigned_to,
      description,
      metadata,
      priority: self.priority,
      status: self.status,
      title,
    };

    let tx = repo::transaction::begin(&conn, project_id, "task create").await?;
    let task = repo::task::create(&conn, project_id, &new).await?;
    repo::transaction::record_event(&conn, tx.id(), "tasks", &task.id().to_string(), "created", None).await?;

    // Apply tags
    for label in &self.tag {
      let tag = repo::tag::attach(&conn, EntityType::Task, task.id(), label).await?;
      repo::transaction::record_event(&conn, tx.id(), "entity_tags", &tag.id().to_string(), "created", None).await?;
    }

    // Apply links
    for link_spec in &self.link {
      let (rel_type, target_id) = parse_link_spec(link_spec)?;
      let target_table = resolve_entity_table(&conn, &target_id).await;
      let target = repo::resolve::resolve_id(&conn, &target_table, &target_id).await?;
      let target_type = table_to_entity_type(&target_table);
      let rel = repo::relationship::create(&conn, rel_type, EntityType::Task, task.id(), target_type, &target).await?;
      repo::transaction::record_event(&conn, tx.id(), "relationships", &rel.id().to_string(), "created", None).await?;
    }

    // Add to iteration
    if let Some(iter_ref) = &self.iteration {
      let iter_id = repo::resolve::resolve_id(&conn, "iterations", iter_ref).await?;
      let phase = match self.phase {
        Some(p) => p,
        None => {
          let max = repo::iteration::max_phase(&conn, &iter_id).await?;
          max.map(|m| m + 1).unwrap_or(1)
        }
      };
      repo::iteration::add_task(&conn, &iter_id, task.id(), phase).await?;
    }

    let short_id = task.id().short();
    self.output.print_entity(&task, &short_id, || {
      let mut message = SuccessMessage::new("created task").id(task.id().short());
      message = message.field("title", task.title().to_string());
      message.to_string()
    })?;
    Ok(())
  }

  async fn batch_create(&self, _context: &AppContext, project_id: &Id, conn: &Connection) -> Result<(), Error> {
    let input = std::io::read_to_string(std::io::stdin()).unwrap_or_default();
    let mut count = 0u64;

    for line in input.lines() {
      let line = line.trim();
      if line.is_empty() {
        continue;
      }

      let new: New = serde_json::from_str(line).map_err(|e| Error::Editor(format!("invalid NDJSON: {e}")))?;
      let tx = repo::transaction::begin(conn, project_id, "task create").await?;
      let task = repo::task::create(conn, project_id, &new).await?;
      repo::transaction::record_event(conn, tx.id(), "tasks", &task.id().to_string(), "created", None).await?;
      count += 1;
    }

    let message = SuccessMessage::new("batch created").field("count", count.to_string());
    println!("{message}");
    Ok(())
  }

  fn resolve_title_and_description(&self) -> Result<(String, String), Error> {
    if let Some(title) = &self.title {
      // Title given explicitly — description from --description, stdin, or editor
      let description = if let Some(desc) = &self.description {
        desc.clone()
      } else if std::io::stdin().is_terminal() {
        crate::io::editor::edit_text_with_suffix("", ".md").map_err(|e| Error::Editor(e.to_string()))?
      } else {
        std::io::read_to_string(std::io::stdin()).unwrap_or_default()
      };
      Ok((title.clone(), description))
    } else if !std::io::stdin().is_terminal() {
      // No title arg, stdin is piped — parse title from first heading
      let input = std::io::read_to_string(std::io::stdin()).unwrap_or_default();
      let title = extract_heading(&input)
        .ok_or_else(|| Error::Editor("no title provided and no # heading found in stdin".into()))?;
      Ok((title, input))
    } else {
      Err(Error::Editor("task title is required".into()))
    }
  }
}

/// Extract the text of the first markdown `# heading` from the input.
fn extract_heading(input: &str) -> Option<String> {
  for line in input.lines() {
    let trimmed = line.trim();
    if let Some(heading) = trimmed.strip_prefix("# ") {
      let heading = heading.trim();
      if !heading.is_empty() {
        return Some(heading.to_string());
      }
    }
  }
  None
}

/// Parse a link spec in the format `rel:target` or just `target` (defaults to relates-to).
fn parse_link_spec(spec: &str) -> Result<(RelationshipType, String), Error> {
  if let Some((rel_str, target)) = spec.split_once(':') {
    let rel_type: RelationshipType = rel_str.parse().map_err(|e: String| Error::Editor(e))?;
    Ok((rel_type, target.to_string()))
  } else {
    Ok((RelationshipType::RelatesTo, spec.to_string()))
  }
}

/// Try to resolve the table for a target entity ID by checking multiple tables.
async fn resolve_entity_table(conn: &Connection, id: &str) -> String {
  // Try tasks first, then artifacts, then iterations
  for table in &["tasks", "artifacts", "iterations"] {
    if repo::resolve::resolve_id(conn, table, id).await.is_ok() {
      return table.to_string();
    }
  }
  "tasks".to_string()
}

/// Map a table name to its EntityType.
fn table_to_entity_type(table: &str) -> EntityType {
  match table {
    "artifacts" => EntityType::Artifact,
    "iterations" => EntityType::Iteration,
    _ => EntityType::Task,
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  mod extract_heading_fn {
    use super::*;

    #[test]
    fn it_extracts_heading_from_markdown() {
      let input = "# Fix the bug\n\ndetails here";
      assert_eq!(extract_heading(input), Some("Fix the bug".into()));
    }

    #[test]
    fn it_returns_none_when_no_heading() {
      let input = "just text";
      assert_eq!(extract_heading(input), None);
    }
  }

  mod parse_link_spec_fn {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_parses_rel_and_target() {
      let (rel, target) = parse_link_spec("blocks:abc123").unwrap();

      assert_eq!(rel, RelationshipType::Blocks);
      assert_eq!(target, "abc123");
    }

    #[test]
    fn it_defaults_to_relates_to() {
      let (rel, target) = parse_link_spec("abc123").unwrap();

      assert_eq!(rel, RelationshipType::RelatesTo);
      assert_eq!(target, "abc123");
    }

    #[test]
    fn it_rejects_invalid_rel_type() {
      let result = parse_link_spec("invalid:abc123");

      assert!(result.is_err());
    }
  }
}
