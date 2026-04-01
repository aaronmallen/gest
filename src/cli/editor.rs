use std::{path::Path, process::Command};

use crate::config::env::{EDITOR, VISUAL};

/// Open the user's editor with a temporary file, optionally pre-filled, and return the final contents.
pub fn edit_temp(initial_content: Option<&str>, suffix: &str) -> crate::cli::Result<String> {
  let tmp = tempfile::Builder::new().suffix(suffix).tempfile()?;

  if let Some(content) = initial_content {
    std::fs::write(tmp.path(), content)?;
  }

  open_editor(tmp.path())?;
  let content = std::fs::read_to_string(tmp.path())?;
  Ok(content)
}

/// Open the user's preferred editor (`$VISUAL` or `$EDITOR`) on the given path.
pub fn open_editor(path: &Path) -> crate::cli::Result<()> {
  let editor =
    resolve_editor().ok_or_else(|| crate::cli::Error::generic("No editor configured ($VISUAL or $EDITOR)"))?;
  open_editor_with(&editor, path)
}

/// Return the editor command from `$VISUAL` or `$EDITOR`, if set and non-empty.
pub fn resolve_editor() -> Option<String> {
  VISUAL
    .value()
    .ok()
    .or_else(|| EDITOR.value().ok())
    .filter(|s: &String| !s.is_empty())
}

fn open_editor_with(editor: &str, path: &Path) -> crate::cli::Result<()> {
  let parts = shell_words::split(editor)
    .map_err(|e| crate::cli::Error::generic(format!("Failed to parse editor command: {e}")))?;
  let (program, args) = parts
    .split_first()
    .ok_or_else(|| crate::cli::Error::generic("Editor command is empty"))?;
  let status = Command::new(program).args(args).arg(path).status()?;

  if !status.success() {
    return Err(crate::cli::Error::generic(format!(
      "Editor '{}' exited with {}",
      editor,
      status
        .code()
        .map(|c| c.to_string())
        .unwrap_or_else(|| "signal".to_string())
    )));
  }

  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;

  mod edit_temp {
    use temp_env::{with_var, with_vars};

    use super::*;

    #[test]
    fn it_returns_empty_string_for_empty_temp() {
      with_var("VISUAL", Some("true"), || {
        let content = edit_temp(None, ".md").unwrap();

        assert_eq!(content, "");
      });
    }

    #[test]
    fn it_returns_initial_content_with_editor_fallback() {
      with_vars([("VISUAL", None::<&str>), ("EDITOR", Some("true"))], || {
        let content = edit_temp(Some("fallback test"), ".md").unwrap();

        assert_eq!(content, "fallback test");
      });
    }

    #[test]
    fn it_returns_initial_content_with_noop_editor() {
      with_var("VISUAL", Some("true"), || {
        let content = edit_temp(Some("hello world"), ".md").unwrap();

        assert_eq!(content, "hello world");
      });
    }
  }

  mod open_editor_with {
    use super::*;

    #[test]
    fn it_errors_on_nonzero_exit() {
      let tmp = tempfile::NamedTempFile::new().unwrap();
      let result = open_editor_with("false", tmp.path());

      assert!(result.is_err());
    }

    #[test]
    fn it_succeeds_with_true_editor() {
      let tmp = tempfile::NamedTempFile::new().unwrap();
      let result = open_editor_with("true", tmp.path());

      assert!(result.is_ok());
    }
  }

  mod resolve_editor {
    use super::*;

    #[test]
    fn it_returns_none_when_neither_set() {
      let result = resolve_editor();
      if let Some(ref editor) = result {
        assert!(!editor.is_empty());
      }
    }
  }
}
