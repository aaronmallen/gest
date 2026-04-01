use clap::Args;

use crate::{
  cli::{self, AppContext},
  store,
  ui::views::note::NoteListView,
};

/// List all notes on a task.
#[derive(Debug, Args)]
pub struct Command {
  /// Task ID or unique prefix.
  pub id: String,
  /// Output as JSON.
  #[arg(long)]
  pub json: bool,
}

impl Command {
  /// Print all notes for the task.
  pub fn call(&self, ctx: &AppContext) -> cli::Result<()> {
    let config = &ctx.settings;
    let theme = &ctx.theme;
    let task_id = store::resolve_task_id(config, &self.id, true)?;
    let notes = store::note::list_notes(config, &task_id)?;

    if self.json {
      let json = serde_json::to_string_pretty(&notes)?;
      println!("{json}");
      return Ok(());
    }

    if notes.is_empty() {
      println!("No notes on task {}", task_id.short());
      return Ok(());
    }

    let view = NoteListView {
      notes: &notes,
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
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::{
      model::{NewNote, note::AuthorType},
      test_helpers::{make_test_context, make_test_task},
    };

    #[test]
    fn it_lists_empty_notes() {
      let dir = tempfile::tempdir().unwrap();
      let ctx = make_test_context(dir.path());
      let task = make_test_task("zyxwvutsrqponmlkzyxwvutsrqponmlk");
      store::write_task(&ctx.settings, &task).unwrap();

      let cmd = Command {
        id: "zyxw".to_string(),
        json: false,
      };
      cmd.call(&ctx).unwrap();
    }

    #[test]
    fn it_lists_notes_as_json() {
      let dir = tempfile::tempdir().unwrap();
      let ctx = make_test_context(dir.path());
      let task = make_test_task("zyxwvutsrqponmlkzyxwvutsrqponmlk");
      store::write_task(&ctx.settings, &task).unwrap();

      let new = NewNote {
        author: "claude".to_string(),
        author_email: None,
        author_type: AuthorType::Agent,
        body: "A note".to_string(),
      };
      store::note::add_note(&ctx.settings, &task.id, new).unwrap();

      let cmd = Command {
        id: "zyxw".to_string(),
        json: true,
      };
      cmd.call(&ctx).unwrap();

      let notes = store::note::list_notes(&ctx.settings, &task.id).unwrap();
      assert_eq!(notes.len(), 1);
    }
  }
}
