//! Miscellaneous JSON/HTML API endpoints that are not tied to a single domain.

use axum::{
  Json,
  http::header::CONTENT_TYPE,
  response::{IntoResponse, Response},
};
use serde::Deserialize;

use crate::web::markdown;

/// Request body for the render-markdown endpoint.
#[derive(Deserialize)]
pub struct RenderMarkdownBody {
  pub body: String,
}

/// POST /api/render-markdown — render Markdown to HTML.
pub async fn api_render_markdown(Json(payload): Json<RenderMarkdownBody>) -> Response {
  let html_output = markdown::render_markdown_to_html(&payload.body);
  ([(CONTENT_TYPE, "text/html; charset=utf-8")], html_output).into_response()
}
