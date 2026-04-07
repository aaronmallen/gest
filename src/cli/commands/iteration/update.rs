use clap::Args;

use crate::{
  AppContext,
  cli::Error,
  store::{model::iteration::Patch, repo},
  ui::{components::SuccessMessage, json},
};

/// Update an iteration.
#[derive(Args, Debug)]
pub struct Command {
  /// The iteration ID or prefix.
  id: String,
  /// Set the iteration description.
  #[arg(long, short)]
  description: Option<String>,
  /// Set the iteration title.
  #[arg(long, short)]
  title: Option<String>,
  #[command(flatten)]
  output: json::Flags,
}

impl Command {
  pub async fn call(&self, context: &AppContext) -> Result<(), Error> {
    let project_id = context.project_id().as_ref().ok_or(Error::UninitializedProject)?;
    let conn = context.store().connect().await?;

    let id = repo::resolve::resolve_id(&conn, "iterations", &self.id).await?;
    let before_iter = repo::iteration::find_by_id(&conn, id.clone())
      .await?
      .ok_or(Error::UninitializedProject)?;
    let before = serde_json::to_value(&before_iter)?;
    let tx = repo::transaction::begin(&conn, project_id, "iteration update").await?;
    let patch = Patch {
      description: self.description.clone(),
      title: self.title.clone(),
      ..Default::default()
    };

    let iteration = repo::iteration::update(&conn, &id, &patch).await?;
    repo::transaction::record_event(&conn, tx.id(), "iterations", &id.to_string(), "modified", Some(&before)).await?;

    let short_id = iteration.id().short();
    self.output.print_entity(&iteration, &short_id, || {
      SuccessMessage::new("updated iteration")
        .id(iteration.id().short())
        .field("title", iteration.title().to_string())
        .to_string()
    })?;
    Ok(())
  }
}
