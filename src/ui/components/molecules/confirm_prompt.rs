//! Interactive Yes/No confirmation prompt using crossterm for raw terminal input.

use std::io::{self, BufRead, IsTerminal, Write};

use crossterm::{
  event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
  terminal,
};
use yansi::Paint;

use crate::ui::style;

/// The user's selection in the confirm prompt.
#[derive(Clone, Copy, Eq, PartialEq)]
enum Selection {
  No,
  Yes,
}

/// Renders and drives an interactive Yes/No selector.
///
/// Arrow keys, `h`/`l`, and `j`/`k` toggle between options.
/// `y`/`n` act as direct shortcuts. Escape and Ctrl+C decline.
/// The default selection is No.
pub struct Component {
  description: String,
}

impl Component {
  /// Create a confirm prompt with the given description text.
  pub fn new(description: impl Into<String>) -> Self {
    Self {
      description: description.into(),
    }
  }

  /// Run the interactive prompt, returning `true` for Yes and `false` for No.
  ///
  /// When stdin is not a TTY (e.g. piped input), falls back to a simple
  /// line-based `[y/N]` prompt for scriptability.
  pub fn confirm(&self) -> io::Result<bool> {
    if !io::stdin().is_terminal() {
      return self.confirm_piped();
    }

    let mut stdout = io::stdout();
    terminal::enable_raw_mode()?;
    let result = self.run_loop(&mut stdout);
    terminal::disable_raw_mode()?;
    // Move to a new line after the prompt
    writeln!(stdout)?;
    result
  }

  /// Fallback for non-TTY stdin: print a `[y/N]` prompt and read one line.
  fn confirm_piped(&self) -> io::Result<bool> {
    let mut stdout = io::stdout();
    write!(stdout, "  {} [y/N] ", self.description)?;
    stdout.flush()?;

    let mut line = String::new();
    if io::stdin().lock().read_line(&mut line)? == 0 {
      return Ok(false);
    }

    let answer = line.trim().to_ascii_lowercase();
    Ok(matches!(answer.as_str(), "y" | "yes"))
  }

  /// Render the current state of the selector.
  fn render<W: Write>(&self, writer: &mut W, selection: Selection, first: bool) -> io::Result<()> {
    let theme = style::global();

    if first {
      // Description on its own line
      writeln!(writer, "  {}", self.description.paint(*theme.confirm_description()))?;
    }

    // Clear the button line and rewrite
    write!(writer, "\r\x1b[2K")?;

    let (yes_label, no_label) = match selection {
      Selection::Yes => (
        format!(" › {} ", "Yes".paint(*theme.confirm_selected())),
        format!("   {}  ", "No".paint(*theme.confirm_unselected())),
      ),
      Selection::No => (
        format!("   {}  ", "Yes".paint(*theme.confirm_unselected())),
        format!(" › {} ", "No".paint(*theme.confirm_selected())),
      ),
    };

    write!(writer, "  {yes_label} {no_label}")?;

    writer.flush()
  }

  /// Main event loop reading key events.
  fn run_loop<W: Write>(&self, writer: &mut W) -> io::Result<bool> {
    let mut selection = Selection::No;
    self.render(writer, selection, true)?;

    loop {
      if let Event::Key(KeyEvent {
        code,
        modifiers,
        kind: KeyEventKind::Press,
        ..
      }) = crossterm::event::read()?
      {
        match (code, modifiers) {
          // Ctrl+C or Escape => decline
          (KeyCode::Char('c'), KeyModifiers::CONTROL) | (KeyCode::Esc, _) => {
            return Ok(false);
          }
          // Enter => confirm current selection
          (KeyCode::Enter, _) => {
            return Ok(selection == Selection::Yes);
          }
          // Direct shortcuts
          (KeyCode::Char('y'), KeyModifiers::NONE) => {
            selection = Selection::Yes;
            self.render(writer, selection, false)?;
            return Ok(true);
          }
          (KeyCode::Char('n'), KeyModifiers::NONE) => {
            selection = Selection::No;
            self.render(writer, selection, false)?;
            return Ok(false);
          }
          // Navigation: left/h/k => Yes, right/l/j => No
          (KeyCode::Left | KeyCode::Char('h') | KeyCode::Char('k'), _) => {
            selection = Selection::Yes;
            self.render(writer, selection, false)?;
          }
          (KeyCode::Right | KeyCode::Char('l') | KeyCode::Char('j'), _) => {
            selection = Selection::No;
            self.render(writer, selection, false)?;
          }
          _ => {}
        }
      }
    }
  }
}
