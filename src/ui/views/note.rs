use std::fmt::{self, Display, Formatter};

use crate::{
  model::{Note, note::AuthorType},
  ui::{
    atoms::{id::Id, label::Label, separator::Separator, value::Value},
    markdown,
    theming::theme::Theme,
    utils,
  },
};

/// Renders a single note with full attribution and markdown body.
pub struct NoteDetailView<'a> {
  pub note: &'a Note,
  pub theme: &'a Theme,
}

impl Display for NoteDetailView<'_> {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    let note = self.note;
    let max_label = 7;

    let short_id = note.id.short();
    let id_atom = Id::new(&short_id, self.theme);
    writeln!(f, "{id_atom}")?;
    writeln!(f)?;

    let author_display = format_author(note);
    let label = Label::new("author", self.theme.task_detail_label).pad_to(max_label);
    let val = Value::new(&author_display, self.theme.task_detail_value);
    writeln!(f, "  {label}  {val}")?;

    let created = note.created_at.format("%Y-%m-%d %H:%M").to_string();
    let label = Label::new("created", self.theme.task_detail_label).pad_to(max_label);
    let val = Value::new(&created, self.theme.task_detail_value);
    writeln!(f, "  {label}  {val}")?;

    if note.updated_at != note.created_at {
      let updated = note.updated_at.format("%Y-%m-%d %H:%M").to_string();
      let label = Label::new("updated", self.theme.task_detail_label).pad_to(max_label);
      let val = Value::new(&updated, self.theme.task_detail_value);
      writeln!(f, "  {label}  {val}")?;
    }

    writeln!(f)?;
    writeln!(f, "{}", Separator::labeled("body", self.theme.task_detail_separator))?;
    writeln!(f)?;
    let width = utils::terminal_width() as usize;
    let rendered = markdown::render(&note.body);
    for line in rendered.lines() {
      writeln!(f, "  {line}")?;
    }
    writeln!(f)?;
    write!(f, "{}", Separator::rule(self.theme.task_detail_separator))?;
    Ok(())
  }
}

/// Renders a list of notes with author attribution.
pub struct NoteListView<'a> {
  pub notes: &'a [Note],
  pub theme: &'a Theme,
}

impl Display for NoteListView<'_> {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    let max_label = 7; // "created" is the longest label
    for (i, note) in self.notes.iter().enumerate() {
      if i > 0 {
        writeln!(f)?;
      }

      let short_id = note.id.short();
      let id_atom = Id::new(&short_id, self.theme);
      writeln!(f, "{id_atom}")?;

      let author_display = format_author(note);
      let label = Label::new("author", self.theme.task_detail_label).pad_to(max_label);
      let val = Value::new(&author_display, self.theme.task_detail_value);
      writeln!(f, "  {label}  {val}")?;

      let created = note.created_at.format("%Y-%m-%d %H:%M").to_string();
      let label = Label::new("created", self.theme.task_detail_label).pad_to(max_label);
      let val = Value::new(&created, self.theme.task_detail_value);
      writeln!(f, "  {label}  {val}")?;

      let first_line = note.body.lines().next().unwrap_or("");
      let preview = if first_line.len() > 80 {
        format!("{}...", &first_line[..77])
      } else {
        first_line.to_string()
      };
      let label = Label::new("body", self.theme.task_detail_label).pad_to(max_label);
      let val = Value::new(&preview, self.theme.task_detail_value);
      writeln!(f, "  {label}  {val}")?;
    }
    Ok(())
  }
}

fn format_author(note: &Note) -> String {
  match note.author_type {
    AuthorType::Agent => format!("{} (agent)", note.author),
    AuthorType::Human => match &note.author_email {
      Some(email) => format!("{} <{}>", note.author, email),
      None => note.author.clone(),
    },
  }
}
