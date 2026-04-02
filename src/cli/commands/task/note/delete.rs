use clap::Args;

use crate::{
  action,
  cli::{self, AppContext},
  store,
  ui::composites::success_message::SuccessMessage,
};

/// Delete a note from a task.
#[derive(Debug, Args)]
pub struct Command {
  /// Task ID or unique prefix.
  pub task_id: String,
  /// Note ID or unique prefix.
  pub note_id: String,
}

impl Command {
  /// Remove the note and print a confirmation.
  pub fn call(&self, ctx: &AppContext) -> cli::Result<()> {
    let config = &ctx.settings;
    let theme = &ctx.theme;
    let task_id = store::resolve_task_id(config, &self.task_id, true)?;

    let note = action::resolve_note_prefix(config, &task_id, &self.note_id)?;
    let note_id = note.id;

    store::note::delete_note(config, &task_id, &note_id)?;

    let msg = format!("deleted note {} from task {}", note_id.short(), task_id.short());
    println!("{}", SuccessMessage::new(&msg, theme));
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
    fn it_deletes_a_note() {
      let dir = tempfile::tempdir().unwrap();
      let ctx = make_test_context(dir.path());
      let task = make_test_task("zyxwvutsrqponmlkzyxwvutsrqponmlk");
      store::write_task(&ctx.settings, &task).unwrap();

      let new = NewNote {
        author: "claude".to_string(),
        author_email: None,
        author_type: AuthorType::Agent,
        body: "To be deleted".to_string(),
      };
      let note = store::note::add_note(&ctx.settings, &task.id, new).unwrap();

      let cmd = Command {
        task_id: "zyxw".to_string(),
        note_id: note.id.short(),
      };
      cmd.call(&ctx).unwrap();

      let notes = store::note::list_notes(&ctx.settings, &task.id).unwrap();
      assert_eq!(notes.len(), 0);
    }
  }
}
