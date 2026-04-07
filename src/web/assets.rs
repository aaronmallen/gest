//! Embedded static assets served via rust-embed.

use axum::{
  extract::Path,
  http::{StatusCode, header},
  response::{IntoResponse, Response},
};
use rust_embed::Embed;

#[derive(Embed)]
#[folder = "templates/"]
#[include = "*.css"]
#[include = "*.js"]
struct StaticAssets;

/// Serve an embedded static asset by path.
pub async fn serve(Path(path): Path<String>) -> Response {
  match StaticAssets::get(&path) {
    Some(file) => {
      let mime = mime_for(&path);
      ([(header::CONTENT_TYPE, mime)], file.data).into_response()
    }
    None => StatusCode::NOT_FOUND.into_response(),
  }
}

/// Infer a MIME type from the file extension.
fn mime_for(path: &str) -> &'static str {
  if path.ends_with(".css") {
    "text/css; charset=utf-8"
  } else if path.ends_with(".js") {
    "application/javascript; charset=utf-8"
  } else {
    "application/octet-stream"
  }
}
