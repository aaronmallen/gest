use clap::Args;

use crate::{
  AppContext,
  cli::{Error, tag_arg},
  store::{model::entity_tag, repo},
  ui::{components::SuccessMessage, json},
};

/// Remove tags from any entity (task, artifact, or iteration).
#[derive(Args, Debug)]
pub struct Command {
  /// The entity ID or prefix.
  id: String,
  /// One or more tag labels to remove (comma-separated values split into multiple tags).
  #[arg(required = true, value_delimiter = ',', value_parser = tag_arg::trim_tag)]
  tags: Vec<String>,
  #[command(flatten)]
  output: json::Flags,
}

impl Command {
  /// Detach each requested tag from the resolved entity inside a single transaction.
  pub async fn call(&self, context: &AppContext) -> Result<(), Error> {
    log::debug!("tag remove: entry");
    let project_id = context.project_id().as_ref().ok_or(Error::UninitializedProject)?;
    let conn = context.store().connect().await?;
    let (entity_type, id) = repo::resolve::resolve_entity(&conn, &self.id).await?;

    let labels = tag_arg::normalize_tags(&self.tags);
    let tx = repo::transaction::begin(&conn, project_id, &format!("{entity_type} untag")).await?;
    for label in &labels {
      let before_tag = repo::tag::find_by_label(&conn, label).await?;
      repo::tag::detach(&conn, entity_type, &id, label).await?;
      if let Some(tag) = &before_tag {
        let before = serde_json::to_value(entity_tag::Model::new(entity_type, id.clone(), tag.id().clone()))?;
        repo::transaction::record_event(
          &conn,
          tx.id(),
          "entity_tags",
          &tag.id().to_string(),
          "deleted",
          Some(&before),
        )
        .await?;
      }
    }

    let short_id = id.short();
    self.output.print_delete(|| {
      SuccessMessage::new(format!("untagged {entity_type}"))
        .id(short_id.clone())
        .field("tags", labels.join(", "))
        .to_string()
    })?;
    Ok(())
  }
}
