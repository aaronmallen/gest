mod cli;
mod config;
mod io;
mod logging;
mod store;
mod ui;
mod web;

use std::{fmt::Display, path::PathBuf, sync::Arc};

use clap::Parser;
use getset::Getters;

use crate::{
  cli::App,
  config::Settings,
  logging::LevelFilter,
  store::{Db, model::primitives::Id},
  ui::{components::ErrorMessage, style, style::Theme},
};

/// Shared application state threaded through every subcommand.
#[derive(Clone, Debug, Getters)]
pub struct AppContext {
  /// The `.gest` directory path, if the project is in local mode.
  #[get = "pub"]
  gest_dir: Option<PathBuf>,
  /// The current project's ID, if one has been initialized for the working directory.
  #[get = "pub"]
  project_id: Option<Id>,
  /// Resolved configuration settings (file + env overrides).
  #[get = "pub"]
  settings: Settings,
  /// Database connection handle.
  #[get = "pub"]
  store: Arc<Db>,
}

#[tokio::main]
async fn main() {
  ui::init();

  let app = App::parse();
  let verbosity = match app.verbosity_level() {
    1 => Some(LevelFilter::Info),
    2 => Some(LevelFilter::Debug),
    3 => Some(LevelFilter::Trace),
    _ => None,
  };

  logging::init(verbosity.unwrap_or(LevelFilter::default()));

  let settings = config::load().unwrap_or_else(die);
  if verbosity.is_none() {
    log::set_max_level(settings.log().level().into());
  }

  style::set_global(Theme::from_config(&settings));

  let store = store::open(&settings).await.unwrap_or_else(die);
  let (project_id, gest_dir) = resolve_project(&store).await;

  // Configure transparent sync in the store layer
  if settings.storage().sync_enabled()
    && let (Some(pid), Some(dir)) = (&project_id, &gest_dir)
  {
    store.configure_sync(pid.clone(), dir.clone());
  }

  store.import_if_needed().await.unwrap_or_else(die);

  let context = AppContext {
    gest_dir: gest_dir.clone(),
    project_id,
    settings,
    store: store.clone(),
  };

  if let Err(e) = app.call(&context).await {
    die::<()>(e);
  }

  store.export_if_needed().await.unwrap_or_else(die);
}

/// Print an error message to stderr and exit with code 1.
fn die<T>(error: impl Display) -> T {
  eprintln!("{}", ErrorMessage::new(error.to_string()));
  std::process::exit(1);
}

/// Resolve the current project ID and `.gest` directory from the working directory.
async fn resolve_project(store: &Arc<Db>) -> (Option<Id>, Option<PathBuf>) {
  let Ok(cwd) = std::env::current_dir() else {
    return (None, None);
  };
  let conn = store.connect().await.unwrap_or_else(die);
  match store::repo::project::find_by_path(&conn, &cwd).await {
    Ok(Some(project)) => {
      let gest_dir = store::sync::find_gest_dir(project.root());
      (Some(project.id().clone()), gest_dir)
    }
    Ok(None) => (None, None),
    Err(e) => die(e),
  }
}
