mod cli;
mod config;
mod event_store;
mod logger;
mod model;
mod server;
mod store;
#[cfg(test)]
mod test_helpers;
mod ui;

use ui::{composites::error_message::ErrorMessage, theming::theme::Theme};

/// Run the CLI, printing any top-level error to stderr and exiting non-zero.
fn main() {
  ui::init();

  if let Err(e) = cli::run() {
    let exit_code = e.exit_code();
    let theme = Theme::default();
    let msg = ErrorMessage::new(e.to_string(), &theme);
    let _ = std::io::Write::write_fmt(&mut std::io::stderr(), format_args!("{msg}"));
    std::process::exit(exit_code);
  }
}
