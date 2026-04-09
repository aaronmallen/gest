//! Security headers middleware.
//!
//! Adds standard security headers to every HTTP response:
//! - `Content-Security-Policy` -- restricts resource origins and uses a
//!   per-request nonce for inline `<script>` and `<style>` tags instead of
//!   the much weaker `'unsafe-inline'`.
//! - `Permissions-Policy` -- denies access to browser capabilities the
//!   dashboard does not use (camera, microphone, geolocation, etc.).
//! - `Referrer-Policy: no-referrer` -- prevents leaking URLs to third parties
//!   when the user follows an outbound link.
//! - `X-Content-Type-Options: nosniff` -- disables MIME-type sniffing.
//! - `X-Frame-Options: DENY` -- blocks framing to defeat clickjacking.
//!
//! The nonce is produced by [`crate::web::nonce::attach_nonce`] and read from
//! the request extensions before the inner handler runs, so it is guaranteed to
//! match the nonce the nonce middleware later stamps into the HTML body.

use axum::{extract::Request, http::HeaderValue, middleware::Next, response::Response};

use crate::web::nonce::CspNonce;

/// Permissions-Policy value denying every capability the dashboard does not use.
///
/// Browsers treat an empty allowlist (`()`) as "no origin can use this
/// feature", which is what we want for a purely read/edit oriented dashboard.
const PERMISSIONS_POLICY: &str = concat!(
  "accelerometer=(), ",
  "camera=(), ",
  "clipboard-read=(), ",
  "clipboard-write=(), ",
  "display-capture=(), ",
  "fullscreen=(), ",
  "geolocation=(), ",
  "gyroscope=(), ",
  "hid=(), ",
  "interest-cohort=(), ",
  "magnetometer=(), ",
  "microphone=(), ",
  "midi=(), ",
  "payment=(), ",
  "publickey-credentials-get=(), ",
  "screen-wake-lock=(), ",
  "serial=(), ",
  "usb=(), ",
  "xr-spatial-tracking=()",
);

/// Middleware that appends security headers to every response.
///
/// The `Content-Security-Policy` is built per request so it can reference the
/// nonce stored in the request extensions by [`crate::web::nonce::attach_nonce`].
/// If the nonce middleware is missing the CSP falls back to `'self'` only,
/// which is safer than silently accepting inline content.
pub async fn add_security_headers(request: Request, next: Next) -> Response {
  let nonce = request.extensions().get::<CspNonce>().cloned();
  let csp = build_csp_header(nonce.as_ref().map(CspNonce::as_str));

  let mut response = next.run(request).await;
  let headers = response.headers_mut();

  if let Ok(value) = HeaderValue::from_str(&csp) {
    headers.insert("content-security-policy", value);
  }
  headers.insert("permissions-policy", HeaderValue::from_static(PERMISSIONS_POLICY));
  headers.insert("referrer-policy", HeaderValue::from_static("no-referrer"));
  headers.insert("x-content-type-options", HeaderValue::from_static("nosniff"));
  headers.insert("x-frame-options", HeaderValue::from_static("DENY"));

  response
}

/// Build the `Content-Security-Policy` header value for a request.
///
/// When a per-request nonce is available, inline `<script>` and `<style>` tags
/// are authorized via `'nonce-<value>'`. When it is not, the policy omits
/// inline sources entirely rather than falling back to `'unsafe-inline'`.
fn build_csp_header(nonce: Option<&str>) -> String {
  let (script_src, style_src) = match nonce {
    Some(n) => (format!("'self' 'nonce-{n}'"), format!("'self' 'nonce-{n}'")),
    None => ("'self'".to_owned(), "'self'".to_owned()),
  };
  format!(
    "default-src 'self'; script-src {script_src}; style-src {style_src}; \
      img-src 'self' https://www.gravatar.com; connect-src 'self'; \
      frame-ancestors 'none'; base-uri 'self'; form-action 'self'"
  )
}

#[cfg(test)]
mod tests {
  use super::*;

  mod build_csp_header {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_embeds_the_nonce_in_script_and_style_sources_when_present() {
      let csp = build_csp_header(Some("abc123"));

      assert!(csp.contains("script-src 'self' 'nonce-abc123'"));
      assert!(csp.contains("style-src 'self' 'nonce-abc123'"));
    }

    #[test]
    fn it_includes_gravatar_in_the_img_src_allowlist() {
      let csp = build_csp_header(Some("x"));

      assert!(csp.contains("img-src 'self' https://www.gravatar.com"));
    }

    #[test]
    fn it_never_emits_unsafe_inline_regardless_of_nonce_presence() {
      let with_nonce = build_csp_header(Some("x"));
      let without_nonce = build_csp_header(None);

      assert!(!with_nonce.contains("'unsafe-inline'"));
      assert!(!without_nonce.contains("'unsafe-inline'"));
    }

    #[test]
    fn it_sets_frame_ancestors_to_none_to_block_framing() {
      let csp = build_csp_header(None);

      assert!(csp.contains("frame-ancestors 'none'"));
    }

    #[test]
    fn it_uses_self_only_when_the_nonce_is_missing() {
      let csp = build_csp_header(None);

      assert_eq!(
        csp,
        "default-src 'self'; script-src 'self'; style-src 'self'; \
          img-src 'self' https://www.gravatar.com; connect-src 'self'; \
          frame-ancestors 'none'; base-uri 'self'; form-action 'self'"
      );
    }
  }
}
