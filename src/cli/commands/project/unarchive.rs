use clap::Args;

use crate::{
  AppContext,
  cli::Error,
  store::repo::{self, resolve::Table},
  ui::{components::SuccessMessage, envelope::Envelope, json},
};

/// Restore an archived project to active status.
#[derive(Args, Debug)]
pub struct Command {
  /// The project ID or prefix.
  id: String,
  #[command(flatten)]
  output: json::Flags,
}

impl Command {
  /// Clear the archived flag on the project and print a workspace reattach hint.
  pub async fn call(&self, context: &AppContext) -> Result<(), Error> {
    log::debug!("project unarchive: entry");
    let conn = context.store().connect().await?;

    let id = repo::resolve::resolve_id(&conn, Table::Projects, &self.id).await?;
    let project = repo::project::find_by_id(&conn, id)
      .await?
      .ok_or_else(|| Error::NotFound(format!("project {} not found", self.id)))?;

    repo::project::unarchive(&conn, project.id()).await?;
    let restored = repo::project::find_by_id(&conn, project.id().clone())
      .await?
      .ok_or_else(|| Error::NotFound(format!("project {} not found after unarchive", project.id().short())))?;

    let envelope = Envelope {
      entity: &restored,
      notes: None,
      relationships: vec![],
      tags: vec![],
    };
    let short_id = restored.id().short();
    self.output.print_envelope(&envelope, &short_id, || {
      let message = SuccessMessage::new("unarchived project")
        .id(short_id.clone())
        .field("root", restored.root().display().to_string());
      format!(
        "{message}\n\n  Workspace paths are not automatically restored \u{2014} run \
        `gest project attach {short_id}` to re-enable workspace discovery."
      )
    })?;
    Ok(())
  }
}
