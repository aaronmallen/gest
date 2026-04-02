use crate::{
  cli,
  config::Settings,
  model::{Id, Note},
  store,
};

/// Resolve a note by ID prefix on the given task.
///
/// Lists all notes for `task_id` and returns the one whose full ID starts with
/// `note_prefix`. Returns [`cli::Error::NotFound`] when no note matches.
pub fn resolve_note_prefix(config: &Settings, task_id: &Id, note_prefix: &str) -> cli::Result<Note> {
  let notes = store::note::list_notes(config, task_id)?;

  notes
    .into_iter()
    .find(|n| n.id.to_string().starts_with(note_prefix))
    .ok_or_else(|| cli::Error::NotFound(format!("Note matching '{}' not found on task {}", note_prefix, task_id)))
}

#[cfg(test)]
mod tests {
  use super::*;

  mod resolve_note_prefix_fn {
    use super::*;
    use crate::{
      model::{NewNote, note::AuthorType},
      test_helpers::{make_test_context, make_test_task},
    };

    #[test]
    fn it_finds_note_by_prefix() {
      let dir = tempfile::tempdir().unwrap();
      let ctx = make_test_context(dir.path());
      let task = make_test_task("zyxwvutsrqponmlkzyxwvutsrqponmlk");
      store::write_task(&ctx.settings, &task).unwrap();

      let new = NewNote {
        author: "claude".to_string(),
        author_email: None,
        author_type: AuthorType::Agent,
        body: "Test note".to_string(),
      };
      let note = store::note::add_note(&ctx.settings, &task.id, new).unwrap();

      let found = resolve_note_prefix(&ctx.settings, &task.id, &note.id.short()).unwrap();
      assert_eq!(found.id, note.id);
    }

    #[test]
    fn it_errors_on_missing_note() {
      let dir = tempfile::tempdir().unwrap();
      let ctx = make_test_context(dir.path());
      let task = make_test_task("zyxwvutsrqponmlkzyxwvutsrqponmlk");
      store::write_task(&ctx.settings, &task).unwrap();

      let result = resolve_note_prefix(&ctx.settings, &task.id, "nonexistent");
      assert!(result.is_err());
    }
  }
}
