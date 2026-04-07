use clap::Args;

use crate::{
  AppContext,
  cli::{Error, meta_args},
  store::{
    model::{
      primitives::{AuthorType, EntityType, TaskStatus},
      task::Patch,
    },
    repo,
  },
  ui::{components::SuccessMessage, json},
};

/// Update a task.
#[derive(Args, Debug)]
pub struct Command {
  /// The task ID or prefix.
  id: String,
  /// Set the assigned author by name.
  #[arg(long)]
  assigned_to: Option<String>,
  /// Set the task description.
  #[arg(long, short)]
  description: Option<String>,
  /// Set a metadata key=value pair (repeatable; supports dot-paths and scalar inference).
  #[arg(long = "metadata", short = 'm', value_name = "KEY=VALUE")]
  metadata: Vec<String>,
  /// Merge a JSON object into metadata (repeatable; applied after --metadata pairs).
  #[arg(long = "metadata-json", value_name = "JSON")]
  metadata_json: Vec<String>,
  /// Move the task to a phase within its iteration.
  #[arg(long)]
  phase: Option<u32>,
  /// Set the task priority (0-4).
  #[arg(long, short)]
  priority: Option<u8>,
  /// Set the task status.
  #[arg(long, short)]
  status: Option<TaskStatus>,
  /// Replace all tags on the task. Repeatable.
  #[arg(long)]
  tag: Vec<String>,
  /// Set the task title.
  #[arg(long, short)]
  title: Option<String>,
  #[command(flatten)]
  output: json::Flags,
}

impl Command {
  pub async fn call(&self, context: &AppContext) -> Result<(), Error> {
    let project_id = context.project_id().as_ref().ok_or(Error::UninitializedProject)?;
    let conn = context.store().connect().await?;

    let id = repo::resolve::resolve_id(&conn, "tasks", &self.id).await?;
    let before_task = repo::task::find_by_id(&conn, id.clone())
      .await?
      .ok_or(Error::UninitializedProject)?;
    let before = serde_json::to_value(&before_task)?;
    let tx = repo::transaction::begin(&conn, project_id, "task update").await?;

    let metadata = if self.metadata.is_empty() && self.metadata_json.is_empty() {
      None
    } else {
      meta_args::build_metadata(
        Some(before_task.metadata().clone()),
        &self.metadata,
        &self.metadata_json,
      )?
    };

    // Resolve assigned_to
    let assigned_to = if let Some(name) = &self.assigned_to {
      let author = repo::author::find_or_create(&conn, name, None, AuthorType::Human).await?;
      Some(Some(author.id().clone()))
    } else {
      None
    };

    let patch = Patch {
      assigned_to,
      description: self.description.clone(),
      metadata,
      priority: self.priority.map(Some),
      status: self.status,
      title: self.title.clone(),
    };

    let task = repo::task::update(&conn, &id, &patch).await?;
    repo::transaction::record_event(&conn, tx.id(), "tasks", &id.to_string(), "modified", Some(&before)).await?;

    // Replace all tags if --tag was specified
    if !self.tag.is_empty() {
      repo::tag::detach_all(&conn, EntityType::Task, &id).await?;
      for label in &self.tag {
        let tag = repo::tag::attach(&conn, EntityType::Task, &id, label).await?;
        repo::transaction::record_event(&conn, tx.id(), "entity_tags", &tag.id().to_string(), "created", None).await?;
      }
    }

    // Update phase if specified
    if let Some(phase) = self.phase {
      repo::iteration::update_task_phase(&conn, &id, phase).await?;
    }

    let prefix_len = if task.status().is_terminal() {
      repo::task::shortest_all_prefix(&conn, project_id).await?
    } else {
      repo::task::shortest_active_prefix(&conn, project_id).await?
    };

    let short_id = task.id().short();
    self.output.print_entity(&task, &short_id, || {
      SuccessMessage::new("updated task")
        .id(task.id().short())
        .prefix_len(prefix_len)
        .field("title", task.title().to_string())
        .to_string()
    })?;
    Ok(())
  }
}
