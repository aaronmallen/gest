//! Generic meta get/set actions for TOML-backed entities.

use chrono::Utc;

use super::{HasMetadata, Resolvable, Storable};
use crate::{
  cli::{self, AppContext},
  store,
};

/// Look up a metadata value by dot-delimited path and return the formatted string.
pub fn meta_get<E>(ctx: &AppContext, prefix: &str, path: &str) -> cli::Result<String>
where
  E: HasMetadata + Resolvable + Storable,
{
  let config = &ctx.settings;
  let id = E::resolve_id(config, prefix)?;
  let entity = E::read(config, &id)?;

  let root = toml::Value::Table(entity.metadata().clone());
  let value = store::meta::resolve_dot_path(&root, path)
    .ok_or_else(|| cli::Error::NotFound(format!("Metadata key not found: '{path}'")))?;

  Ok(store::meta::format_toml_value(value))
}

/// Set a metadata value at a dot-delimited path and persist the entity.
///
/// Returns the updated entity so the caller can format output (JSON, quiet, styled).
pub fn meta_set<E>(ctx: &AppContext, prefix: &str, path: &str, value: &str) -> cli::Result<E>
where
  E: HasMetadata + Resolvable + Storable,
{
  let config = &ctx.settings;
  let id = E::resolve_id(config, prefix)?;
  let mut entity = E::read(config, &id)?;

  store::meta::set_dot_path(entity.metadata_mut(), path, value)?;

  entity.set_updated_at(Utc::now());
  E::write(config, &entity)?;

  Ok(entity)
}
