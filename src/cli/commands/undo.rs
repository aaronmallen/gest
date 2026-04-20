use clap::Args;

use crate::{AppContext, cli::Error, store::repo, ui::components::SuccessMessage};

/// Undo the last command.
#[derive(Args, Debug)]
pub struct Command {
  /// Number of commands to undo.
  #[arg(default_value = "1")]
  steps: u32,
}

impl Command {
  /// Roll back the most recent `steps` undoable transactions for the current project.
  pub async fn call(&self, context: &AppContext) -> Result<(), Error> {
    log::debug!("undo: entry");
    let project_id = context.project_id().as_ref().ok_or(Error::UninitializedProject)?;
    let conn = context.store().connect().await?;

    let transactions = repo::transaction::latest_undoable_n(&conn, project_id, self.steps).await?;

    if transactions.is_empty() {
      return Err(crate::store::Error::NothingToUndo.into());
    }

    for tx in &transactions {
      let command = repo::transaction::undo(&conn, tx.id()).await?;
      let message = SuccessMessage::new("undone").field("command", command);
      println!("{message}");
    }

    Ok(())
  }
}
