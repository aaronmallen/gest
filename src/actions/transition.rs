//! Generic status-transition pipeline shared by task and iteration lifecycle
//! commands.
//!
//! Concrete commands (e.g. `task cancel`, `iteration complete`) resolve an
//! entity ID and delegate here. The generic absorbs the six repeated steps:
//! resolve → load before-snapshot → begin transaction → patch → record event
//! → emit envelope. Entity-specific details are provided through the
//! [`HasStatus`] trait.

use std::fmt::Display;

use super::{Findable, HasMetadata, Prefixable};
use crate::{
  AppContext,
  cli::Error,
  store::repo,
  ui::{components::SuccessMessage, envelope::Envelope, json},
};

/// An entity whose lifecycle is tracked by a status enum that can be patched
/// atomically via [`HasMetadata::update`].
pub trait HasStatus: Findable + HasMetadata + Prefixable {
  /// The lifecycle status enum (e.g. `TaskStatus`, `IterationStatus`).
  type Status: Copy + Display + Send;

  /// Read the current status off the domain model.
  fn status(model: &Self::Model) -> Self::Status;

  /// Produce a patch that sets only the status field to `status`.
  fn status_patch(status: Self::Status) -> Self::Patch;

  /// Read the human-readable title off the domain model, used by success
  /// messages.
  fn title(model: &Self::Model) -> &str;
}

/// Transition `raw_id` to `new_status` inside a recorded transaction.
///
/// Returns after printing either a JSON envelope or a [`SuccessMessage`]
/// (based on `output`). `action` is the transaction description
/// (e.g. `"task cancel"`), `semantic_type` is the timeline label
/// (e.g. `"cancelled"`, `"status-change"`), and `success_label` is the
/// imperative past-tense phrase shown to the user (e.g. `"cancelled task"`).
pub async fn transition_status<E: HasStatus>(
  context: &AppContext,
  raw_id: &str,
  new_status: E::Status,
  action: &str,
  semantic_type: &str,
  success_label: &str,
  output: &json::Flags,
) -> Result<(), Error> {
  let entity_name = E::entity_type().to_string();
  log::debug!("{entity_name} transition: entry");

  let project_id = context.project_id().as_ref().ok_or(Error::UninitializedProject)?;
  let conn = context.store().connect().await?;
  let id = repo::resolve::resolve_id(&conn, E::table(), raw_id).await?;

  let before_model = E::find_by_id(&conn, id.clone()).await?;
  let before_status = E::status(&before_model);
  let before_value = serde_json::to_value(&before_model)?;

  let tx = repo::transaction::begin(&conn, project_id, action).await?;
  let patch = E::status_patch(new_status);
  let model = E::update(&conn, &id, &patch).await?;
  repo::transaction::record_semantic_event(
    &conn,
    tx.id(),
    E::event_table(),
    &id.to_string(),
    "modified",
    Some(&before_value),
    Some(semantic_type),
    Some(&before_status.to_string()),
    Some(&E::status(&model).to_string()),
  )
  .await?;

  let envelope = Envelope::load_one(&conn, E::entity_type(), &id, &model, true).await?;
  let prefix_len = E::prefix_length(&conn, project_id, &id.to_string()).await?;

  let short_id = id.short();
  output.print_envelope(&envelope, &short_id, || {
    log::info!("{success_label}");
    SuccessMessage::new(success_label)
      .id(short_id.clone())
      .prefix_len(prefix_len)
      .field("title", E::title(&model).to_string())
      .to_string()
  })?;
  Ok(())
}
