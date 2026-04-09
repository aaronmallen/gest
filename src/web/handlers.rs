//! HTTP handlers for the web dashboard, organized by domain.

use std::fmt::Display;

use askama::Template;
use axum::{
  http::StatusCode,
  response::{Html, IntoResponse, Response},
};

pub mod api;
pub mod artifact;
pub mod avatar;
pub mod dashboard;
pub mod iteration;
pub mod search;
pub mod task;

pub use api::api_render_markdown;
pub use artifact::{
  artifact_archive, artifact_create_form, artifact_create_submit, artifact_detail, artifact_detail_fragment,
  artifact_edit_form, artifact_list, artifact_list_fragment, artifact_note_add, artifact_update,
};
pub use avatar::avatar_get;
pub use dashboard::{dashboard, dashboard_fragment};
pub use iteration::{
  iteration_board, iteration_board_fragment, iteration_detail, iteration_detail_fragment, iteration_list,
  iteration_list_fragment,
};
pub use search::{api_search, search};
pub use task::{
  note_add, task_create_form, task_create_submit, task_detail, task_detail_fragment, task_edit_form, task_list,
  task_list_fragment, task_update,
};

/// Result alias used throughout the web handlers; errors render as HTML responses.
pub(super) type Result<T> = std::result::Result<T, AppError>;

/// Typed error returned from web handlers.
///
/// Each variant maps to an HTTP status code and renders an HTML response via
/// [`IntoResponse`]. The detail carried by `Internal` is logged but never shown
/// to the user; `BadRequest` messages are surfaced directly.
#[derive(Debug)]
pub enum AppError {
  /// 400 Bad Request with a user-facing message.
  BadRequest(String),
  /// 500 Internal Server Error; the wrapped detail is logged, never rendered.
  Internal(String),
  /// 404 Not Found; renders the `not_found.html` template.
  NotFound,
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

impl From<String> for AppError {
  fn from(value: String) -> Self {
    Self::Internal(value)
  }
}

impl IntoResponse for AppError {
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
      Self::NotFound => {
        let body = NotFoundTemplate
          .render()
          .unwrap_or_else(|_| "404 — not found".to_owned());
        (StatusCode::NOT_FOUND, Html(body)).into_response()
      }
    }
  }
}

/// Fallback handler for unmatched routes.
pub async fn not_found() -> Response {
  AppError::NotFound.into_response()
}

/// Log an error from a web handler at `error` level and convert it into an
/// [`AppError::Internal`]. Use as a `.map_err` argument:
///
/// ```ignore
/// .map_err(log_err("task_detail"))?
/// ```
pub(super) fn log_err<E: Display>(context: &'static str) -> impl FnOnce(E) -> AppError {
  move |e| {
    log::error!("{context}: {e}");
    AppError::Internal(e.to_string())
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

#[cfg(test)]
mod tests {
  use super::*;

  mod app_error_into_response {
    use axum::body::to_bytes;
    use pretty_assertions::assert_eq;

    use super::*;

    async fn body_string(response: Response) -> (StatusCode, String) {
      let status = response.status();
      let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
      (status, String::from_utf8(bytes.to_vec()).unwrap())
    }

    #[tokio::test]
    async fn it_hides_internal_details_in_the_rendered_body() {
      let secret = "db connection string leaked";
      let err = AppError::Internal(secret.to_owned());

      let (status, body) = body_string(err.into_response()).await;

      assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
      assert!(!body.contains(secret));
      assert!(body.contains("Something went wrong"));
    }

    #[tokio::test]
    async fn it_renders_bad_request_with_the_user_facing_message() {
      let err = AppError::BadRequest("invalid priority".to_owned());

      let (status, body) = body_string(err.into_response()).await;

      assert_eq!(status, StatusCode::BAD_REQUEST);
      assert!(body.contains("invalid priority"));
      assert!(body.contains("<html"));
    }

    #[tokio::test]
    async fn it_renders_internal_as_an_html_error_page_with_status_500() {
      let err = AppError::Internal("boom".to_owned());

      let (status, body) = body_string(err.into_response()).await;

      assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
      assert!(body.contains("<html"));
      assert!(body.contains("500"));
    }

    #[tokio::test]
    async fn it_renders_not_found_with_the_not_found_template() {
      let err = AppError::NotFound;

      let (status, body) = body_string(err.into_response()).await;

      assert_eq!(status, StatusCode::NOT_FOUND);
      assert!(body.contains("404"));
      assert!(body.contains("<html"));
    }
  }

  mod from_string {
    use super::*;

    #[test]
    fn it_converts_a_string_into_an_internal_error() {
      let err: AppError = "oops".to_owned().into();

      assert!(matches!(err, AppError::Internal(msg) if msg == "oops"));
    }
  }
}
