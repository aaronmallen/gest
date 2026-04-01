use chrono::Utc;

use crate::{
  config::Settings,
  model::{Id, NewNote, Note, NotePatch},
};

/// Add a new note to a task.
pub fn add_note(config: &Settings, task_id: &Id, new: NewNote) -> super::Result<Note> {
  let mut task = super::read_task(config, task_id)?;
  let now = Utc::now();
  let note = Note {
    author: new.author,
    author_email: new.author_email,
    author_type: new.author_type,
    body: new.body,
    created_at: now,
    id: super::fs::next_id(config)?,
    updated_at: now,
  };
  task.notes.push(note.clone());
  task.updated_at = now;
  super::write_task(config, &task)?;
  Ok(note)
}

/// List all notes for a task, ordered by creation time (oldest first).
pub fn list_notes(config: &Settings, task_id: &Id) -> super::Result<Vec<Note>> {
  let task = super::read_task(config, task_id)?;
  let mut notes = task.notes;
  notes.sort_by_key(|a| a.created_at);
  Ok(notes)
}

/// Read a single note by ID from a task.
#[allow(dead_code)]
pub fn read_note(config: &Settings, task_id: &Id, note_id: &Id) -> super::Result<Note> {
  let task = super::read_task(config, task_id)?;
  task
    .notes
    .into_iter()
    .find(|n| n.id == *note_id)
    .ok_or_else(|| super::Error::generic(format!("Note {note_id} not found on task {task_id}")))
}

/// Update an existing note on a task.
pub fn update_note(config: &Settings, task_id: &Id, note_id: &Id, patch: NotePatch) -> super::Result<Note> {
  let mut task = super::read_task(config, task_id)?;
  let note = task
    .notes
    .iter_mut()
    .find(|n| n.id == *note_id)
    .ok_or_else(|| super::Error::generic(format!("Note {note_id} not found on task {task_id}")))?;

  if let Some(body) = patch.body {
    note.body = body;
  }
  note.updated_at = Utc::now();
  task.updated_at = note.updated_at;

  let updated = note.clone();
  super::write_task(config, &task)?;
  Ok(updated)
}

/// Delete a note from a task.
pub fn delete_note(config: &Settings, task_id: &Id, note_id: &Id) -> super::Result<()> {
  let mut task = super::read_task(config, task_id)?;
  let len_before = task.notes.len();
  task.notes.retain(|n| n.id != *note_id);
  if task.notes.len() == len_before {
    return Err(super::Error::generic(format!(
      "Note {note_id} not found on task {task_id}"
    )));
  }
  task.updated_at = Utc::now();
  super::write_task(config, &task)?;
  Ok(())
}
