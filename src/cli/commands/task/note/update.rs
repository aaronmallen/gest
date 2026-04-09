use clap::Args;

use crate::{
  AppContext,
  cli::Error,
  store::{model::note::Patch, repo},
  ui::{components::SuccessMessage, json},
};

/// Update a note's body.
#[derive(Args, Debug)]
pub struct Command {
  /// The note ID or prefix.
  id: String,
  /// The new body text (use `-` to open `$EDITOR`).
  #[arg(long, short)]
  body: Option<String>,
  #[command(flatten)]
  output: json::Flags,
}

impl Command {
  /// Replace the resolved note's body within a recorded transaction.
  pub async fn call(&self, context: &AppContext) -> Result<(), Error> {
    log::debug!("task note update: entry");
    let project_id = context.project_id().as_ref().ok_or(Error::UninitializedProject)?;
    let conn = context.store().connect().await?;
    let note_id = repo::resolve::resolve_id(&conn, "notes", &self.id).await?;

    let existing = repo::note::find_required_by_id(&conn, note_id.clone()).await?;

    let body = match &self.body {
      Some(b) if b == "-" => {
        crate::io::editor::edit_text_with_suffix(existing.body(), ".md").map_err(|e| Error::Editor(e.to_string()))?
      }
      Some(b) => b.clone(),
      None => {
        crate::io::editor::edit_text_with_suffix(existing.body(), ".md").map_err(|e| Error::Editor(e.to_string()))?
      }
    };

    if body.trim().is_empty() {
      return Err(Error::Editor("Aborting: empty note body".into()));
    }

    let before = serde_json::to_value(&existing)?;
    let tx = repo::transaction::begin(&conn, project_id, "task note update").await?;
    let patch = Patch {
      body: Some(body),
    };
    let note = repo::note::update(&conn, &note_id, &patch).await?;
    repo::transaction::record_event(&conn, tx.id(), "notes", &note_id.to_string(), "modified", Some(&before)).await?;

    let short_id = note.id().short();
    self.output.print_entity(&note, &short_id, || {
      SuccessMessage::new("updated note").id(note.id().short()).to_string()
    })?;
    Ok(())
  }
}
