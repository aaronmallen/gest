use clap::Args;

use crate::{
  AppContext,
  cli::Error,
  io::git,
  store::{
    model::{
      note::New,
      primitives::{AuthorType, EntityType},
    },
    repo,
  },
  ui::{components::SuccessMessage, json},
};

/// Add a note to a task.
#[derive(Args, Debug)]
pub struct Command {
  /// The task ID or prefix.
  id: String,
  /// The note body (use `-` to open `$EDITOR`).
  #[arg(short, long)]
  body: String,
  /// Set the author (agent) identifier for this note.
  #[arg(long)]
  agent: Option<String>,
  #[command(flatten)]
  output: json::Flags,
}

impl Command {
  pub async fn call(&self, context: &AppContext) -> Result<(), Error> {
    log::debug!("task note add: entry");
    let project_id = context.project_id().as_ref().ok_or(Error::UninitializedProject)?;
    let conn = context.store().connect().await?;
    let task_id = repo::resolve::resolve_id(&conn, "tasks", &self.id).await?;

    let body = if self.body == "-" {
      crate::io::editor::edit_text_with_suffix("", ".md").map_err(|e| Error::Editor(e.to_string()))?
    } else {
      self.body.clone()
    };

    if body.trim().is_empty() {
      return Err(Error::Editor("Aborting: empty note body".into()));
    }

    let author_id = match &self.agent {
      Some(name) => {
        let author = repo::author::find_or_create(&conn, name, None, AuthorType::Agent).await?;
        Some(author.id().clone())
      }
      None => {
        if let Some(ga) = git::resolve_author_or_env() {
          let author = repo::author::find_or_create(&conn, &ga.name, ga.email.as_deref(), AuthorType::Human).await?;
          Some(author.id().clone())
        } else {
          None
        }
      }
    };

    let new = New {
      author_id,
      body,
    };
    let tx = repo::transaction::begin(&conn, project_id, "task note add").await?;
    let note = repo::note::create(&conn, EntityType::Task, &task_id, &new).await?;
    repo::transaction::record_event(&conn, tx.id(), "notes", &note.id().to_string(), "created", None).await?;

    let short_id = note.id().short();
    self.output.print_entity(&note, &short_id, || {
      log::info!("added note");
      SuccessMessage::new("added note")
        .id(note.id().short())
        .field("task", task_id.short())
        .to_string()
    })?;
    Ok(())
  }
}
