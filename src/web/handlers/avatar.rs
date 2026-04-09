//! Local Gravatar-proxy handler backed by [`crate::store::avatar_cache`].
//!
//! Browsers fetch avatars from `/avatars/{hash}` instead of `gravatar.com`
//! directly. The handler consults the shared [`AvatarCache`] on the
//! [`AppState`] which reads a fresh on-disk copy when available and only
//! reaches out to Gravatar on a cache miss.

use axum::{
  extract::{Path, State},
  http::{StatusCode, header},
  response::{IntoResponse, Response},
};

use crate::{
  store::avatar_cache::AvatarCache,
  web::{
    AppState,
    handlers::{AppError, log_err},
  },
};

/// Browser cache lifetime advertised for successful avatar responses.
///
/// Seven days matches the server-side [`crate::store::avatar_cache::DEFAULT_TTL`]
/// and keeps first-party caching consistent with the upstream behaviour the
/// proxy is replacing.
const CACHE_CONTROL_MAX_AGE: &str = "public, max-age=604800";

/// Maximum length of an avatar hash accepted by the handler.
///
/// Real Gravatar identifiers are either MD5 (32 chars) or SHA-256 (64 chars);
/// anything longer is almost certainly an attempt to abuse the route.
const MAX_HASH_LEN: usize = 64;

/// `GET /avatars/:hash` — serve a cached avatar, fetching it on a miss.
///
/// The handler delegates to [`AvatarCache::get_or_fetch`] which validates the
/// hash (rejecting anything outside `[0-9a-f]+`) before hitting the filesystem
/// or upstream Gravatar. The response body is the raw image bytes with the
/// `Content-Type` reported by either the upstream fetch or the fallback used
/// on cache hits, and a long `Cache-Control` header so browsers avoid
/// re-requesting the same avatar on every page load.
pub async fn avatar_get(
  State(state): State<AppState>,
  Path(hash): Path<String>,
) -> crate::web::handlers::Result<Response> {
  // Validate the hash in the handler so we can cleanly distinguish malformed
  // input (400) from transient upstream fetch failures (500) — both of which
  // surface as `store::Error::InvalidValue` out of `AvatarCache::get_or_fetch`.
  if hash.is_empty() || hash.len() > MAX_HASH_LEN || !hash.chars().all(|c| c.is_ascii_hexdigit()) {
    return Err(AppError::BadRequest(format!("invalid avatar hash: {hash:?}")));
  }

  let cache: &AvatarCache = state.avatar_cache();
  let (bytes, mime) = cache.get_or_fetch(&hash).await.map_err(log_err("avatar_get"))?;

  Ok(
    (
      StatusCode::OK,
      [
        (header::CONTENT_TYPE, mime.as_ref().to_owned()),
        (header::CACHE_CONTROL, CACHE_CONTROL_MAX_AGE.to_owned()),
      ],
      bytes,
    )
      .into_response(),
  )
}

#[cfg(test)]
mod tests {
  use std::{fs, sync::Arc};

  use axum::{
    body::to_bytes,
    extract::{Path, State},
    http::StatusCode,
  };
  use tempfile::TempDir;

  use super::*;
  use crate::{
    store::{self, avatar_cache::AvatarCache, model::Project},
    web::AppState,
  };

  async fn setup_with_cache(tmp: &TempDir) -> AppState {
    let (store, db_tmp) = store::open_temp().await.unwrap();
    let conn = store.connect().await.unwrap();
    let project = Project::new("/tmp/web-avatar-test".into());
    conn
      .execute(
        "INSERT INTO projects (id, root, created_at, updated_at) VALUES (?1, ?2, ?3, ?4)",
        [
          project.id().to_string(),
          project.root().to_string_lossy().into_owned(),
          project.created_at().to_rfc3339(),
          project.updated_at().to_rfc3339(),
        ],
      )
      .await
      .unwrap();
    std::mem::forget(db_tmp);

    let cache = Arc::new(AvatarCache::new(tmp.path()));
    AppState::new(store, project.id().clone()).with_avatar_cache(cache)
  }

  mod avatar_get {
    use pretty_assertions::assert_eq;

    use super::*;

    #[tokio::test]
    async fn it_rejects_a_non_hex_hash_with_bad_request() {
      let tmp = TempDir::new().unwrap();
      let state = setup_with_cache(&tmp).await;

      let result = super::super::avatar_get(State(state), Path("../escape".to_owned())).await;

      let err = result.unwrap_err();
      assert!(matches!(err, AppError::BadRequest(_)), "got {err:?}");
    }

    #[tokio::test]
    async fn it_serves_cached_bytes_on_a_cache_hit() {
      let tmp = TempDir::new().unwrap();
      let hash = "d41d8cd98f00b204e9800998ecf8427e";
      let avatars_dir = tmp.path().join("avatars");
      fs::create_dir_all(&avatars_dir).unwrap();
      fs::write(avatars_dir.join(hash), b"cached-bytes").unwrap();
      let state = setup_with_cache(&tmp).await;

      let response = super::super::avatar_get(State(state), Path(hash.to_owned()))
        .await
        .expect("cache hit should succeed");

      assert_eq!(response.status(), StatusCode::OK);

      let content_type = response
        .headers()
        .get(header::CONTENT_TYPE)
        .unwrap()
        .to_str()
        .unwrap()
        .to_owned();
      let cache_control = response
        .headers()
        .get(header::CACHE_CONTROL)
        .unwrap()
        .to_str()
        .unwrap()
        .to_owned();

      assert_eq!(content_type, "image/jpeg");
      assert_eq!(cache_control, "public, max-age=604800");

      let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
      assert_eq!(body.as_ref(), b"cached-bytes");
    }

    #[tokio::test]
    async fn it_takes_the_miss_path_when_the_cached_entry_is_expired() {
      use std::{thread, time::Duration};

      let tmp = TempDir::new().unwrap();
      let hash = "d41d8cd98f00b204e9800998ecf8427e";
      let avatars_dir = tmp.path().join("avatars");
      fs::create_dir_all(&avatars_dir).unwrap();
      let entry = avatars_dir.join(hash);
      fs::write(&entry, b"stale").unwrap();
      thread::sleep(Duration::from_millis(5));

      let (store, db_tmp) = store::open_temp().await.unwrap();
      let conn = store.connect().await.unwrap();
      let project = Project::new("/tmp/web-avatar-miss".into());
      conn
        .execute(
          "INSERT INTO projects (id, root, created_at, updated_at) VALUES (?1, ?2, ?3, ?4)",
          [
            project.id().to_string(),
            project.root().to_string_lossy().into_owned(),
            project.created_at().to_rfc3339(),
            project.updated_at().to_rfc3339(),
          ],
        )
        .await
        .unwrap();
      std::mem::forget(db_tmp);

      let cache = Arc::new(AvatarCache::with_ttl(tmp.path(), Duration::from_nanos(1)));
      let state = AppState::new(store, project.id().clone()).with_avatar_cache(cache);

      let result = super::super::avatar_get(State(state), Path(hash.to_owned())).await;

      // The miss path must either (a) successfully fetch fresh bytes and
      // overwrite the stale entry, or (b) surface a fetch error after
      // removing the expired file. What it must *never* do is serve the
      // stale `b"stale"` bytes: that would mean the TTL check was skipped.
      match result {
        Ok(response) => {
          assert_eq!(response.status(), StatusCode::OK);
          let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
          assert_ne!(body.as_ref(), b"stale", "miss path must not serve stale bytes");
          assert!(entry.exists(), "successful fetch should repopulate the entry");
        }
        Err(_) => {
          assert!(!entry.exists(), "expired entry should be removed when the fetch fails");
        }
      }
    }
  }
}
