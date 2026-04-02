use clap::Args;

use crate::{
  action,
  cli::{self, AppContext},
  store,
  ui::views::note::NoteDetailView,
};

/// Show a single note on a task.
#[derive(Debug, Args)]
pub struct Command {
  /// Task ID or unique prefix.
  pub task_id: String,
  /// Note ID or unique prefix.
  pub note_id: String,
  /// Output as JSON.
  #[arg(long, short = 'j')]
  pub json: bool,
}

impl Command {
  /// Print the full note with attribution and rendered markdown body.
  pub fn call(&self, ctx: &AppContext) -> cli::Result<()> {
    let config = &ctx.settings;
    let theme = &ctx.theme;
    let task_id = store::resolve_task_id(config, &self.task_id, true)?;
    let note = action::resolve_note_prefix(config, &task_id, &self.note_id)?;

    if self.json {
      let json = serde_json::to_string_pretty(&note)?;
      println!("{json}");
      return Ok(());
    }

    let view = NoteDetailView {
      note: &note,
      theme,
    };
    println!("{view}");
    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  mod call {
    use super::*;
    use crate::{
      model::{NewNote, note::AuthorType},
      test_helpers::{make_test_context, make_test_task},
    };

    #[test]
    fn it_errors_on_missing_note() {
      let dir = tempfile::tempdir().unwrap();
      let ctx = make_test_context(dir.path());
      let task = make_test_task("zyxwvutsrqponmlkzyxwvutsrqponmlk");
      store::write_task(&ctx.settings, &task).unwrap();

      let cmd = Command {
        task_id: "zyxw".to_string(),
        note_id: "nonexistent".to_string(),
        json: false,
      };
      assert!(cmd.call(&ctx).is_err());
    }

    #[test]
    fn it_shows_a_note() {
      let dir = tempfile::tempdir().unwrap();
      let ctx = make_test_context(dir.path());
      let task = make_test_task("zyxwvutsrqponmlkzyxwvutsrqponmlk");
      store::write_task(&ctx.settings, &task).unwrap();

      let new = NewNote {
        author: "claude".to_string(),
        author_email: None,
        author_type: AuthorType::Agent,
        body: "Detailed observation".to_string(),
      };
      let note = store::note::add_note(&ctx.settings, &task.id, new).unwrap();

      let cmd = Command {
        task_id: "zyxw".to_string(),
        note_id: note.id.short(),
        json: false,
      };
      cmd.call(&ctx).unwrap();
    }

    #[test]
    fn it_shows_a_note_as_json() {
      let dir = tempfile::tempdir().unwrap();
      let ctx = make_test_context(dir.path());
      let task = make_test_task("zyxwvutsrqponmlkzyxwvutsrqponmlk");
      store::write_task(&ctx.settings, &task).unwrap();

      let new = NewNote {
        author: "alice".to_string(),
        author_email: Some("alice@example.com".to_string()),
        author_type: AuthorType::Human,
        body: "A human note".to_string(),
      };
      let note = store::note::add_note(&ctx.settings, &task.id, new).unwrap();

      let cmd = Command {
        task_id: "zyxw".to_string(),
        note_id: note.id.short(),
        json: true,
      };
      cmd.call(&ctx).unwrap();
    }
  }
}
