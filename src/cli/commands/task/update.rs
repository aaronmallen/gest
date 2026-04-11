use clap::Args;

use crate::{
  AppContext,
  cli::{Error, commands::task::priority::parse_priority, meta_args, tag_arg},
  store::{
    model::{
      primitives::{AuthorType, EntityType, TaskStatus},
      task::Patch,
    },
    repo,
  },
  ui::{components::SuccessMessage, envelope::Envelope, json},
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
  /// Set the task priority; either 0-4 (lower is higher) or one of critical|high|medium|low|lowest.
  #[arg(long, short, value_parser = parse_priority)]
  priority: Option<u8>,
  /// Set the task status.
  #[arg(long, short)]
  status: Option<TaskStatus>,
  /// Replace all tags on the task. Repeatable; comma-separated values split into multiple tags.
  #[arg(long, value_delimiter = ',', value_parser = tag_arg::trim_tag)]
  tag: Vec<String>,
  /// Set the task title.
  #[arg(long, short)]
  title: Option<String>,
  #[command(flatten)]
  output: json::Flags,
}

impl Command {
  /// Apply field, status, priority, tag, and phase changes to the resolved task within a recorded transaction.
  pub async fn call(&self, context: &AppContext) -> Result<(), Error> {
    log::debug!("task update: entry");
    let project_id = context.project_id().as_ref().ok_or(Error::UninitializedProject)?;
    let conn = context.store().connect().await?;

    let id = repo::resolve::resolve_id(&conn, repo::resolve::Table::Tasks, &self.id).await?;
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

    // Emit semantic events for status/priority changes; fall back to a plain
    // "modified" event (no semantic_type) when only free-form fields changed.
    let status_changed = patch.status.is_some() && before_task.status() != task.status();
    let priority_changed = patch.priority.is_some() && before_task.priority() != task.priority();

    if status_changed {
      let semantic = match task.status() {
        TaskStatus::Done => "completed",
        TaskStatus::Cancelled => "cancelled",
        _ => "status-change",
      };
      repo::transaction::record_semantic_event(
        &conn,
        tx.id(),
        "tasks",
        &id.to_string(),
        "modified",
        Some(&before),
        Some(semantic),
        Some(&before_task.status().to_string()),
        Some(&task.status().to_string()),
      )
      .await?;
    }

    if priority_changed {
      let old = before_task.priority().map(|p| p.to_string());
      let new = task.priority().map(|p| p.to_string());
      repo::transaction::record_semantic_event(
        &conn,
        tx.id(),
        "tasks",
        &id.to_string(),
        "modified",
        Some(&before),
        Some("priority-change"),
        old.as_deref(),
        new.as_deref(),
      )
      .await?;
    }

    if !status_changed && !priority_changed {
      repo::transaction::record_event(&conn, tx.id(), "tasks", &id.to_string(), "modified", Some(&before)).await?;
    }

    // Replace all tags if --tag was specified
    if !self.tag.is_empty() {
      repo::tag::detach_all(&conn, EntityType::Task, &id).await?;
      for label in tag_arg::normalize_tags(&self.tag) {
        let tag = repo::tag::attach(&conn, EntityType::Task, &id, &label).await?;
        repo::transaction::record_event(&conn, tx.id(), "entity_tags", &tag.id().to_string(), "created", None).await?;
      }
    }

    // Update phase if specified
    if let Some(phase) = self.phase {
      let old_phase = repo::iteration::task_phase(&conn, &id).await?;
      repo::iteration::update_task_phase(&conn, &id, phase).await?;
      // Only record a timeline-facing phase-change when the task is actually
      // tracked in an iteration and the phase moved.
      if let Some(old) = old_phase
        && old != phase
      {
        let old_str = old.to_string();
        let new_str = phase.to_string();
        repo::transaction::record_semantic_event(
          &conn,
          tx.id(),
          "iteration_tasks",
          &id.to_string(),
          "modified",
          None,
          Some("phase-change"),
          Some(&old_str),
          Some(&new_str),
        )
        .await?;
      }
    }

    let envelope = Envelope::load_one(&conn, EntityType::Task, task.id(), &task, true).await?;

    let prefix_map = repo::task::per_id_prefix_lengths(&conn, project_id).await?;
    let prefix_len = prefix_map.get(&task.id().to_string()).copied().unwrap_or(1);

    let short_id = task.id().short();
    self.output.print_envelope(&envelope, &short_id, || {
      log::info!("updated task");
      SuccessMessage::new("updated task")
        .id(task.id().short())
        .prefix_len(prefix_len)
        .field("title", task.title().to_string())
        .to_string()
    })?;
    Ok(())
  }
}
