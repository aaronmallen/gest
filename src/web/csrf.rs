//! CSRF protection middleware (signed-cookie double-submit pattern).
//!
//! Implements the scheme described in ADR qktwrnmm:
//!
//! 1. A random per-session token is issued in a `SameSite=Strict` cookie named
//!    [`CSRF_COOKIE`]. The cookie value is
//!    `base64url(token) . base64url(issued_at_be) . base64url(HMAC-SHA256(key, token || issued_at_be))`.
//! 2. Mutating requests (POST/PUT/DELETE) must include a `_csrf` form field
//!    whose value matches the cookie's raw `token` component; mismatch is
//!    rejected with `403 Forbidden`.
//! 3. Safe methods (GET/HEAD/OPTIONS/TRACE) are exempt from verification and
//!    still get a fresh cookie stamped if one was missing or invalid.
//! 4. Templates embed the raw token into mutating forms via the
//!    [`CSRF_TOKEN_PLACEHOLDER`] placeholder, which this middleware rewrites in
//!    `text/html` response bodies -- the same technique the CSP nonce
//!    middleware uses.
//!
//! The signing key is a 32-byte HMAC-SHA256 key sourced from the `[serve]`
//! `csrf_signing_key` config value and loaded on server start by
//! [`crate::web::serve`]. Key rotation invalidates every outstanding cookie on
//! the next request, which the middleware reissues transparently.

use std::{sync::Arc, time::SystemTime};

use axum::{
  body::{Body, to_bytes},
  extract::Request,
  http::{HeaderValue, Method, StatusCode, header},
  middleware::Next,
  response::{IntoResponse, Response},
};
use hmac::{Hmac, KeyInit, Mac};
use rand::Rng;
use sha2::Sha256;
use subtle::ConstantTimeEq;

/// Name of the cookie carrying the signed CSRF token.
pub const CSRF_COOKIE: &str = "gest_csrf";

/// Placeholder string templates use in place of the raw CSRF token.
///
/// This middleware rewrites every occurrence of the literal with the
/// per-request token before the response is returned to the client, so every
/// mutating form picks up a matching value without the handler plumbing a
/// token through the template struct.
pub const CSRF_TOKEN_PLACEHOLDER: &str = "__CSRF_TOKEN__";

/// Cookie `Max-Age` in seconds (24 hours).
const CSRF_COOKIE_MAX_AGE_SECS: u64 = 86_400;

/// Maximum age of a signed cookie accepted on verification.
const CSRF_MAX_AGE_SECS: u64 = CSRF_COOKIE_MAX_AGE_SECS;

/// Length in bytes of the random CSRF token before base64 encoding.
const CSRF_TOKEN_BYTE_LEN: usize = 32;

type HmacSha256 = Hmac<Sha256>;

/// Shared signing-key state for the CSRF middleware.
///
/// Wrapped in an [`Arc`] so the axum middleware closure can capture it by
/// clone without allocating per-request copies of the 32-byte key.
#[derive(Clone, Debug)]
pub struct CsrfKey(Arc<[u8; 32]>);

impl CsrfKey {
  /// Build a key from a 32-byte slice.
  pub fn from_bytes(bytes: [u8; 32]) -> Self {
    Self(Arc::new(bytes))
  }

  /// Parse a hex-encoded 32-byte key.
  pub fn from_hex(hex: &str) -> Result<Self, String> {
    let bytes = decode_hex(hex).ok_or_else(|| "csrf signing key must be hex".to_owned())?;
    let array: [u8; 32] = bytes
      .try_into()
      .map_err(|_| "csrf signing key must be exactly 32 bytes".to_owned())?;
    Ok(Self::from_bytes(array))
  }

  /// Generate a fresh random 32-byte key suitable for HMAC-SHA256.
  pub fn generate() -> Self {
    let mut bytes = [0u8; 32];
    rand::rng().fill_bytes(&mut bytes);
    Self::from_bytes(bytes)
  }

  /// Encode the key bytes as lowercase hex (for persistence in config.toml).
  pub fn to_hex(&self) -> String {
    encode_hex(&self.0[..])
  }

  fn as_bytes(&self) -> &[u8] {
    &self.0[..]
  }
}

/// Middleware that enforces CSRF protection on mutating requests and rewrites
/// the [`CSRF_TOKEN_PLACEHOLDER`] placeholder in HTML responses.
///
/// On every request the middleware reads the existing `gest_csrf` cookie and
/// verifies its HMAC signature; a missing, malformed, or expired cookie is
/// silently replaced with a fresh token. On mutating requests
/// (POST/PUT/DELETE/PATCH) the middleware additionally reads the submitted
/// `_csrf` form field and rejects the request with `403 Forbidden` if it does
/// not match the cookie's raw token. Safe methods (GET/HEAD/OPTIONS/TRACE)
/// skip verification but still get a refreshed cookie when needed.
pub async fn csrf_layer(key: CsrfKey, mut request: Request, next: Next) -> Response {
  let now_secs = unix_secs();
  let cookie_header = request
    .headers()
    .get(header::COOKIE)
    .and_then(|v| v.to_str().ok())
    .map(str::to_owned);
  let existing = cookie_header
    .as_deref()
    .and_then(|h| parse_csrf_cookie(h, &key, now_secs));

  let (current_token, set_cookie) = match existing {
    Some(t) => (t, None),
    None => {
      let token = generate_token();
      let cookie = build_signed_cookie(&token, now_secs, &key);
      (token, Some(cookie))
    }
  };

  if is_mutating(request.method()) {
    let (parts, body) = request.into_parts();
    let bytes = match to_bytes(body, usize::MAX).await {
      Ok(b) => b,
      Err(err) => {
        log::error!("csrf_layer: failed to buffer request body: {err}");
        return (StatusCode::INTERNAL_SERVER_ERROR, "internal error").into_response();
      }
    };
    let submitted = extract_csrf_field(&bytes);
    if !submitted.as_deref().map(|s| ct_eq(s, &current_token)).unwrap_or(false) {
      log::warn!(
        "csrf_layer: rejected {} {} -- missing or mismatched token",
        parts.method,
        parts.uri
      );
      return (StatusCode::FORBIDDEN, "csrf token missing or invalid").into_response();
    }
    request = Request::from_parts(parts, Body::from(bytes));
  }

  let response = next.run(request).await;
  let response = rewrite_html_body(response, &current_token).await;
  match set_cookie {
    Some(cookie) => attach_cookie(response, &cookie),
    None => response,
  }
}

/// Attach a `Set-Cookie` header to the response.
fn attach_cookie(mut response: Response, cookie: &str) -> Response {
  match HeaderValue::from_str(cookie) {
    Ok(value) => {
      response.headers_mut().append(header::SET_COOKIE, value);
    }
    Err(err) => log::error!("csrf_layer: failed to build Set-Cookie header: {err}"),
  }
  response
}

/// Base64-url decode (no padding) into raw bytes.
fn base64url_decode(input: &str) -> Option<Vec<u8>> {
  const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
  let mut lut = [0xffu8; 256];
  for (i, c) in ALPHABET.iter().enumerate() {
    lut[*c as usize] = i as u8;
  }
  let bytes = input.as_bytes();
  let mut out = Vec::with_capacity(bytes.len() * 3 / 4);
  let mut buf = 0u32;
  let mut bits = 0u32;
  for &b in bytes {
    let v = lut[b as usize];
    if v == 0xff {
      return None;
    }
    buf = (buf << 6) | u32::from(v);
    bits += 6;
    if bits >= 8 {
      bits -= 8;
      out.push(((buf >> bits) & 0xff) as u8);
    }
  }
  Some(out)
}

/// Base64-url encode (no padding) -- the same alphabet as [`base64url_decode`].
fn base64url_encode(input: &[u8]) -> String {
  const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
  let mut out = String::with_capacity(input.len().div_ceil(3) * 4);
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

/// Build the `Set-Cookie` header value for a freshly issued CSRF cookie.
fn build_signed_cookie(token: &str, issued_at: u64, key: &CsrfKey) -> String {
  let signed = sign_token(token, issued_at, key);
  format!("{CSRF_COOKIE}={signed}; Path=/; Max-Age={CSRF_COOKIE_MAX_AGE_SECS}; SameSite=Strict; HttpOnly",)
}

/// Constant-time string compare used for `_csrf` form-field verification.
fn ct_eq(a: &str, b: &str) -> bool {
  let lhs = a.as_bytes();
  let rhs = b.as_bytes();
  if lhs.len() != rhs.len() {
    return false;
  }
  lhs.ct_eq(rhs).into()
}

/// Decode a lowercase or mixed-case hex string into a byte vector.
fn decode_hex(s: &str) -> Option<Vec<u8>> {
  if !s.len().is_multiple_of(2) {
    return None;
  }
  let mut out = Vec::with_capacity(s.len() / 2);
  let bytes = s.as_bytes();
  for pair in bytes.chunks_exact(2) {
    let hi = hex_nibble(pair[0])?;
    let lo = hex_nibble(pair[1])?;
    out.push((hi << 4) | lo);
  }
  Some(out)
}

/// Encode bytes as lowercase hex.
fn encode_hex(bytes: &[u8]) -> String {
  const LUT: &[u8; 16] = b"0123456789abcdef";
  let mut out = String::with_capacity(bytes.len() * 2);
  for b in bytes {
    out.push(LUT[(b >> 4) as usize] as char);
    out.push(LUT[(b & 0x0f) as usize] as char);
  }
  out
}

/// Read the `_csrf` field from a `application/x-www-form-urlencoded` body.
fn extract_csrf_field(body: &[u8]) -> Option<String> {
  for (key, value) in form_urlencoded::parse(body) {
    if key == "_csrf" {
      return Some(value.into_owned());
    }
  }
  None
}

/// Generate a URL-safe base64url (no padding) random token.
fn generate_token() -> String {
  let mut bytes = [0u8; CSRF_TOKEN_BYTE_LEN];
  rand::rng().fill_bytes(&mut bytes);
  base64url_encode(&bytes)
}

/// Parse a single hex digit into its nibble value.
fn hex_nibble(b: u8) -> Option<u8> {
  match b {
    b'0'..=b'9' => Some(b - b'0'),
    b'a'..=b'f' => Some(b - b'a' + 10),
    b'A'..=b'F' => Some(b - b'A' + 10),
    _ => None,
  }
}

/// Return `true` for request methods that should go through CSRF verification.
fn is_mutating(method: &Method) -> bool {
  matches!(*method, Method::POST | Method::PUT | Method::PATCH | Method::DELETE)
}

/// Parse a `Cookie:` header, verify the CSRF cookie's HMAC, and return the
/// raw token if the cookie is fresh and genuine. Returns `None` on any
/// parse or signature failure so the caller reissues a new cookie.
fn parse_csrf_cookie(header_value: &str, key: &CsrfKey, now_secs: u64) -> Option<String> {
  let raw = header_value
    .split(';')
    .map(str::trim)
    .find_map(|pair| pair.strip_prefix(&format!("{CSRF_COOKIE}=")))?;
  let mut parts = raw.splitn(3, '.');
  let token_b64 = parts.next()?;
  let issued_b64 = parts.next()?;
  let mac_b64 = parts.next()?;
  let token_bytes = base64url_decode(token_b64)?;
  let issued_bytes = base64url_decode(issued_b64)?;
  if issued_bytes.len() != 8 {
    return None;
  }
  let mac_bytes = base64url_decode(mac_b64)?;
  let mut issued_arr = [0u8; 8];
  issued_arr.copy_from_slice(&issued_bytes);
  let issued_at = u64::from_be_bytes(issued_arr);
  if now_secs.saturating_sub(issued_at) > CSRF_MAX_AGE_SECS {
    return None;
  }

  let mut mac = HmacSha256::new_from_slice(key.as_bytes()).ok()?;
  mac.update(&token_bytes);
  mac.update(&issued_arr);
  mac.verify_slice(&mac_bytes).ok()?;

  Some(token_b64.to_owned())
}

/// Replace every occurrence of [`CSRF_TOKEN_PLACEHOLDER`] in an HTML body.
///
/// Only `text/html` responses are rewritten; binary assets and non-UTF-8 HTML
/// pass through untouched.
async fn rewrite_html_body(response: Response, token: &str) -> Response {
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
    Ok(b) => b,
    Err(err) => {
      log::error!("csrf_layer: failed to buffer response body: {err}");
      return (StatusCode::INTERNAL_SERVER_ERROR, "internal error").into_response();
    }
  };

  let body_str = match std::str::from_utf8(&bytes) {
    Ok(s) => s,
    Err(_) => return Response::from_parts(parts, Body::from(bytes)),
  };

  if !body_str.contains(CSRF_TOKEN_PLACEHOLDER) {
    return Response::from_parts(parts, Body::from(bytes));
  }

  let rewritten = body_str.replace(CSRF_TOKEN_PLACEHOLDER, token);
  parts.headers.remove(header::CONTENT_LENGTH);
  Response::from_parts(parts, Body::from(rewritten))
}

/// HMAC-sign `token || issued_at` with `key`, returning the full signed cookie
/// value as `base64(token) . base64(issued_at) . base64(mac)`.
fn sign_token(token: &str, issued_at: u64, key: &CsrfKey) -> String {
  let token_bytes = base64url_decode(token).expect("generate_token returns valid base64url");
  let issued_be = issued_at.to_be_bytes();
  let mut mac = HmacSha256::new_from_slice(key.as_bytes()).expect("HMAC accepts any key length");
  mac.update(&token_bytes);
  mac.update(&issued_be);
  let tag = mac.finalize().into_bytes();
  format!("{}.{}.{}", token, base64url_encode(&issued_be), base64url_encode(&tag),)
}

/// Seconds since the Unix epoch, saturating to 0 on clocks set before 1970.
fn unix_secs() -> u64 {
  SystemTime::now()
    .duration_since(SystemTime::UNIX_EPOCH)
    .map(|d| d.as_secs())
    .unwrap_or(0)
}

#[cfg(test)]
mod tests {
  use axum::{
    Router,
    body::{Body, to_bytes},
    http::{HeaderValue, Request as HttpRequest, StatusCode, header},
    middleware,
    response::Html,
    routing::{get, post},
  };
  use tower::ServiceExt;

  use super::*;

  fn test_router(key: CsrfKey) -> Router {
    async fn page() -> Html<&'static str> {
      Html(
        "<html><body><form method=\"post\" action=\"/submit\"><input type=\"hidden\" name=\"_csrf\" value=\"__CSRF_TOKEN__\"></form></body></html>",
      )
    }
    async fn submit() -> &'static str {
      "ok"
    }
    Router::new()
      .route("/", get(page))
      .route("/submit", post(submit))
      .layer(middleware::from_fn(move |req, next| {
        let key = key.clone();
        async move { csrf_layer(key, req, next).await }
      }))
  }

  fn extract_set_cookie(response: &Response) -> String {
    response
      .headers()
      .get(header::SET_COOKIE)
      .expect("middleware issues a cookie on GET")
      .to_str()
      .unwrap()
      .to_owned()
  }

  fn extract_token_from_form(html: &str) -> String {
    let marker = "name=\"_csrf\" value=\"";
    let start = html.find(marker).expect("form stamps the csrf token") + marker.len();
    let end = start + html[start..].find('"').unwrap();
    html[start..end].to_owned()
  }

  fn signed_value_from_set_cookie(header_value: &str) -> String {
    let prefix = format!("{CSRF_COOKIE}=");
    let rest = &header_value[prefix.len()..];
    let end = rest.find(';').unwrap_or(rest.len());
    rest[..end].to_owned()
  }

  mod base64url_encode {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_round_trips_through_base64url_decode() {
      let bytes = [0u8, 1, 2, 3, 4, 5, 6, 7, 8, 9];
      let encoded = super::super::base64url_encode(&bytes);
      let decoded = base64url_decode(&encoded).unwrap();

      assert_eq!(decoded, bytes);
    }
  }

  mod csrf_key {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_round_trips_through_hex() {
      let key = CsrfKey::generate();
      let hex = key.to_hex();

      let parsed = CsrfKey::from_hex(&hex).unwrap();

      assert_eq!(parsed.as_bytes(), key.as_bytes());
      assert_eq!(hex.len(), 64);
    }

    #[test]
    fn it_rejects_hex_of_the_wrong_length() {
      let err = CsrfKey::from_hex("deadbeef").unwrap_err();

      assert!(err.contains("32 bytes"));
    }

    #[test]
    fn it_rejects_non_hex_characters() {
      let err = CsrfKey::from_hex("zz").unwrap_err();

      assert!(err.contains("hex"));
    }
  }

  mod csrf_layer {
    use pretty_assertions::assert_eq;

    use super::*;

    async fn body_string(response: Response) -> String {
      let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
      String::from_utf8(bytes.to_vec()).unwrap()
    }

    #[tokio::test]
    async fn it_accepts_a_post_with_a_matching_token_and_cookie() {
      let key = CsrfKey::generate();
      let router = test_router(key.clone());

      let get_resp = router
        .clone()
        .oneshot(HttpRequest::builder().uri("/").body(Body::empty()).unwrap())
        .await
        .unwrap();
      let signed = signed_value_from_set_cookie(&extract_set_cookie(&get_resp));
      let mut get_resp = get_resp;
      get_resp
        .headers_mut()
        .insert(header::CONTENT_TYPE, HeaderValue::from_static("text/html"));
      let html = body_string(get_resp).await;
      let token = extract_token_from_form(&html);

      let body = format!("_csrf={token}&data=hi");
      let post_resp = router
        .oneshot(
          HttpRequest::builder()
            .method("POST")
            .uri("/submit")
            .header(header::COOKIE, format!("{CSRF_COOKIE}={signed}"))
            .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
            .body(Body::from(body))
            .unwrap(),
        )
        .await
        .unwrap();

      assert_eq!(post_resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn it_exempts_get_requests_from_verification() {
      let key = CsrfKey::generate();
      let router = test_router(key);

      let response = router
        .oneshot(HttpRequest::builder().uri("/").body(Body::empty()).unwrap())
        .await
        .unwrap();

      assert_eq!(response.status(), StatusCode::OK);
      assert!(response.headers().get(header::SET_COOKIE).is_some());
    }

    #[tokio::test]
    async fn it_rejects_a_post_with_a_mismatched_token() {
      let key = CsrfKey::generate();
      let router = test_router(key.clone());

      let get_resp = router
        .clone()
        .oneshot(HttpRequest::builder().uri("/").body(Body::empty()).unwrap())
        .await
        .unwrap();
      let signed = signed_value_from_set_cookie(&extract_set_cookie(&get_resp));

      let post_resp = router
        .oneshot(
          HttpRequest::builder()
            .method("POST")
            .uri("/submit")
            .header(header::COOKIE, format!("{CSRF_COOKIE}={signed}"))
            .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
            .body(Body::from("_csrf=not-the-real-token"))
            .unwrap(),
        )
        .await
        .unwrap();

      assert_eq!(post_resp.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn it_rejects_a_post_with_no_cookie_at_all() {
      let key = CsrfKey::generate();
      let router = test_router(key);

      let post_resp = router
        .oneshot(
          HttpRequest::builder()
            .method("POST")
            .uri("/submit")
            .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
            .body(Body::from("_csrf=anything"))
            .unwrap(),
        )
        .await
        .unwrap();

      assert_eq!(post_resp.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn it_rejects_a_post_with_no_form_field() {
      let key = CsrfKey::generate();
      let router = test_router(key.clone());

      let get_resp = router
        .clone()
        .oneshot(HttpRequest::builder().uri("/").body(Body::empty()).unwrap())
        .await
        .unwrap();
      let signed = signed_value_from_set_cookie(&extract_set_cookie(&get_resp));

      let post_resp = router
        .oneshot(
          HttpRequest::builder()
            .method("POST")
            .uri("/submit")
            .header(header::COOKIE, format!("{CSRF_COOKIE}={signed}"))
            .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
            .body(Body::from("data=without+token"))
            .unwrap(),
        )
        .await
        .unwrap();

      assert_eq!(post_resp.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn it_rewrites_the_placeholder_in_html_responses_with_the_same_token_as_the_cookie() {
      let key = CsrfKey::generate();
      let router = test_router(key);

      let response = router
        .oneshot(HttpRequest::builder().uri("/").body(Body::empty()).unwrap())
        .await
        .unwrap();
      let set_cookie = extract_set_cookie(&response);
      let mut response = response;
      response
        .headers_mut()
        .insert(header::CONTENT_TYPE, HeaderValue::from_static("text/html"));
      let html = body_string(response).await;

      assert!(!html.contains(CSRF_TOKEN_PLACEHOLDER));
      let token = extract_token_from_form(&html);
      let signed = signed_value_from_set_cookie(&set_cookie);
      assert!(signed.starts_with(&format!("{token}.")));
    }

    #[tokio::test]
    async fn it_sets_the_cookie_with_strict_same_site_and_path_root() {
      let key = CsrfKey::generate();
      let router = test_router(key);

      let response = router
        .oneshot(HttpRequest::builder().uri("/").body(Body::empty()).unwrap())
        .await
        .unwrap();

      let cookie = extract_set_cookie(&response);
      assert!(cookie.contains("SameSite=Strict"));
      assert!(cookie.contains("Path=/"));
      assert!(cookie.contains("Max-Age=86400"));
    }
  }

  mod encode_hex {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_round_trips_through_decode_hex() {
      let bytes: Vec<u8> = (0..=255u8).collect();
      let hex = super::super::encode_hex(&bytes);
      let decoded = decode_hex(&hex).unwrap();

      assert_eq!(decoded, bytes);
    }
  }

  mod parse_csrf_cookie {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_rejects_a_cookie_signed_with_a_different_key() {
      let key_a = CsrfKey::generate();
      let key_b = CsrfKey::generate();
      let token = generate_token();
      let signed = sign_token(&token, 1_000, &key_a);
      let header_value = format!("other=x; {CSRF_COOKIE}={signed}");

      let result = parse_csrf_cookie(&header_value, &key_b, 1_000);

      assert_eq!(result, None);
    }

    #[test]
    fn it_rejects_an_expired_cookie() {
      let key = CsrfKey::generate();
      let token = generate_token();
      let signed = sign_token(&token, 0, &key);
      let header_value = format!("{CSRF_COOKIE}={signed}");

      let result = parse_csrf_cookie(&header_value, &key, CSRF_MAX_AGE_SECS + 1);

      assert_eq!(result, None);
    }

    #[test]
    fn it_returns_the_raw_token_when_the_signature_verifies() {
      let key = CsrfKey::generate();
      let token = generate_token();
      let signed = sign_token(&token, 1_000, &key);
      let header_value = format!("{CSRF_COOKIE}={signed}; other=x");

      let result = parse_csrf_cookie(&header_value, &key, 1_000);

      assert_eq!(result, Some(token));
    }
  }
}
