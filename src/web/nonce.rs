//! Per-request CSP nonce middleware.
//!
//! Generates a cryptographically random nonce for every request, stores it in
//! the request extensions so downstream middleware (notably
//! [`crate::web::security_headers`]) can read it, and replaces any occurrences
//! of [`NONCE_PLACEHOLDER`] in the rendered HTML body with the real value.
//!
//! Templates that contain inline `<script>` or `<style>` tags stamp the
//! placeholder literally (e.g. `<style nonce="__CSP_NONCE__">`); this
//! middleware rewrites the placeholder on the way out so the nonce in the
//! `Content-Security-Policy` header matches the one on the tag.

use axum::{
  body::{Body, to_bytes},
  extract::Request,
  http::{StatusCode, header},
  middleware::Next,
  response::{IntoResponse, Response},
};
use rand::Rng;

/// Placeholder string that templates use in place of the real nonce.
///
/// The middleware rewrites every occurrence of this literal in HTML response
/// bodies with the per-request nonce before the response is returned to the
/// client.
pub const NONCE_PLACEHOLDER: &str = "__CSP_NONCE__";

/// Length in bytes of the random nonce before base64 encoding.
///
/// 16 bytes (128 bits) is the minimum recommended by the W3C CSP spec for
/// nonce-based strict-CSP deployments.
const NONCE_BYTE_LEN: usize = 16;

/// Per-request CSP nonce stored in the request extensions.
///
/// Wrapped in a newtype so it can be extracted from extensions by type and so
/// the inner string is not accidentally confused with other request-scoped
/// strings.
#[derive(Clone, Debug)]
pub struct CspNonce(String);

impl CspNonce {
  /// Generate a fresh random nonce suitable for a single HTTP response.
  pub fn generate() -> Self {
    let mut bytes = [0u8; NONCE_BYTE_LEN];
    rand::rng().fill_bytes(&mut bytes);
    Self(base64_encode(&bytes))
  }

  /// The nonce value as rendered into the CSP header and HTML tags.
  pub fn as_str(&self) -> &str {
    &self.0
  }
}

/// Middleware that attaches a per-request nonce and rewrites the response body.
///
/// The nonce is generated before the inner handler runs so downstream middleware
/// (e.g. `security_headers`) can read it from the request extensions. After the
/// handler produces its response, the body is buffered and every occurrence of
/// [`NONCE_PLACEHOLDER`] is replaced with the real nonce value. Only
/// `text/html` responses are rewritten; all other content types pass through
/// untouched so binary assets (images, JSON, etc.) are not corrupted.
pub async fn attach_nonce(mut request: Request, next: Next) -> Response {
  let nonce = CspNonce::generate();
  request.extensions_mut().insert(nonce.clone());

  let response = next.run(request).await;
  rewrite_html_body(response, nonce.as_str()).await
}

/// Buffer the response body and replace [`NONCE_PLACEHOLDER`] with `nonce`.
///
/// Non-HTML responses and non-UTF-8 HTML bodies pass through unchanged. Any
/// error buffering the body is logged and surfaced as a plain 500 response so
/// a malformed downstream body cannot leak past this layer.
async fn rewrite_html_body(response: Response, nonce: &str) -> Response {
  let is_html = response
    .headers()
    .get(header::CONTENT_TYPE)
    .and_then(|v| v.to_str().ok())
    .map(|v| v.starts_with("text/html"))
    .unwrap_or(false);
  if !is_html {
    return response;
  }

  let (mut parts, body) = response.into_parts();
  let bytes = match to_bytes(body, usize::MAX).await {
    Ok(bytes) => bytes,
    Err(err) => {
      log::error!("rewrite_html_body: failed to buffer response body: {err}");
      return (StatusCode::INTERNAL_SERVER_ERROR, "internal error").into_response();
    }
  };

  let body_str = match std::str::from_utf8(&bytes) {
    Ok(s) => s,
    Err(_) => return Response::from_parts(parts, Body::from(bytes)),
  };

  let rewritten = body_str.replace(NONCE_PLACEHOLDER, nonce);
  // Body length changed, drop the stale Content-Length so the framework recomputes it.
  parts.headers.remove(header::CONTENT_LENGTH);
  Response::from_parts(parts, Body::from(rewritten))
}

/// Base64-encode bytes without padding using the URL-safe alphabet.
///
/// Small standalone helper to avoid pulling in a full base64 crate for a single
/// nonce-encoding use case. The output is safe to embed in both the CSP header
/// and HTML attribute values.
fn base64_encode(input: &[u8]) -> String {
  const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
  let mut out = String::with_capacity((input.len() * 4).div_ceil(3));
  let chunks = input.chunks_exact(3);
  let remainder = chunks.remainder();
  for chunk in chunks {
    let n = (u32::from(chunk[0]) << 16) | (u32::from(chunk[1]) << 8) | u32::from(chunk[2]);
    out.push(ALPHABET[((n >> 18) & 0x3F) as usize] as char);
    out.push(ALPHABET[((n >> 12) & 0x3F) as usize] as char);
    out.push(ALPHABET[((n >> 6) & 0x3F) as usize] as char);
    out.push(ALPHABET[(n & 0x3F) as usize] as char);
  }
  match remainder.len() {
    1 => {
      let n = u32::from(remainder[0]) << 16;
      out.push(ALPHABET[((n >> 18) & 0x3F) as usize] as char);
      out.push(ALPHABET[((n >> 12) & 0x3F) as usize] as char);
    }
    2 => {
      let n = (u32::from(remainder[0]) << 16) | (u32::from(remainder[1]) << 8);
      out.push(ALPHABET[((n >> 18) & 0x3F) as usize] as char);
      out.push(ALPHABET[((n >> 12) & 0x3F) as usize] as char);
      out.push(ALPHABET[((n >> 6) & 0x3F) as usize] as char);
    }
    _ => {}
  }
  out
}

#[cfg(test)]
mod tests {
  use axum::http::HeaderValue;

  use super::*;

  async fn body_string(response: Response) -> String {
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    String::from_utf8(bytes.to_vec()).unwrap()
  }

  fn html_response(body: &'static str) -> Response {
    let mut response = Response::new(Body::from(body));
    response.headers_mut().insert(
      header::CONTENT_TYPE,
      HeaderValue::from_static("text/html; charset=utf-8"),
    );
    response
  }

  mod base64_encode {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_encodes_empty_input_as_empty_string() {
      assert_eq!(base64_encode(&[]), "");
    }

    #[test]
    fn it_encodes_one_byte_to_two_characters() {
      assert_eq!(base64_encode(&[0xff]), "_w");
    }

    #[test]
    fn it_encodes_three_bytes_to_four_characters() {
      assert_eq!(base64_encode(&[0x00, 0x00, 0x00]), "AAAA");
      assert_eq!(base64_encode(&[0xff, 0xff, 0xff]), "____");
    }

    #[test]
    fn it_encodes_two_bytes_to_three_characters() {
      assert_eq!(base64_encode(&[0xff, 0x00]), "_wA");
    }
  }

  mod csp_nonce {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_generates_a_distinct_nonce_each_call() {
      let a = CspNonce::generate();
      let b = CspNonce::generate();

      assert_ne!(a.as_str(), b.as_str());
    }

    #[test]
    fn it_produces_nonces_of_the_expected_length() {
      let nonce = CspNonce::generate();

      // 16 bytes in base64url (no padding) = ceil(16 * 4 / 3) = 22 chars.
      assert_eq!(nonce.as_str().len(), 22);
    }
  }

  mod rewrite_html_body {
    use pretty_assertions::assert_eq;

    use super::*;

    #[tokio::test]
    async fn it_drops_the_stale_content_length_after_rewriting() {
      let mut response = html_response("<style nonce=\"__CSP_NONCE__\"></style>");
      response
        .headers_mut()
        .insert(header::CONTENT_LENGTH, HeaderValue::from_static("999"));

      let out = super::super::rewrite_html_body(response, "abcd").await;

      assert!(out.headers().get(header::CONTENT_LENGTH).is_none());
    }

    #[tokio::test]
    async fn it_leaves_a_non_utf8_html_body_untouched() {
      let mut response = Response::new(Body::from(vec![0xff, 0xfe, 0xfd]));
      response
        .headers_mut()
        .insert(header::CONTENT_TYPE, HeaderValue::from_static("text/html"));

      let out = super::super::rewrite_html_body(response, "abcd").await;
      let bytes = to_bytes(out.into_body(), usize::MAX).await.unwrap();

      assert_eq!(bytes.as_ref(), &[0xff, 0xfe, 0xfd]);
    }

    #[tokio::test]
    async fn it_passes_non_html_responses_through_unchanged() {
      let mut response = Response::new(Body::from("{\"value\":\"__CSP_NONCE__\"}"));
      response
        .headers_mut()
        .insert(header::CONTENT_TYPE, HeaderValue::from_static("application/json"));

      let out = super::super::rewrite_html_body(response, "abcd").await;
      let body = body_string(out).await;

      assert!(body.contains("__CSP_NONCE__"));
    }

    #[tokio::test]
    async fn it_replaces_every_occurrence_of_the_placeholder_in_an_html_body() {
      let response =
        html_response("<style nonce=\"__CSP_NONCE__\"></style><script nonce=\"__CSP_NONCE__\">1;</script>");

      let out = super::super::rewrite_html_body(response, "xYz123").await;
      assert_eq!(out.status(), StatusCode::OK);
      let body = body_string(out).await;

      assert!(!body.contains("__CSP_NONCE__"));
      assert_eq!(body.matches("nonce=\"xYz123\"").count(), 2);
    }
  }
}
