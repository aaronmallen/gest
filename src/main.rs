mod cli;
mod config;
mod logger;
mod model;
mod server;
mod store;
#[cfg(test)]
mod test_helpers;
mod ui;

/// Run the CLI, printing any top-level error to stderr and exiting non-zero.
fn main() {
  ui::init();

  if let Err(e) = cli::run() {
    let exit_code = e.exit_code();
    let theme = ui::theme::Theme::default();
    let msg = ui::composites::error_message::ErrorMessage::new(e.to_string(), &theme);
    let _ = std::io::Write::write_fmt(&mut std::io::stderr(), format_args!("{msg}"));
    std::process::exit(exit_code);
  }
}
