use serde_json::Value;

use super::HasMetadata;
use crate::{
  AppContext,
  cli::Error,
  store::{meta, repo},
  ui::{
    components::{FieldList, SuccessMessage},
    envelope::Envelope,
    json,
  },
};

/// Display the full metadata blob for an entity.
pub async fn bare<E: HasMetadata>(context: &AppContext, raw_id: &str, output: &json::Flags) -> Result<(), Error> {
  let conn = context.store().connect().await?;
  let id = repo::resolve::resolve_id(&conn, E::table(), raw_id).await?;
  let model = E::find_by_id(&conn, id).await?;
  let metadata = E::metadata(&model);
  output.print_raw_or(metadata, || render_raw(metadata), || render_normal(metadata))
}

/// Set a metadata value on an entity at a dot-delimited path within a recorded transaction.
pub async fn set<E: HasMetadata>(
  context: &AppContext,
  raw_id: &str,
  path: &str,
  value: &str,
  as_json: bool,
  output: &json::Flags,
) -> Result<(), Error> {
  let project_id = context.project_id().as_ref().ok_or(Error::UninitializedProject)?;
  let conn = context.store().connect().await?;
  let id = repo::resolve::resolve_id(&conn, E::table(), raw_id).await?;
  let model = E::find_by_id(&conn, id.clone()).await?;

  let parsed = if as_json {
    serde_json::from_str(value)?
  } else {
    meta::parse_scalar(value)
  };

  let mut metadata = E::metadata(&model).clone();
  if !metadata.is_object() {
    metadata = Value::Object(serde_json::Map::new());
  }
  if !meta::set_path(&mut metadata, path, parsed) {
    return Err(Error::MetaKeyNotFound(path.to_owned()));
  }

  let before = serde_json::to_value(&model)?;
  let event_table = E::event_table();
  let label = format!("{} meta set", event_table.trim_end_matches('s'));
  let tx = repo::transaction::begin(&conn, project_id, &label).await?;

  let patch = E::metadata_patch(metadata);
  E::update(&conn, &id, &patch).await?;
  repo::transaction::record_event(&conn, tx.id(), event_table, &id.to_string(), "modified", Some(&before)).await?;

  let updated = E::find_by_id(&conn, id.clone()).await?;
  let envelope = Envelope::load_one(&conn, E::entity_type(), &id, &updated, true).await?;
  let short_id = id.short();
  output.print_envelope(&envelope, &short_id, || {
    SuccessMessage::new("set metadata")
      .id(id.short())
      .field("path", path.to_owned())
      .to_string()
  })?;
  Ok(())
}

/// Remove a metadata value from an entity at a dot-delimited path within a recorded transaction.
pub async fn unset<E: HasMetadata>(
  context: &AppContext,
  raw_id: &str,
  path: &str,
  output: &json::Flags,
) -> Result<(), Error> {
  let project_id = context.project_id().as_ref().ok_or(Error::UninitializedProject)?;
  let conn = context.store().connect().await?;
  let id = repo::resolve::resolve_id(&conn, E::table(), raw_id).await?;
  let model = E::find_by_id(&conn, id.clone()).await?;

  let mut metadata = E::metadata(&model).clone();
  if !meta::unset_path(&mut metadata, path) {
    return Err(Error::MetaKeyNotFound(path.to_owned()));
  }

  let before = serde_json::to_value(&model)?;
  let event_table = E::event_table();
  let label = format!("{} meta unset", event_table.trim_end_matches('s'));
  let tx = repo::transaction::begin(&conn, project_id, &label).await?;

  let patch = E::metadata_patch(metadata);
  E::update(&conn, &id, &patch).await?;
  repo::transaction::record_event(&conn, tx.id(), event_table, &id.to_string(), "modified", Some(&before)).await?;

  let updated = E::find_by_id(&conn, id.clone()).await?;
  let envelope = Envelope::load_one(&conn, E::entity_type(), &id, &updated, true).await?;
  let short_id = id.short();
  output.print_envelope(&envelope, &short_id, || {
    SuccessMessage::new("unset metadata")
      .id(id.short())
      .field("path", path.to_owned())
      .to_string()
  })?;
  Ok(())
}

fn render_normal(metadata: &Value) -> String {
  let pairs = meta::flatten_dot_paths(metadata);
  if pairs.is_empty() {
    return "(no metadata)".to_string();
  }
  let mut list = FieldList::new();
  for (path, value) in pairs {
    list = list.field(path, value);
  }
  list.to_string()
}

fn render_raw(metadata: &Value) -> String {
  let pairs = meta::flatten_dot_paths(metadata);
  pairs
    .into_iter()
    .map(|(k, v)| format!("{k}={v}"))
    .collect::<Vec<_>>()
    .join("\n")
}
