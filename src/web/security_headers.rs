//! Security headers middleware.
//!
//! Adds standard security headers to all HTTP responses:
//! - `Content-Security-Policy` -- restricts resource origins
//! - `X-Content-Type-Options` -- prevents MIME-type sniffing
//! - `X-Frame-Options` -- prevents clickjacking via iframes

use axum::{extract::Request, http::HeaderValue, middleware::Next, response::Response};

/// Middleware that appends security headers to every response.
pub async fn add_security_headers(request: Request, next: Next) -> Response {
  let mut response = next.run(request).await;
  let headers = response.headers_mut();

  headers.insert(
    "content-security-policy",
    HeaderValue::from_static("default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline'; img-src 'self' https://www.gravatar.com"),
  );
  headers.insert("x-content-type-options", HeaderValue::from_static("nosniff"));
  headers.insert("x-frame-options", HeaderValue::from_static("DENY"));

  response
}
