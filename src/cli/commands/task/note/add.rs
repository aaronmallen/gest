use clap::Args;

use crate::{
  cli::{self, AppContext},
  model::{NewNote, note::AuthorType},
  store,
  ui::composites::success_message::SuccessMessage,
};

/// Add a note to a task.
#[derive(Debug, Args)]
pub struct Command {
  /// Task ID or unique prefix.
  pub id: String,
  /// Agent name for attribution (mutually exclusive with git-derived authorship).
  #[arg(long)]
  pub agent: Option<String>,
  /// Note body text (opens `$EDITOR` if omitted and stdin is a terminal).
  #[arg(short, long)]
  pub body: Option<String>,
  /// Output as JSON.
  #[arg(short, long, conflicts_with = "quiet")]
  pub json: bool,
  /// Print only the note ID.
  #[arg(short, long, conflicts_with = "json")]
  pub quiet: bool,
}

impl Command {
  /// Create a note on the task and print a confirmation.
  pub fn call(&self, ctx: &AppContext) -> cli::Result<()> {
    let config = &ctx.settings;
    let theme = &ctx.theme;
    let task_id = store::resolve_task_id(config, &self.id, true)?;

    let body = crate::cli::helpers::read_from_editor(self.body.as_deref(), ".md", "Aborting: empty note body")?;
    if body.trim().is_empty() {
      return Err(cli::Error::InvalidInput("Aborting: empty note body".into()));
    }

    let (author, author_email, author_type) = if let Some(agent_name) = &self.agent {
      (agent_name.clone(), None, AuthorType::Agent)
    } else {
      let git_author = crate::cli::git::resolve_author().ok_or_else(|| {
        cli::Error::InvalidInput("Could not resolve git user.name; use --agent for agent attribution".into())
      })?;
      (git_author.name, git_author.email, AuthorType::Human)
    };

    let new = NewNote {
      author,
      author_email,
      author_type,
      body,
    };

    let note = store::note::add_note(config, &task_id, new)?;

    if self.json {
      println!("{}", serde_json::to_string_pretty(&note)?);
    } else if self.quiet {
      println!("{}", note.id);
    } else {
      let msg = format!("added note {} to task {}", note.id.short(), task_id.short());
      println!("{}", SuccessMessage::new(&msg, theme));
    }

    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  mod call {
    use super::*;
    use crate::test_helpers::{make_test_context, make_test_task};

    #[test]
    fn it_adds_agent_note() {
      let dir = tempfile::tempdir().unwrap();
      let ctx = make_test_context(dir.path());
      let task = make_test_task("zyxwvutsrqponmlkzyxwvutsrqponmlk");
      store::write_task(&ctx.settings, &task).unwrap();

      let cmd = Command {
        id: "zyxw".to_string(),
        agent: Some("claude".to_string()),
        body: Some("Found the root cause".to_string()),
        json: false,
        quiet: false,
      };
      cmd.call(&ctx).unwrap();

      let notes = store::note::list_notes(&ctx.settings, &task.id).unwrap();
      assert_eq!(notes.len(), 1);
      assert_eq!(notes[0].author, "claude");
      assert_eq!(notes[0].author_type, AuthorType::Agent);
      assert_eq!(notes[0].body, "Found the root cause");
    }

    #[test]
    fn it_rejects_empty_body() {
      let dir = tempfile::tempdir().unwrap();
      let ctx = make_test_context(dir.path());
      let task = make_test_task("zyxwvutsrqponmlkzyxwvutsrqponmlk");
      store::write_task(&ctx.settings, &task).unwrap();

      let cmd = Command {
        id: "zyxw".to_string(),
        agent: Some("claude".to_string()),
        body: Some("".to_string()),
        json: false,
        quiet: false,
      };
      assert!(cmd.call(&ctx).is_err());
    }
  }
}
