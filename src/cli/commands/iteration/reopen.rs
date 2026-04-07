use clap::Args;

use crate::{
  AppContext,
  cli::Error,
  store::{
    model::{iteration::Patch, primitives::IterationStatus},
    repo,
  },
  ui::{components::SuccessMessage, json},
};

/// Reopen a completed or cancelled iteration.
#[derive(Args, Debug)]
pub struct Command {
  /// The iteration ID or prefix.
  id: String,
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
    let tx = repo::transaction::begin(&conn, project_id, "iteration reopen").await?;
    let patch = Patch {
      status: Some(IterationStatus::Active),
      ..Default::default()
    };

    let iteration = repo::iteration::update(&conn, &id, &patch).await?;
    repo::transaction::record_semantic_event(
      &conn,
      tx.id(),
      "iterations",
      &id.to_string(),
      "modified",
      Some(&before),
      Some("status-change"),
      Some(&before_iter.status().to_string()),
      Some(&iteration.status().to_string()),
    )
    .await?;

    let prefix_len = repo::iteration::shortest_active_prefix(&conn, project_id).await?;

    let short_id = iteration.id().short();
    self.output.print_entity(&iteration, &short_id, || {
      SuccessMessage::new("reopened iteration")
        .id(iteration.id().short())
        .prefix_len(prefix_len)
        .field("title", iteration.title().to_string())
        .to_string()
    })?;
    Ok(())
  }
}
