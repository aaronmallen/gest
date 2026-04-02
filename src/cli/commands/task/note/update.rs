use clap::Args;

use crate::{
  action,
  cli::{self, AppContext},
  model::NotePatch,
  store,
  ui::composites::success_message::SuccessMessage,
};

/// Update a note on a task.
#[derive(Debug, Args)]
pub struct Command {
  /// Task ID or unique prefix.
  pub task_id: String,
  /// Note ID or unique prefix.
  pub note_id: String,
  /// New body text (opens `$EDITOR` pre-filled if omitted and stdin is a terminal).
  #[arg(short, long)]
  pub body: Option<String>,
}

impl Command {
  /// Edit the note body and persist.
  pub fn call(&self, ctx: &AppContext) -> cli::Result<()> {
    let config = &ctx.settings;
    let theme = &ctx.theme;
    let task_id = store::resolve_task_id(config, &self.task_id, true)?;

    let note = action::resolve_note_prefix(config, &task_id, &self.note_id)?;
    let note_id = note.id;

    let body = if let Some(body) = &self.body {
      body.clone()
    } else {
      crate::cli::helpers::read_from_editor(None, ".md", "Aborting: empty note body")?
    };

    if body.trim().is_empty() {
      return Err(cli::Error::InvalidInput("Aborting: empty note body".into()));
    }

    let patch = NotePatch {
      body: Some(body),
    };

    store::note::update_note(config, &task_id, &note_id, patch)?;

    let msg = format!("updated note {} on task {}", note_id.short(), task_id.short());
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
    fn it_updates_a_note_body() {
      let dir = tempfile::tempdir().unwrap();
      let ctx = make_test_context(dir.path());
      let task = make_test_task("zyxwvutsrqponmlkzyxwvutsrqponmlk");
      store::write_task(&ctx.settings, &task).unwrap();

      let new = NewNote {
        author: "claude".to_string(),
        author_email: None,
        author_type: AuthorType::Agent,
        body: "Original body".to_string(),
      };
      let note = store::note::add_note(&ctx.settings, &task.id, new).unwrap();

      let cmd = Command {
        task_id: "zyxw".to_string(),
        note_id: note.id.short(),
        body: Some("Updated body".to_string()),
      };
      cmd.call(&ctx).unwrap();

      let updated = store::note::read_note(&ctx.settings, &task.id, &note.id).unwrap();
      assert_eq!(updated.body, "Updated body");
    }
  }
}
