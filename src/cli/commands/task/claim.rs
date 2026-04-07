use clap::Args;

use crate::{
  AppContext,
  cli::Error,
  io::git,
  store::{
    model::{
      primitives::{AuthorType, TaskStatus},
      task::Patch,
    },
    repo,
  },
  ui::{components::SuccessMessage, json},
};

/// Claim a task (assign to self and mark in-progress).
#[derive(Args, Debug)]
pub struct Command {
  /// The task ID or prefix.
  id: String,
  /// Claim as a specific author name (defaults to git user).
  #[arg(long = "as")]
  as_author: Option<String>,
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

    let (author_name, author_email) = match &self.as_author {
      Some(name) => (name.clone(), None),
      None => {
        let ga = git::resolve_author_or_env();
        (
          ga.as_ref()
            .map(|a| a.name.clone())
            .unwrap_or_else(|| "unknown".to_string()),
          ga.and_then(|a| a.email),
        )
      }
    };

    let author = repo::author::find_or_create(&conn, &author_name, author_email.as_deref(), AuthorType::Human).await?;

    let patch = Patch {
      assigned_to: Some(Some(author.id().clone())),
      status: Some(TaskStatus::InProgress),
      ..Default::default()
    };

    let tx = repo::transaction::begin(&conn, project_id, "task claim").await?;
    let task = repo::task::update(&conn, &id, &patch).await?;
    repo::transaction::record_semantic_event(
      &conn,
      tx.id(),
      "tasks",
      &id.to_string(),
      "modified",
      Some(&before),
      Some("status-change"),
      Some(&before_task.status().to_string()),
      Some(&task.status().to_string()),
    )
    .await?;

    // Claim moves to in-progress, which is active.
    let prefix_len = repo::task::shortest_active_prefix(&conn, project_id).await?;

    let short_id = task.id().short();
    self.output.print_entity(&task, &short_id, || {
      SuccessMessage::new("claimed task")
        .id(task.id().short())
        .prefix_len(prefix_len)
        .field("title", task.title().to_string())
        .field("assigned to", author_name.clone())
        .to_string()
    })?;
    Ok(())
  }
}
