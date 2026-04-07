use std::{io::Error as IoError, path::Path, process::Command};

/// Errors that can occur when launching an external editor.
#[derive(Debug, thiserror::Error)]
pub enum Error {
  #[error("Editor '{0}' exited with {1}")]
  EditorFailed(String, String),
  #[error("No editor configured: set $EDITOR or $VISUAL")]
  EditorNotFound,
  #[error(transparent)]
  Io(#[from] IoError),
}

/// Open the user's preferred editor with `initial` text pre-filled in a temporary file and return
/// the final contents after the editor exits.
///
/// The temp file has no particular file extension. Use [`edit_text_with_suffix`] when syntax
/// highlighting matters.
pub fn edit_text(initial: &str) -> Result<String, Error> {
  edit_text_with_suffix(initial, ".txt")
}

/// Like [`edit_text`] but uses the given `suffix` (e.g. `".md"`) for the temporary file so
/// editors can apply syntax highlighting.
pub fn edit_text_with_suffix(initial: &str, suffix: &str) -> Result<String, Error> {
  let tmp = tempfile::Builder::new().suffix(suffix).tempfile()?;

  if !initial.is_empty() {
    std::fs::write(tmp.path(), initial)?;
  }

  open_editor(tmp.path())?;
  let content = std::fs::read_to_string(tmp.path())?;
  Ok(content)
}

/// Return the editor command from `$EDITOR`, falling back to `$VISUAL`, then `vi`.
pub fn resolve_editor() -> String {
  std::env::var("EDITOR")
    .ok()
    .or_else(|| std::env::var("VISUAL").ok())
    .filter(|s| !s.is_empty())
    .unwrap_or_else(|| "vi".to_string())
}

fn open_editor(path: &Path) -> Result<(), Error> {
  let editor = resolve_editor();
  let parts =
    shell_words::split(&editor).map_err(|e| Error::Io(std::io::Error::other(format!("bad editor command: {e}"))))?;
  let (program, args) = parts.split_first().ok_or(Error::EditorNotFound)?;
  let status = Command::new(program).args(args).arg(path).status()?;

  if !status.success() {
    let code = status
      .code()
      .map(|c| c.to_string())
      .unwrap_or_else(|| "signal".to_string());
    return Err(Error::EditorFailed(editor, code));
  }

  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;

  mod edit_text {
    use temp_env::with_vars;

    use super::*;

    #[test]
    fn it_returns_initial_content_with_noop_editor() {
      with_vars([("EDITOR", Some("true")), ("VISUAL", None::<&str>)], || {
        let content = edit_text("hello world").unwrap();

        assert_eq!(content, "hello world");
      });
    }

    #[test]
    fn it_returns_empty_string_when_no_initial_content() {
      with_vars([("EDITOR", Some("true")), ("VISUAL", None::<&str>)], || {
        let content = edit_text("").unwrap();

        assert_eq!(content, "");
      });
    }
  }

  mod edit_text_with_suffix {
    use temp_env::with_vars;

    use super::*;

    #[test]
    fn it_creates_temp_file_with_suffix() {
      with_vars([("EDITOR", Some("true")), ("VISUAL", None::<&str>)], || {
        let content = edit_text_with_suffix("markdown content", ".md").unwrap();

        assert_eq!(content, "markdown content");
      });
    }

    #[test]
    fn it_returns_empty_for_empty_initial() {
      with_vars([("EDITOR", Some("true")), ("VISUAL", None::<&str>)], || {
        let content = edit_text_with_suffix("", ".rs").unwrap();

        assert_eq!(content, "");
      });
    }
  }

  mod open_editor {
    use temp_env::with_vars;

    use super::*;

    #[test]
    fn it_errors_on_nonzero_exit() {
      with_vars([("EDITOR", Some("false")), ("VISUAL", None::<&str>)], || {
        let tmp = tempfile::NamedTempFile::new().unwrap();
        let result = open_editor(tmp.path());

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, Error::EditorFailed(..)));
      });
    }

    #[test]
    fn it_succeeds_with_true_editor() {
      with_vars([("EDITOR", Some("true")), ("VISUAL", None::<&str>)], || {
        let tmp = tempfile::NamedTempFile::new().unwrap();
        let result = open_editor(tmp.path());

        assert!(result.is_ok());
      });
    }
  }

  mod resolve_editor {
    use temp_env::with_vars;

    use super::*;

    #[test]
    fn it_prefers_editor_over_visual() {
      with_vars([("EDITOR", Some("nano")), ("VISUAL", Some("code"))], || {
        assert_eq!(resolve_editor(), "nano");
      });
    }

    #[test]
    fn it_falls_back_to_visual() {
      with_vars([("EDITOR", None::<&str>), ("VISUAL", Some("code"))], || {
        assert_eq!(resolve_editor(), "code");
      });
    }

    #[test]
    fn it_falls_back_to_vi() {
      with_vars([("EDITOR", None::<&str>), ("VISUAL", None::<&str>)], || {
        assert_eq!(resolve_editor(), "vi");
      });
    }
  }
}
