use clap::Args;

use crate::{
  AppContext,
  cli::Error,
  store::repo,
  ui::{components::FieldList, json},
};

/// Show a single note.
#[derive(Args, Debug)]
pub struct Command {
  /// The note ID or prefix.
  id: String,
  #[command(flatten)]
  output: json::Flags,
}

impl Command {
  pub async fn call(&self, context: &AppContext) -> Result<(), Error> {
    let conn = context.store().connect().await?;
    let note_id = repo::resolve::resolve_id(&conn, "notes", &self.id).await?;

    let note = repo::note::find_by_id(&conn, note_id)
      .await?
      .ok_or_else(|| repo::note::Error::NotFound(self.id.clone()))?;

    let short_id = note.id().short();
    self.output.print_entity(&note, &short_id, || {
      let mut fields = FieldList::new()
        .field("id", note.id().short())
        .field("body", note.body().to_string())
        .field("created", note.created_at().to_rfc3339());

      if let Some(author) = note.author_id() {
        fields = fields.field("author", author.short());
      }

      fields = fields.field("updated", note.updated_at().to_rfc3339());

      fields.to_string()
    })?;
    Ok(())
  }
}
