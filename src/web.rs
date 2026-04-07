//! Web server for the gest dashboard.

mod assets;
mod forms;
mod gravatar;
mod handlers;
mod markdown;
mod note_display;
mod request_log;
mod security_headers;
mod sse;
mod state;

use std::{
  io::Error as IoError,
  net::SocketAddr,
  path::PathBuf,
  sync::Arc,
  time::{Duration, Instant},
};

use axum::{Router, middleware};
use notify::{Error as NotifyError, Event as NotifyEvent, RecursiveMode, Watcher};
pub use state::AppState;

use crate::{io::git, store::Db};

/// Start the web server on the given address.
///
/// When `gest_dir` is provided, a file watcher monitors the `.gest/`
/// directory and sends SSE reload pings on changes (debounced).
pub async fn serve(
  store: Arc<Db>,
  project_id: crate::store::model::primitives::Id,
  addr: SocketAddr,
  gest_dir: Option<PathBuf>,
  debounce_ms: u64,
) -> Result<(), Error> {
  let mut state = AppState::new(store, project_id);

  // Resolve the git author once at startup so note creation can tag the user.
  if let Some(ga) = git::resolve_author_or_env() {
    let conn = state
      .store()
      .connect()
      .await
      .map_err(|e| Error::Io(IoError::other(e.to_string())))?;
    if let Ok(author) = crate::store::repo::author::find_or_create(
      &conn,
      &ga.name,
      ga.email.as_deref(),
      crate::store::model::primitives::AuthorType::Human,
    )
    .await
    {
      state = state.with_author_id(author.id().clone());
    }
  }

  // Start file watcher for .gest/ directory
  let _watcher = if let Some(dir) = gest_dir {
    let reload_tx = state.reload_tx().clone();
    let debounce = Duration::from_millis(debounce_ms);
    let mut last_send = Instant::now() - debounce;

    let mut watcher = notify::recommended_watcher(move |res: Result<NotifyEvent, NotifyError>| {
      if res.is_ok() {
        let now = Instant::now();
        if now.duration_since(last_send) >= debounce {
          last_send = now;
          let _ = reload_tx.send(());
        }
      }
    })?;

    watcher.watch(&dir, RecursiveMode::Recursive)?;
    log::info!("watching {} for changes", dir.display());
    Some(watcher)
  } else {
    None
  };

  let app = router(state);

  log::info!("starting web server at http://{addr}");
  let listener = tokio::net::TcpListener::bind(addr).await?;
  axum::serve(listener, app).await?;

  Ok(())
}

/// Build the application router with all routes.
fn router(state: AppState) -> Router {
  Router::new()
    .route("/", axum::routing::get(handlers::dashboard))
    .route("/_dashboard", axum::routing::get(handlers::dashboard_fragment))
    // Artifact routes
    .route(
      "/artifacts",
      axum::routing::get(handlers::artifact_list).post(handlers::artifact_create_submit),
    )
    .route("/artifacts/_list", axum::routing::get(handlers::artifact_list_fragment))
    .route("/artifacts/new", axum::routing::get(handlers::artifact_create_form))
    .route(
      "/artifacts/{id}",
      axum::routing::get(handlers::artifact_detail).post(handlers::artifact_update),
    )
    .route(
      "/artifacts/{id}/_detail",
      axum::routing::get(handlers::artifact_detail_fragment),
    )
    .route(
      "/artifacts/{id}/archive",
      axum::routing::post(handlers::artifact_archive),
    )
    .route("/artifacts/{id}/edit", axum::routing::get(handlers::artifact_edit_form))
    .route(
      "/artifacts/{id}/notes",
      axum::routing::post(handlers::artifact_note_add),
    )
    // SSE + API
    .route("/events", axum::routing::get(sse::events))
    .route("/api/search", axum::routing::get(handlers::api_search))
    .route(
      "/api/render-markdown",
      axum::routing::post(handlers::api_render_markdown),
    )
    // Iteration routes
    .route("/iterations", axum::routing::get(handlers::iteration_list))
    .route(
      "/iterations/_list",
      axum::routing::get(handlers::iteration_list_fragment),
    )
    .route("/iterations/{id}", axum::routing::get(handlers::iteration_detail))
    .route("/iterations/{id}/board", axum::routing::get(handlers::iteration_board))
    .route(
      "/iterations/{id}/_detail",
      axum::routing::get(handlers::iteration_detail_fragment),
    )
    .route(
      "/iterations/{id}/_board",
      axum::routing::get(handlers::iteration_board_fragment),
    )
    // Search
    .route("/search", axum::routing::get(handlers::search))
    // Task routes
    .route(
      "/tasks",
      axum::routing::get(handlers::task_list).post(handlers::task_create_submit),
    )
    .route("/tasks/_list", axum::routing::get(handlers::task_list_fragment))
    .route("/tasks/new", axum::routing::get(handlers::task_create_form))
    .route(
      "/tasks/{id}",
      axum::routing::get(handlers::task_detail).post(handlers::task_update),
    )
    .route(
      "/tasks/{id}/_detail",
      axum::routing::get(handlers::task_detail_fragment),
    )
    .route("/tasks/{id}/edit", axum::routing::get(handlers::task_edit_form))
    .route("/tasks/{id}/notes", axum::routing::post(handlers::note_add))
    // Static assets
    .route("/static/{*path}", axum::routing::get(assets::serve))
    .fallback(handlers::not_found)
    .layer(middleware::from_fn(request_log::log_request))
    .layer(middleware::from_fn(security_headers::add_security_headers))
    .with_state(state)
}

/// Errors that can occur in the web server.
#[derive(Debug, thiserror::Error)]
pub enum Error {
  /// An I/O error.
  #[error(transparent)]
  Io(#[from] IoError),
  /// File watcher error.
  #[error(transparent)]
  Notify(#[from] NotifyError),
}
