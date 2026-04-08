use clap::Args;
use serde_json::{Map, Value};

use crate::{
  AppContext,
  cli::Error,
  store::{meta, repo},
  ui::{components::MetaGet, json},
};

/// Get a metadata value from a task by dot-delimited path.
#[derive(Args, Debug)]
pub struct Command {
  /// The task ID or prefix.
  id: String,
  /// The dot-delimited metadata path.
  path: String,
  #[command(flatten)]
  output: json::Flags,
}

impl Command {
  pub async fn call(&self, context: &AppContext) -> Result<(), Error> {
    log::debug!("task meta get: entry");
    let conn = context.store().connect().await?;
    let id = repo::resolve::resolve_id(&conn, "tasks", &self.id).await?;
    let task = repo::task::find_by_id(&conn, id)
      .await?
      .ok_or_else(|| Error::Resolve(repo::resolve::Error::NotFound(self.id.clone())))?;

    let value =
      meta::resolve_path(task.metadata(), &self.path).ok_or_else(|| Error::MetaKeyNotFound(self.path.clone()))?;

    let mut wrapped = Map::new();
    wrapped.insert(self.path.clone(), value.clone());
    let wrapped = Value::Object(wrapped);

    self.output.print_raw_or(
      &wrapped,
      || meta::format_meta_value(value),
      || MetaGet::new(meta::format_meta_value(value)).to_string(),
    )
  }
}
