//! Display-ready note structure for rendering in templates.

use libsql::Connection;

use crate::{
  store::{
    model::{note, primitives::AuthorType},
    repo,
  },
  web::{gravatar, markdown},
};

/// A display-friendly representation of a note, with the author resolved.
pub(crate) struct NoteDisplay {
  pub(crate) author_gravatar: Option<String>,
  pub(crate) author_is_agent: bool,
  pub(crate) author_name: Option<String>,
  pub(crate) body_html: String,
  pub(crate) created_at: String,
  pub(crate) id_short: String,
}

/// Convert raw note models into display structs, resolving authors.
pub(crate) async fn build_note_displays(conn: &Connection, notes: Vec<note::Model>) -> Vec<NoteDisplay> {
  let mut displays = Vec::with_capacity(notes.len());
  for n in notes {
    let (author_name, author_gravatar, author_is_agent) = match n.author_id() {
      Some(aid) => match repo::author::find_by_id(conn, aid.clone()).await {
        Ok(Some(author)) => (
          Some(author.name().to_string()),
          gravatar::url(author.email()),
          author.author_type() == AuthorType::Agent,
        ),
        _ => (None, None, false),
      },
      None => (None, None, false),
    };
    displays.push(NoteDisplay {
      author_gravatar,
      author_is_agent,
      author_name,
      body_html: markdown::render_markdown_to_html(n.body()),
      created_at: n.created_at().format("%Y-%m-%d %H:%M UTC").to_string(),
      id_short: n.id().short(),
    });
  }
  displays
}
