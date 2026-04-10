use clap::Args;

use crate::{
  AppContext,
  cli::{Error, tag_arg},
  store::repo,
  ui::{components::SuccessMessage, json},
};

/// Add tags to any entity (task, artifact, or iteration).
#[derive(Args, Debug)]
pub struct Command {
  /// The entity ID or prefix.
  id: String,
  /// One or more tag labels to add (comma-separated values split into multiple tags).
  #[arg(required = true, value_delimiter = ',', value_parser = tag_arg::trim_tag)]
  tags: Vec<String>,
  #[command(flatten)]
  output: json::Flags,
}

impl Command {
  /// Attach each requested tag to the resolved entity inside a single transaction.
  pub async fn call(&self, context: &AppContext) -> Result<(), Error> {
    log::debug!("tag add: entry");
    let project_id = context.project_id().as_ref().ok_or(Error::UninitializedProject)?;
    let conn = context.store().connect().await?;
    let (entity_type, id) = repo::resolve::resolve_entity(&conn, &self.id).await?;

    let labels = tag_arg::normalize_tags(&self.tags);
    let tx = repo::transaction::begin(&conn, project_id, &format!("{entity_type} tag")).await?;
    let mut attached_tags = Vec::new();
    for label in &labels {
      let tag = repo::tag::attach(&conn, entity_type, &id, label).await?;
      repo::transaction::record_event(&conn, tx.id(), "entity_tags", &tag.id().to_string(), "created", None).await?;
      attached_tags.push(tag);
    }

    let short_id = id.short();
    self.output.print_entity(&attached_tags, &short_id, || {
      SuccessMessage::new(format!("tagged {entity_type}"))
        .id(id.short())
        .field("tags", labels.join(", "))
        .to_string()
    })?;
    Ok(())
  }
}
