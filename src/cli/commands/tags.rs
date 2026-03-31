use chrono::{DateTime, Utc};

use crate::{
  cli::{self, AppContext},
  config::Settings,
  model::id::Id,
  store,
  ui::composites::success_message::SuccessMessage,
};

/// Append tags from `to_add` into `tags`, skipping duplicates.
pub fn apply_tags(tags: &mut Vec<String>, to_add: &[String]) {
  for tag in to_add {
    if !tags.contains(tag) {
      tags.push(tag.clone());
    }
  }
}

/// Remove all entries in `to_remove` from `tags`.
pub fn remove_tags(tags: &mut Vec<String>, to_remove: &[String]) {
  tags.retain(|t| !to_remove.contains(t));
}

/// Generic helper for the tag command flow: resolve ID, read entity, apply tags, write entity.
#[allow(clippy::too_many_arguments)]
pub fn tag_entity<E>(
  ctx: &AppContext,
  id_prefix: &str,
  tags: &[String],
  noun: &str,
  resolve: impl FnOnce(&Settings, &str, bool) -> store::Result<Id>,
  read: impl FnOnce(&Settings, &Id) -> store::Result<E>,
  get_tags: impl FnOnce(&mut E) -> &mut Vec<String>,
  set_updated_at: impl FnOnce(&mut E, DateTime<Utc>),
  write: impl FnOnce(&Settings, &E) -> store::Result<()>,
) -> cli::Result<()> {
  let config = &ctx.settings;
  let id = resolve(config, id_prefix, false)?;
  let mut entity = read(config, &id)?;

  apply_tags(get_tags(&mut entity), tags);

  set_updated_at(&mut entity, Utc::now());
  write(config, &entity)?;

  let msg = format!("Tagged {noun} {id} with {}", tags.join(", "));
  println!("{}", SuccessMessage::new(&msg, &ctx.theme));
  Ok(())
}

/// Generic helper for the untag command flow: resolve ID, read entity, remove tags, write entity.
#[allow(clippy::too_many_arguments)]
pub fn untag_entity<E>(
  ctx: &AppContext,
  id_prefix: &str,
  tags: &[String],
  noun: &str,
  resolve: impl FnOnce(&Settings, &str, bool) -> store::Result<Id>,
  read: impl FnOnce(&Settings, &Id) -> store::Result<E>,
  get_tags: impl FnOnce(&mut E) -> &mut Vec<String>,
  set_updated_at: impl FnOnce(&mut E, DateTime<Utc>),
  write: impl FnOnce(&Settings, &E) -> store::Result<()>,
) -> cli::Result<()> {
  let config = &ctx.settings;
  let id = resolve(config, id_prefix, false)?;
  let mut entity = read(config, &id)?;

  remove_tags(get_tags(&mut entity), tags);

  set_updated_at(&mut entity, Utc::now());
  write(config, &entity)?;

  let msg = format!("Untagged {noun} {id} from {}", tags.join(", "));
  println!("{}", SuccessMessage::new(&msg, &ctx.theme));
  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;

  mod apply_tags {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_adds_new_tags() {
      let mut tags = vec!["a".to_string()];
      apply_tags(&mut tags, &["b".to_string(), "c".to_string()]);

      assert_eq!(tags, vec!["a", "b", "c"]);
    }

    #[test]
    fn it_skips_duplicates() {
      let mut tags = vec!["a".to_string(), "b".to_string()];
      apply_tags(&mut tags, &["b".to_string(), "c".to_string()]);

      assert_eq!(tags, vec!["a", "b", "c"]);
    }
  }

  mod remove_tags {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_filters_matching_tags() {
      let mut tags = vec!["a".to_string(), "b".to_string(), "c".to_string()];
      remove_tags(&mut tags, &["b".to_string()]);

      assert_eq!(tags, vec!["a", "c"]);
    }

    #[test]
    fn it_ignores_absent_tags() {
      let mut tags = vec!["a".to_string()];
      remove_tags(&mut tags, &["z".to_string()]);

      assert_eq!(tags, vec!["a"]);
    }
  }
}
