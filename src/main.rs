mod cli;
mod config;
mod error;
mod logger;
mod model;
mod store;
#[cfg(test)]
mod test_helpers;
mod ui;

pub use error::{Error, Result};

fn main() {
  ui::init();

  if let Err(e) = cli::run() {
    let theme = ui::theme::Theme::default();
    let _ = ui::components::ErrorMessage::new(&e.to_string()).write_to(&mut std::io::stderr(), &theme);
    std::process::exit(1);
  }
}
