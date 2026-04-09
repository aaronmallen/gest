use crate::{
  AppContext,
  cli::Error,
  store::{meta, repo},
  ui::{components::FieldList, json},
};

/// Render the full metadata blob for an iteration.
pub async fn call(context: &AppContext, raw_id: &str, output: &json::Flags) -> Result<(), Error> {
  let conn = context.store().connect().await?;
  let id = repo::resolve::resolve_id(&conn, "iterations", raw_id).await?;
  let iteration = repo::iteration::find_required_by_id(&conn, id).await?;

  let metadata = iteration.metadata();

  output.print_raw_or(metadata, || render_raw(metadata), || render_normal(metadata))
}

fn render_normal(metadata: &serde_json::Value) -> String {
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

fn render_raw(metadata: &serde_json::Value) -> String {
  let pairs = meta::flatten_dot_paths(metadata);
  pairs
    .into_iter()
    .map(|(k, v)| format!("{k}={v}"))
    .collect::<Vec<_>>()
    .join("\n")
}
