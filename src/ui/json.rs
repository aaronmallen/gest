//! Shared output-mode helpers for `--json` and `--quiet` flags.

use clap::Args;
use serde::Serialize;

use crate::{cli::Error, ui::envelope::Envelope};

/// Common output-mode flags that can be flattened into any command struct.
#[derive(Args, Clone, Debug, Default)]
pub struct Flags {
  /// Emit output as JSON.
  #[arg(short, long)]
  pub json: bool,
  /// Suppress normal output (print only entity ID on create/mutate, nothing on delete).
  #[arg(short, long)]
  pub quiet: bool,
  /// Emit script-friendly plain output (no themed styling).
  #[arg(short, long)]
  pub raw: bool,
}

impl Flags {
  /// Print nothing (for delete in quiet mode), or the normal message.
  pub fn print_delete(&self, normal: impl FnOnce() -> String) -> Result<(), Error> {
    if self.json {
      // JSON mode on delete: print empty object
      println!("{{}}");
    } else if self.quiet {
      // nothing
    } else {
      println!("{}", normal());
    }
    Ok(())
  }

  /// Print a single entity wrapped in an [`Envelope`]. In JSON mode the
  /// envelope is serialized; in quiet mode only the short ID is printed;
  /// otherwise `normal` is called.
  pub fn print_envelope<T: Serialize>(
    &self,
    envelope: &Envelope<'_, T>,
    short_id: &str,
    normal: impl FnOnce() -> String,
  ) -> Result<(), Error> {
    if self.json {
      let json = serde_json::to_string_pretty(envelope)?;
      println!("{json}");
    } else if self.quiet {
      println!("{short_id}");
    } else {
      println!("{}", normal());
    }
    Ok(())
  }

  /// Print a list of envelopes. In JSON mode the list is serialized; in raw
  /// mode the `raw` callback renders script-friendly output; otherwise
  /// `normal` is used.
  pub fn print_envelopes<T: Serialize>(
    &self,
    envelopes: &[Envelope<'_, T>],
    raw: impl FnOnce() -> String,
    normal: impl FnOnce() -> String,
  ) -> Result<(), Error> {
    if self.json {
      let json = serde_json::to_string_pretty(envelopes)?;
      println!("{json}");
    } else if self.raw {
      println!("{}", raw());
    } else if self.quiet {
      // nothing
    } else {
      println!("{}", normal());
    }
    Ok(())
  }

  /// Print a list of envelopes with quiet-mode short IDs. In JSON mode the
  /// envelopes are serialized; in quiet mode each short ID is printed on its
  /// own line; otherwise `normal` is called for the human-readable summary.
  pub fn print_envelopes_with_short_ids<T: Serialize>(
    &self,
    envelopes: &[Envelope<'_, T>],
    short_ids: impl FnOnce() -> Vec<String>,
    normal: impl FnOnce() -> String,
  ) -> Result<(), Error> {
    if self.json {
      let json = serde_json::to_string_pretty(envelopes)?;
      println!("{json}");
    } else if self.quiet {
      for id in short_ids() {
        println!("{id}");
      }
    } else {
      println!("{}", normal());
    }
    Ok(())
  }

  /// Print a collection of entities. In JSON mode the slice is serialized; in
  /// quiet mode each short ID is printed on its own line; otherwise `normal` is
  /// called to produce the human-readable output.
  pub fn print_entities<T: Serialize>(
    &self,
    entities: &[T],
    short_ids: impl FnOnce() -> Vec<String>,
    normal: impl FnOnce() -> String,
  ) -> Result<(), Error> {
    if self.json {
      let json = serde_json::to_string_pretty(entities)?;
      println!("{json}");
    } else if self.quiet {
      for id in short_ids() {
        println!("{id}");
      }
    } else {
      println!("{}", normal());
    }
    Ok(())
  }

  /// Print a list of short entity IDs, one per line.
  ///
  /// This is the `--quiet` form of list-style commands: script-friendly output
  /// where each line is a single short ID that can be piped to another command.
  pub fn print_short_ids<I, S>(&self, ids: I) -> Result<(), Error>
  where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
  {
    for id in ids {
      println!("{}", id.as_ref());
    }
    Ok(())
  }

  /// Print a single entity. In JSON mode the entity is serialized; in quiet mode
  /// only the short ID is printed; otherwise `normal` is called to produce the
  /// human-readable output.
  pub fn print_entity<T: Serialize>(
    &self,
    entity: &T,
    short_id: &str,
    normal: impl FnOnce() -> String,
  ) -> Result<(), Error> {
    if self.json {
      let json = serde_json::to_string_pretty(entity)?;
      println!("{json}");
    } else if self.quiet {
      println!("{short_id}");
    } else {
      println!("{}", normal());
    }
    Ok(())
  }

  /// Print a value with explicit support for `--raw`.
  ///
  /// Mode precedence: `--json` (serialized) > `--raw` (script-friendly plain
  /// text) > `--quiet` (nothing) > themed `normal` output.
  pub fn print_raw_or<T: Serialize>(
    &self,
    value: &T,
    raw: impl FnOnce() -> String,
    normal: impl FnOnce() -> String,
  ) -> Result<(), Error> {
    if let Some(out) = self.render_raw_or(value, raw, normal)? {
      println!("{out}");
    }
    Ok(())
  }

  /// Internal: compute the chosen output for `print_raw_or` without printing.
  ///
  /// Returns `None` for the silent (`--quiet`) branch so callers can suppress
  /// output. Exposed for unit testing the precedence rules.
  fn render_raw_or<T: Serialize>(
    &self,
    value: &T,
    raw: impl FnOnce() -> String,
    normal: impl FnOnce() -> String,
  ) -> Result<Option<String>, Error> {
    if self.json {
      return Ok(Some(serde_json::to_string_pretty(value)?));
    }
    if self.raw {
      return Ok(Some(raw()));
    }
    if self.quiet {
      return Ok(None);
    }
    Ok(Some(normal()))
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  mod render_raw_or {
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::*;

    fn render(flags: Flags) -> Option<String> {
      flags
        .render_raw_or(&json!({"k": 1}), || "RAW".to_string(), || "NORMAL".to_string())
        .unwrap()
    }

    #[test]
    fn it_prefers_json_over_everything() {
      let flags = Flags {
        json: true,
        quiet: true,
        raw: true,
      };

      let out = render(flags).expect("json mode emits output");

      assert!(out.contains("\"k\""));
    }

    #[test]
    fn it_prefers_raw_over_quiet() {
      let flags = Flags {
        json: false,
        quiet: true,
        raw: true,
      };

      assert_eq!(render(flags), Some("RAW".to_string()));
    }

    #[test]
    fn it_returns_none_for_quiet() {
      let flags = Flags {
        json: false,
        quiet: true,
        raw: false,
      };

      assert_eq!(render(flags), None);
    }

    #[test]
    fn it_returns_normal_when_no_flags_set() {
      let flags = Flags::default();

      assert_eq!(render(flags), Some("NORMAL".to_string()));
    }
  }
}
