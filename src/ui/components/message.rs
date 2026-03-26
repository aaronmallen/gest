use std::io;

use yansi::Paint;

use crate::{model::Id, ui::theme::Theme};

/// Message displayed when `init` finds everything already set up.
///
/// Produces output like: `OK Already initialized`
pub struct AlreadyInitialized;

impl AlreadyInitialized {
  /// Write the already-initialized message to the given writer.
  pub fn write_to(&self, w: &mut impl io::Write, theme: &Theme) -> io::Result<()> {
    writeln!(w, "{} Already initialized", "OK".paint(theme.success))
  }
}

crate::ui::macros::impl_display_via_write_to!(AlreadyInitialized, theme);

/// Config-set confirmation message.
///
/// Produces output like: `Set harness.command = "codex" in global config`
pub struct ConfigSet {
  key: String,
  scope: String,
  value: String,
}

impl ConfigSet {
  pub fn new(key: &str, value: &str, scope: &str) -> Self {
    Self {
      key: key.to_string(),
      scope: scope.to_string(),
      value: value.to_string(),
    }
  }

  /// Write the config-set message to the given writer.
  pub fn write_to(&self, w: &mut impl io::Write, theme: &Theme) -> io::Result<()> {
    writeln!(
      w,
      "{} {} = \"{}\" in {} config",
      "Set".paint(theme.success),
      self.key,
      self.value,
      self.scope,
    )
  }
}

crate::ui::macros::impl_display_via_write_to!(ConfigSet, theme);

/// Confirmation message for create/update/archive actions.
///
/// Produces output like: `Created artifact abcdefgh`
pub struct Confirmation {
  entity: String,
  id: Id,
  verb: String,
}

impl Confirmation {
  pub fn new(verb: &str, entity: &str, id: &Id) -> Self {
    Self {
      entity: entity.to_string(),
      id: id.clone(),
      verb: verb.to_string(),
    }
  }

  /// Write the confirmation message to the given writer.
  pub fn write_to(&self, w: &mut impl io::Write, theme: &Theme) -> io::Result<()> {
    writeln!(
      w,
      "{} {} {}",
      self.verb.paint(theme.success),
      self.entity,
      self.id.short()
    )
  }
}

crate::ui::macros::impl_display_via_write_to!(Confirmation, theme);

/// Empty-list message for task list.
///
/// Produces output: `No tasks found.`
pub struct EmptyList {
  entity: String,
}

impl EmptyList {
  pub fn new(entity: &str) -> Self {
    Self {
      entity: entity.to_string(),
    }
  }

  /// Write the empty-list message to the given writer.
  pub fn write_to(&self, w: &mut impl io::Write) -> io::Result<()> {
    writeln!(w, "No {} found.", self.entity)
  }
}

crate::ui::macros::impl_display_via_write_to!(EmptyList);

/// Error message written to stderr.
///
/// Produces output like: `ERROR some message`
pub struct ErrorMessage {
  message: String,
}

impl ErrorMessage {
  pub fn new(message: &str) -> Self {
    Self {
      message: message.to_string(),
    }
  }

  /// Write the error message to the given writer.
  pub fn write_to(&self, w: &mut impl io::Write, theme: &Theme) -> io::Result<()> {
    writeln!(w, "{} {}", "ERROR".paint(theme.error), self.message)
  }
}

crate::ui::macros::impl_display_via_write_to!(ErrorMessage, theme);

/// Message displayed when `init` creates directories.
///
/// Produces output like: `Created .gest/tasks/`
pub struct InitCreated {
  subdirs: Vec<String>,
}

impl InitCreated {
  pub fn new(subdirs: Vec<String>) -> Self {
    Self {
      subdirs,
    }
  }

  /// Write the init-created message to the given writer.
  pub fn write_to(&self, w: &mut impl io::Write, theme: &Theme) -> io::Result<()> {
    for subdir in &self.subdirs {
      writeln!(w, "{} .gest/{subdir}/", "Created".paint(theme.success),)?;
    }
    Ok(())
  }
}

crate::ui::macros::impl_display_via_write_to!(InitCreated, theme);

/// Link-added message for link actions.
///
/// Produces output like: `Linked abcdefgh --blocks--> eeffgghh`
pub struct LinkAdded {
  rel: String,
  source_id: Id,
  target_id: Id,
}

impl LinkAdded {
  pub fn new(source_id: &Id, rel: &str, target_id: &Id) -> Self {
    Self {
      rel: rel.to_string(),
      source_id: source_id.clone(),
      target_id: target_id.clone(),
    }
  }

  /// Write the link-added message to the given writer.
  pub fn write_to(&self, w: &mut impl io::Write) -> io::Result<()> {
    writeln!(
      w,
      "Linked {} --{}--> {}",
      self.source_id.short(),
      self.rel,
      self.target_id.short()
    )
  }
}

crate::ui::macros::impl_display_via_write_to!(LinkAdded);

/// Metadata-set message for meta set actions.
///
/// Produces output like: `Set abcdefgh priority = "high"`
pub struct MetadataSet {
  id: Id,
  path: String,
  value: String,
}

impl MetadataSet {
  pub fn new(id: &Id, path: &str, value: &str) -> Self {
    Self {
      id: id.clone(),
      path: path.to_string(),
      value: value.to_string(),
    }
  }

  /// Write the metadata-set message to the given writer.
  pub fn write_to(&self, w: &mut impl io::Write) -> io::Result<()> {
    writeln!(w, "Set {} {} = \"{}\"", self.id.short(), self.path, self.value)
  }
}

crate::ui::macros::impl_display_via_write_to!(MetadataSet);

/// No-results message for search.
///
/// Produces output like: `No results found for 'query'`
pub struct NoResults {
  query: String,
}

impl NoResults {
  pub fn new(query: &str) -> Self {
    Self {
      query: query.to_string(),
    }
  }

  /// Write the no-results message to the given writer.
  pub fn write_to(&self, w: &mut impl io::Write) -> io::Result<()> {
    writeln!(w, "No results found for '{}'", self.query)
  }
}

crate::ui::macros::impl_display_via_write_to!(NoResults);

/// Tag change message for tag/untag actions.
///
/// Produces output like: `Tagged task abcdefgh: rust, cli`
pub struct TagChange {
  action: String,
  entity: String,
  id: Id,
  tags: Vec<String>,
}

impl TagChange {
  pub fn new(action: &str, entity: &str, id: &Id, tags: &[String]) -> Self {
    Self {
      action: action.to_string(),
      entity: entity.to_string(),
      id: id.clone(),
      tags: tags.to_vec(),
    }
  }

  /// Write the tag change message to the given writer.
  pub fn write_to(&self, w: &mut impl io::Write) -> io::Result<()> {
    let tag_list = self.tags.join(", ");
    writeln!(w, "{} {} {}: {}", self.action, self.entity, self.id.short(), tag_list)
  }
}

crate::ui::macros::impl_display_via_write_to!(TagChange);

#[cfg(test)]
mod tests {
  use super::*;

  mod already_initialized {
    use super::*;

    mod display {
      use super::*;

      #[test]
      fn it_delegates_to_write_to() {
        let msg = AlreadyInitialized;
        let display = msg.to_string();
        assert!(display.contains("Already initialized"));
      }
    }

    mod write_to {
      use super::*;

      #[test]
      fn it_writes_ok_message() {
        let msg = AlreadyInitialized;
        let theme = Theme::default();
        let mut buf = Vec::new();
        msg.write_to(&mut buf, &theme).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("OK"), "Should contain 'OK'");
        assert!(output.contains("Already initialized"), "Should contain message");
      }
    }
  }

  mod config_set {
    use super::*;

    mod display {
      use super::*;

      #[test]
      fn it_delegates_to_write_to() {
        let msg = ConfigSet::new("storage.data_dir", "/tmp", "project");
        let display = msg.to_string();
        assert!(display.contains("Set"), "Should contain 'Set'");
        assert!(display.contains("in project config"), "Should contain scope");
      }
    }

    mod write_to {
      use super::*;

      #[test]
      fn it_writes_config_set_message() {
        let msg = ConfigSet::new("harness.command", "codex", "global");
        let theme = Theme::default();
        let mut buf = Vec::new();
        msg.write_to(&mut buf, &theme).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("Set"), "Should contain 'Set'");
        assert!(
          output.contains("harness.command = \"codex\" in global config"),
          "Should contain key=value and scope"
        );
      }
    }
  }

  mod confirmation {
    use super::*;

    mod display {
      use super::*;

      #[test]
      fn it_delegates_to_write_to() {
        let id: Id = "zyxwvutsrqponmlkzyxwvutsrqponmlk".parse().unwrap();
        let msg = Confirmation::new("Archived", "task", &id);
        let display = msg.to_string();
        assert!(display.contains("Archived"), "Should contain verb");
        assert!(display.contains("task"), "Should contain entity");
        assert!(display.contains("zyxwvuts"), "Should contain id");
      }
    }

    mod write_to {
      use super::*;

      #[test]
      fn it_writes_verb_entity_and_id() {
        let id: Id = "zyxwvutsrqponmlkzyxwvutsrqponmlk".parse().unwrap();
        let msg = Confirmation::new("Created", "artifact", &id);
        let theme = Theme::default();
        let mut buf = Vec::new();
        msg.write_to(&mut buf, &theme).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("Created"), "Should contain verb");
        assert!(output.contains("artifact"), "Should contain entity");
        assert!(output.contains("zyxwvuts"), "Should contain id");
      }
    }
  }

  mod empty_list {
    use super::*;

    mod display {
      use super::*;

      #[test]
      fn it_delegates_to_write_to() {
        let msg = EmptyList::new("tasks");
        let display = msg.to_string();
        assert!(display.contains("No tasks found."));
      }
    }

    mod write_to {
      use super::*;

      #[test]
      fn it_writes_empty_list_message() {
        let msg = EmptyList::new("tasks");
        let mut buf = Vec::new();
        msg.write_to(&mut buf).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("No tasks found."));
      }
    }
  }

  mod init_created {
    use super::*;

    mod display {
      use super::*;

      #[test]
      fn it_delegates_to_write_to() {
        let msg = InitCreated::new(vec!["tasks".to_string()]);
        let display = msg.to_string();
        assert!(display.contains("Created"), "Should contain 'Created'");
        assert!(display.contains(".gest/tasks/"), "Should contain dir path");
      }
    }

    mod write_to {
      use super::*;

      #[test]
      fn it_writes_created_lines() {
        let msg = InitCreated::new(vec!["tasks".to_string(), "artifacts".to_string()]);
        let theme = Theme::default();
        let mut buf = Vec::new();
        msg.write_to(&mut buf, &theme).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("Created"), "Should contain 'Created'");
        assert!(output.contains(".gest/tasks/"), "Should contain tasks dir");
        assert!(output.contains(".gest/artifacts/"), "Should contain artifacts dir");
      }
    }
  }

  mod link_added {
    use super::*;

    mod display {
      use super::*;

      #[test]
      fn it_delegates_to_write_to() {
        let source: Id = "zyxwvutsrqponmlkzyxwvutsrqponmlk".parse().unwrap();
        let target: Id = "kkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkk".parse().unwrap();
        let msg = LinkAdded::new(&source, "related", &target);
        let display = msg.to_string();
        assert!(display.contains("Linked zyxwvuts --related--> kkkkkkkk"));
      }
    }

    mod write_to {
      use super::*;

      #[test]
      fn it_writes_link_arrow_notation() {
        let source: Id = "zyxwvutsrqponmlkzyxwvutsrqponmlk".parse().unwrap();
        let target: Id = "kkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkk".parse().unwrap();
        let msg = LinkAdded::new(&source, "blocks", &target);
        let mut buf = Vec::new();
        msg.write_to(&mut buf).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("Linked zyxwvuts --blocks--> kkkkkkkk"));
      }
    }
  }

  mod metadata_set {
    use super::*;

    mod display {
      use super::*;

      #[test]
      fn it_delegates_to_write_to() {
        let id: Id = "zyxwvutsrqponmlkzyxwvutsrqponmlk".parse().unwrap();
        let msg = MetadataSet::new(&id, "config.timeout", "30");
        let display = msg.to_string();
        assert!(display.contains("Set zyxwvuts config.timeout = \"30\""));
      }
    }

    mod write_to {
      use super::*;

      #[test]
      fn it_writes_set_notation() {
        let id: Id = "zyxwvutsrqponmlkzyxwvutsrqponmlk".parse().unwrap();
        let msg = MetadataSet::new(&id, "priority", "high");
        let mut buf = Vec::new();
        msg.write_to(&mut buf).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("Set zyxwvuts priority = \"high\""));
      }
    }
  }

  mod no_results {
    use super::*;

    mod display {
      use super::*;

      #[test]
      fn it_delegates_to_write_to() {
        let msg = NoResults::new("search term");
        let display = msg.to_string();
        assert!(display.contains("No results found for 'search term'"));
      }
    }

    mod write_to {
      use super::*;

      #[test]
      fn it_writes_no_results_message() {
        let msg = NoResults::new("test query");
        let mut buf = Vec::new();
        msg.write_to(&mut buf).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("No results found for 'test query'"));
      }
    }
  }

  mod tag_change {
    use super::*;

    mod display {
      use super::*;

      #[test]
      fn it_delegates_to_write_to() {
        let id: Id = "zyxwvutsrqponmlkzyxwvutsrqponmlk".parse().unwrap();
        let tags = vec!["rust".to_string()];
        let msg = TagChange::new("Untagged", "task", &id, &tags);
        let display = msg.to_string();
        assert!(display.contains("Untagged task zyxwvuts: rust"));
      }
    }

    mod write_to {
      use super::*;

      #[test]
      fn it_writes_action_and_tags() {
        let id: Id = "zyxwvutsrqponmlkzyxwvutsrqponmlk".parse().unwrap();
        let tags = vec!["rust".to_string(), "cli".to_string()];
        let msg = TagChange::new("Tagged", "task", &id, &tags);
        let mut buf = Vec::new();
        msg.write_to(&mut buf).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("Tagged task zyxwvuts: rust, cli"));
      }
    }
  }
}
