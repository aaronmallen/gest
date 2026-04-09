//! Web server for the gest dashboard.

mod assets;
mod csrf;
mod forms;
mod gravatar;
mod handlers;
mod markdown;
mod nonce;
mod reload_ipc;
mod request_log;
mod security_headers;
mod sse;
mod state;
mod timeline;

use std::{
  io::Error as IoError,
  net::SocketAddr,
  path::{Path, PathBuf},
  sync::Arc,
  time::{Duration, Instant},
};

use askama::{Error as AskamaError, Template};
use axum::{
  Router,
  http::StatusCode,
  middleware,
  response::{Html, IntoResponse, Response},
};
pub use csrf::CsrfKey;
use notify::{Error as NotifyError, Event as NotifyEvent, RecursiveMode, Watcher};
use serde_json::Error as SerdeJsonError;
pub use state::AppState;
use thiserror::Error as ThisError;

use crate::{
  io::git,
  store::{Db, Error as StoreError, avatar_cache::AvatarCache},
};

/// Errors produced by the web layer.
///
/// A single `Error` covers both the startup path (file watcher, socket bind, TCP
/// listener) and the request path (handler responses). The [`IntoResponse`] impl
/// maps variants to real HTTP status codes so handlers can return typed errors
/// instead of stringified 500s. Handler call sites still use
/// [`crate::web::handlers::AppError`] until phase 6 of the web refactor.
///
/// `BadRequest` and `Internal` carry user-facing messages that the `IntoResponse`
/// impl renders through the shared error template. `NotFound` renders the
/// dedicated not-found template. `Io` and `Notify` appear during server startup
/// (socket binding, file watcher) and render as opaque 500s if they ever surface
/// through a handler.
#[derive(Debug, ThisError)]
#[allow(dead_code)] // handler variants are wired up in phase 6 of the web refactor
pub enum Error {
  /// 400 Bad Request with a user-facing message.
  #[error("{0}")]
  BadRequest(String),
  /// 500 Internal Server Error; the wrapped detail is logged, never rendered.
  #[error("{0}")]
  Internal(String),
  /// Filesystem or network I/O error (server bootstrap or request handling).
  #[error(transparent)]
  Io(#[from] IoError),
  /// 404 Not Found; renders the `not_found.html` template.
  #[error("not found")]
  NotFound,
  /// File watcher error from the `notify` crate (server bootstrap).
  #[error(transparent)]
  Notify(#[from] NotifyError),
}

#[derive(Template)]
#[template(path = "error.html")]
struct ErrorTemplate {
  message: String,
  status: u16,
}

#[derive(Template)]
#[template(path = "not_found.html")]
struct NotFoundTemplate;

impl From<AskamaError> for Error {
  fn from(value: AskamaError) -> Self {
    Self::Internal(value.to_string())
  }
}

impl From<SerdeJsonError> for Error {
  fn from(value: SerdeJsonError) -> Self {
    Self::Internal(value.to_string())
  }
}

impl From<StoreError> for Error {
  fn from(value: StoreError) -> Self {
    match value {
      StoreError::NotFound(_) => Self::NotFound,
      other => Self::Internal(other.to_string()),
    }
  }
}

impl IntoResponse for Error {
  fn into_response(self) -> Response {
    match self {
      Self::BadRequest(message) => render_error(StatusCode::BAD_REQUEST, message),
      Self::Internal(detail) => {
        log::error!("internal error: {detail}");
        render_error(
          StatusCode::INTERNAL_SERVER_ERROR,
          "Something went wrong. Please try again.".to_owned(),
        )
      }
      Self::Io(err) => {
        log::error!("io error: {err}");
        render_error(
          StatusCode::INTERNAL_SERVER_ERROR,
          "Something went wrong. Please try again.".to_owned(),
        )
      }
      Self::NotFound => {
        let body = NotFoundTemplate
          .render()
          .unwrap_or_else(|_| "404 — not found".to_owned());
        (StatusCode::NOT_FOUND, Html(body)).into_response()
      }
      Self::Notify(err) => {
        log::error!("notify error: {err}");
        render_error(
          StatusCode::INTERNAL_SERVER_ERROR,
          "Something went wrong. Please try again.".to_owned(),
        )
      }
    }
  }
}

/// Render the shared error template for a given status and user-facing message.
fn render_error(status: StatusCode, message: String) -> Response {
  let tmpl = ErrorTemplate {
    message,
    status: status.as_u16(),
  };
  let body = tmpl.render().unwrap_or_else(|_| format!("{} — error", status.as_u16()));
  (status, Html(body)).into_response()
}

/// File name of the unix domain socket used for the web reload IPC channel.
pub const RELOAD_SOCKET_FILE: &str = "web.sock";

/// Resolve the unix domain socket path used by the web server's reload listener.
///
/// When `gest_dir` is provided (the project is in local mode with a `.gest/` directory)
/// the socket lives at `<gest_dir>/web.sock`. Otherwise it lives next to the SQLite
/// database file at `<data_dir>/web.sock`. Both the server-side listener and the
/// CLI-side notifier resolve the same path through this helper.
pub fn reload_socket_path(gest_dir: Option<&Path>, data_dir: &Path) -> PathBuf {
  match gest_dir {
    Some(dir) => dir.join(RELOAD_SOCKET_FILE),
    None => data_dir.join(RELOAD_SOCKET_FILE),
  }
}

/// Start the web server on the given address.
///
/// When `gest_dir` is provided, a file watcher monitors the `.gest/`
/// directory and sends SSE reload pings on changes (debounced).
///
/// When `socket_path` is provided (and the platform is unix), a unix domain socket
/// listener is bound there and forwards bare connection signals to the SSE reload
/// channel — letting CLI mutations wake browser tabs without polling the filesystem.
#[allow(clippy::too_many_arguments)]
pub async fn serve(
  store: Arc<Db>,
  project_id: crate::store::model::primitives::Id,
  addr: SocketAddr,
  gest_dir: Option<PathBuf>,
  socket_path: Option<PathBuf>,
  debounce_ms: u64,
  csrf_key: CsrfKey,
  cache_dir: PathBuf,
) -> Result<(), Error> {
  let avatar_cache = Arc::new(AvatarCache::new(cache_dir));
  let mut state = AppState::new(store, project_id).with_avatar_cache(avatar_cache);

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

  // Bind the unix socket reload listener (no-op on non-unix targets).
  #[cfg(unix)]
  let _reload_socket_guard = match socket_path {
    Some(path) => match reload_ipc::bind_reload_socket(&path) {
      Ok(listener) => {
        log::info!("listening for reload signals at {}", path.display());
        reload_ipc::spawn_reload_listener(listener, state.reload_tx().clone());
        Some(reload_ipc::ReloadSocketGuard::new(path))
      }
      Err(err) => {
        log::warn!("failed to bind reload socket at {}: {err}", path.display());
        None
      }
    },
    None => None,
  };
  #[cfg(not(unix))]
  let _ = socket_path;

  let app = router(state, csrf_key);

  log::info!("starting web server at http://{addr}");
  let listener = tokio::net::TcpListener::bind(addr).await?;
  axum::serve(listener, app).await?;

  Ok(())
}

/// Build the application router with all routes.
fn router(state: AppState, csrf_key: CsrfKey) -> Router {
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
    // Local avatar proxy (first-party replacement for gravatar.com)
    .route("/avatars/{hash}", axum::routing::get(handlers::avatar_get))
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
    .layer(middleware::from_fn(nonce::attach_nonce))
    .layer(middleware::from_fn(move |req, next| {
      let key = csrf_key.clone();
      async move { csrf::csrf_layer(key, req, next).await }
    }))
    .with_state(state)
}

#[cfg(test)]
mod tests {
  use axum::body::to_bytes;

  use super::*;

  async fn body_string(response: Response) -> (StatusCode, String) {
    let status = response.status();
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    (status, String::from_utf8(bytes.to_vec()).unwrap())
  }

  mod from_askama_error {
    use super::*;

    #[test]
    fn it_converts_a_render_error_into_an_internal_error() {
      let askama_err = AskamaError::Fmt;
      let err: Error = askama_err.into();

      assert!(matches!(err, Error::Internal(_)));
    }
  }

  mod from_serde_json_error {
    use super::*;

    #[test]
    fn it_converts_a_json_error_into_an_internal_error() {
      let json_err = serde_json::from_str::<serde_json::Value>("{not json}").unwrap_err();
      let err: Error = json_err.into();

      assert!(matches!(err, Error::Internal(_)));
    }
  }

  mod from_store_error {
    use super::*;

    #[test]
    fn it_maps_any_other_store_error_to_internal() {
      let store_err = StoreError::InvalidPrefix("bad".to_owned());
      let err: Error = store_err.into();

      assert!(matches!(err, Error::Internal(_)));
    }

    #[test]
    fn it_maps_store_not_found_to_not_found() {
      let store_err = StoreError::NotFound("task xyz".to_owned());
      let err: Error = store_err.into();

      assert!(matches!(err, Error::NotFound));
    }
  }

  mod into_response {
    use pretty_assertions::assert_eq;

    use super::*;

    #[tokio::test]
    async fn it_hides_internal_details_in_the_rendered_body() {
      let secret = "db connection string leaked";
      let err = Error::Internal(secret.to_owned());

      let (status, body) = body_string(err.into_response()).await;

      assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
      assert!(!body.contains(secret));
      assert!(body.contains("Something went wrong"));
    }

    #[tokio::test]
    async fn it_renders_bad_request_with_a_400_status() {
      let err = Error::BadRequest("invalid priority".to_owned());

      let (status, body) = body_string(err.into_response()).await;

      assert_eq!(status, StatusCode::BAD_REQUEST);
      assert!(body.contains("invalid priority"));
      assert!(body.contains("<html"));
    }

    #[tokio::test]
    async fn it_renders_internal_as_a_500_html_error_page() {
      let err = Error::Internal("boom".to_owned());

      let (status, body) = body_string(err.into_response()).await;

      assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
      assert!(body.contains("<html"));
      assert!(body.contains("500"));
    }

    #[tokio::test]
    async fn it_renders_io_as_a_500_html_error_page() {
      let err = Error::Io(IoError::other("disk full"));

      let (status, body) = body_string(err.into_response()).await;

      assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
      assert!(!body.contains("disk full"));
      assert!(body.contains("<html"));
      assert!(body.contains("500"));
    }

    #[tokio::test]
    async fn it_renders_not_found_with_a_404_status() {
      let err = Error::NotFound;

      let (status, body) = body_string(err.into_response()).await;

      assert_eq!(status, StatusCode::NOT_FOUND);
      assert!(body.contains("404"));
      assert!(body.contains("<html"));
    }

    #[tokio::test]
    async fn it_renders_notify_as_a_500_html_error_page() {
      let notify_err = NotifyError::generic("watcher exploded");
      let err = Error::Notify(notify_err);

      let (status, body) = body_string(err.into_response()).await;

      assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
      assert!(!body.contains("watcher exploded"));
      assert!(body.contains("<html"));
      assert!(body.contains("500"));
    }
  }

  mod middleware_chain {
    use axum::{Router, body::Body, http::Request as HttpRequest, routing::get};
    use pretty_assertions::assert_eq;
    use tower::ServiceExt;

    use super::*;

    fn test_router() -> Router {
      async fn page() -> Html<&'static str> {
        Html(concat!(
          "<html><head><style nonce=\"__CSP_NONCE__\">x{}</style></head>",
          "<body><script nonce=\"__CSP_NONCE__\">1;</script></body></html>",
        ))
      }
      Router::new()
        .route("/", get(page))
        .layer(middleware::from_fn(security_headers::add_security_headers))
        .layer(middleware::from_fn(nonce::attach_nonce))
    }

    #[tokio::test]
    async fn it_emits_a_csp_header_that_never_contains_unsafe_inline() {
      let response = test_router()
        .oneshot(HttpRequest::builder().uri("/").body(Body::empty()).unwrap())
        .await
        .unwrap();

      let csp = response
        .headers()
        .get("content-security-policy")
        .unwrap()
        .to_str()
        .unwrap()
        .to_owned();

      assert!(!csp.contains("'unsafe-inline'"));
      assert!(csp.contains("default-src 'self'"));
      assert!(csp.contains("script-src 'self' 'nonce-"));
      assert!(csp.contains("style-src 'self' 'nonce-"));
    }

    #[tokio::test]
    async fn it_sets_the_permissions_and_referrer_policy_headers() {
      let response = test_router()
        .oneshot(HttpRequest::builder().uri("/").body(Body::empty()).unwrap())
        .await
        .unwrap();

      let referrer = response.headers().get("referrer-policy").unwrap().to_str().unwrap();
      let permissions = response.headers().get("permissions-policy").unwrap().to_str().unwrap();

      assert_eq!(referrer, "no-referrer");
      assert!(permissions.contains("camera=()"));
      assert!(permissions.contains("microphone=()"));
      assert!(permissions.contains("geolocation=()"));
    }

    #[tokio::test]
    async fn it_stamps_the_same_nonce_in_the_csp_header_and_the_response_body() {
      let response = test_router()
        .oneshot(HttpRequest::builder().uri("/").body(Body::empty()).unwrap())
        .await
        .unwrap();

      let csp = response
        .headers()
        .get("content-security-policy")
        .unwrap()
        .to_str()
        .unwrap()
        .to_owned();
      let (_, body) = body_string(response).await;

      // Extract the nonce value from the CSP header.
      let marker = "'nonce-";
      let start = csp.find(marker).unwrap() + marker.len();
      let end = start + csp[start..].find('\'').unwrap();
      let nonce = &csp[start..end];

      assert!(!nonce.is_empty());
      assert!(!body.contains("__CSP_NONCE__"));
      assert_eq!(body.matches(&format!("nonce=\"{nonce}\"")).count(), 2);
    }
  }
}
