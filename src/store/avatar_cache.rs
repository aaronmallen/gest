//! Filesystem-backed cache for Gravatar avatar bytes.
//!
//! The cache lives under `{storage.cache_dir}/avatars/` and keys each entry by
//! an already-hashed Gravatar identifier (typically the lowercase trimmed email
//! passed through MD5 or SHA-256). Entries are reused for
//! [`DEFAULT_TTL`](self::DEFAULT_TTL) before they are refetched.
//!
//! This module provides the storage layer only: the HTTP route that surfaces
//! cached bytes to browsers is wired up separately as part of the Gravatar
//! proxy feature. Callers fetch on demand via [`AvatarCache::get_or_fetch`].

// The HTTP wiring that consumes this cache lands in a later phase of the
// Gravatar proxy work. Until then the items below are exercised only by unit
// tests, so suppress the dead-code warnings at the module boundary.
#![allow(dead_code)]

use std::{
  fs,
  io::ErrorKind,
  path::{Path, PathBuf},
  time::{Duration, SystemTime},
};

use mime::Mime;
use reqwest::Client;

use crate::store::Error;

/// How long a cached avatar remains valid before it is refetched.
///
/// Seven days matches typical Gravatar cache headers and keeps the cache small
/// while ensuring profile picture changes propagate in a reasonable window.
pub const DEFAULT_TTL: Duration = Duration::from_secs(60 * 60 * 24 * 7);

/// Subdirectory (under `storage.cache_dir`) where avatar bytes are written.
const AVATAR_SUBDIR: &str = "avatars";

/// Default query appended to [`GRAVATAR_URL`] when fetching a missing entry.
///
/// `d=identicon` ensures we always receive a valid image even for unknown
/// emails; `s=160` keeps payloads small enough to be cheap to cache.
const GRAVATAR_QUERY: &str = "?d=identicon&s=160";

/// Upstream template for fetching Gravatar bytes by identifier hash.
const GRAVATAR_URL: &str = "https://www.gravatar.com/avatar/";

/// Filesystem-backed Gravatar avatar cache.
///
/// Each [`AvatarCache`] is scoped to a single `storage.cache_dir` root. Bytes
/// are written to `{root}/avatars/{hash}` so multiple cache flavours can share
/// the same user-visible cache directory without colliding.
pub struct AvatarCache {
  client: Client,
  root: PathBuf,
  ttl: Duration,
}

impl AvatarCache {
  /// Construct a new cache rooted at `cache_dir` with the default TTL.
  ///
  /// `cache_dir` is the user-facing `storage.cache_dir`; the module appends
  /// its own `avatars/` subdirectory internally. The directory is not created
  /// eagerly — it is materialized on the first write in
  /// [`Self::get_or_fetch`].
  pub fn new(cache_dir: impl Into<PathBuf>) -> Self {
    Self::with_ttl(cache_dir, DEFAULT_TTL)
  }

  /// Construct a new cache with an explicit TTL. Primarily used by tests.
  pub fn with_ttl(cache_dir: impl Into<PathBuf>, ttl: Duration) -> Self {
    Self {
      client: Client::new(),
      root: cache_dir.into().join(AVATAR_SUBDIR),
      ttl,
    }
  }

  /// Return the cached bytes for `hash`, fetching from Gravatar on a miss or
  /// when the on-disk copy is older than [`Self::ttl`].
  ///
  /// The returned MIME type is derived from the upstream `Content-Type` header
  /// when fetching and defaults to `image/jpeg` on cache hits, which matches
  /// Gravatar's canonical response format.
  pub async fn get_or_fetch(&self, hash: &str) -> Result<(Vec<u8>, Mime), Error> {
    validate_hash(hash)?;
    let path = self.entry_path(hash);

    if let Some(bytes) = self.read_fresh(&path)? {
      log::trace!("avatar cache hit for {hash}");
      return Ok((bytes, mime::IMAGE_JPEG));
    }

    log::trace!("avatar cache miss for {hash}; fetching upstream");
    let url = format!("{GRAVATAR_URL}{hash}{GRAVATAR_QUERY}");
    let response = self
      .client
      .get(&url)
      .send()
      .await
      .map_err(|e| Error::InvalidValue(format!("avatar fetch failed: {e}")))?
      .error_for_status()
      .map_err(|e| Error::InvalidValue(format!("avatar fetch status: {e}")))?;

    let content_type = response
      .headers()
      .get(reqwest::header::CONTENT_TYPE)
      .and_then(|v| v.to_str().ok())
      .and_then(|v| v.parse::<Mime>().ok())
      .unwrap_or(mime::IMAGE_JPEG);

    let bytes = response
      .bytes()
      .await
      .map_err(|e| Error::InvalidValue(format!("avatar body read failed: {e}")))?
      .to_vec();

    self.write(&path, &bytes)?;
    Ok((bytes, content_type))
  }

  /// Return the TTL applied to cache entries.
  pub fn ttl(&self) -> Duration {
    self.ttl
  }

  /// Resolve the on-disk path for a cache entry.
  fn entry_path(&self, hash: &str) -> PathBuf {
    self.root.join(hash)
  }

  /// Read a cache entry if it exists and is still within TTL.
  ///
  /// Returns `Ok(None)` for both misses and expired entries; expired entries
  /// are also eagerly removed so the caller always observes a clean slate on
  /// the subsequent write.
  fn read_fresh(&self, path: &Path) -> Result<Option<Vec<u8>>, Error> {
    let metadata = match fs::metadata(path) {
      Ok(m) => m,
      Err(e) if e.kind() == ErrorKind::NotFound => return Ok(None),
      Err(e) => return Err(Error::Io(e)),
    };

    let modified = metadata.modified().map_err(Error::Io)?;
    if SystemTime::now().duration_since(modified).unwrap_or_default() > self.ttl {
      log::trace!("avatar cache expired at {}", path.display());
      let _ = fs::remove_file(path);
      return Ok(None);
    }

    let bytes = fs::read(path).map_err(Error::Io)?;
    Ok(Some(bytes))
  }

  /// Persist `bytes` to `path`, creating parent directories as needed.
  fn write(&self, path: &Path, bytes: &[u8]) -> Result<(), Error> {
    if let Some(parent) = path.parent() {
      fs::create_dir_all(parent).map_err(Error::Io)?;
    }
    fs::write(path, bytes).map_err(Error::Io)?;
    Ok(())
  }
}

/// Reject hashes that would escape the cache directory or contain suspicious
/// characters. Gravatar hashes are always lowercase hexadecimal.
fn validate_hash(hash: &str) -> Result<(), Error> {
  if hash.is_empty() || !hash.chars().all(|c| c.is_ascii_hexdigit()) {
    return Err(Error::InvalidValue(format!("invalid avatar hash: {hash:?}")));
  }
  Ok(())
}

#[cfg(test)]
mod tests {
  use std::{fs::File, io::Write as _, time::Duration};

  use tempfile::TempDir;

  use super::*;

  mod get_or_fetch {
    use super::*;

    #[tokio::test]
    async fn it_rejects_invalid_hashes() {
      let tmp = TempDir::new().unwrap();
      let cache = AvatarCache::new(tmp.path());

      let err = cache.get_or_fetch("../escape").await.unwrap_err();

      assert!(matches!(err, Error::InvalidValue(_)), "got {err:?}");
    }

    #[tokio::test]
    async fn it_reports_a_miss_when_no_entry_exists() {
      let tmp = TempDir::new().unwrap();
      let cache = AvatarCache::new(tmp.path());
      let hash = "d41d8cd98f00b204e9800998ecf8427e";

      // On a miss, read_fresh returns None without touching the network.
      let entry = cache.entry_path(hash);
      assert!(cache.read_fresh(&entry).unwrap().is_none());
    }

    #[tokio::test]
    async fn it_returns_cached_bytes_on_hit() {
      let tmp = TempDir::new().unwrap();
      let cache = AvatarCache::new(tmp.path());
      let hash = "d41d8cd98f00b204e9800998ecf8427e";

      let avatars_dir = tmp.path().join("avatars");
      std::fs::create_dir_all(&avatars_dir).unwrap();
      let entry = avatars_dir.join(hash);
      File::create(&entry).unwrap().write_all(b"cached-bytes").unwrap();

      let (bytes, mime) = cache.get_or_fetch(hash).await.unwrap();

      assert_eq!(bytes, b"cached-bytes");
      assert_eq!(mime, mime::IMAGE_JPEG);
    }

    #[tokio::test]
    async fn it_treats_expired_entries_as_a_miss() {
      let tmp = TempDir::new().unwrap();
      let cache = AvatarCache::with_ttl(tmp.path(), Duration::from_nanos(1));
      let hash = "d41d8cd98f00b204e9800998ecf8427e";

      let avatars_dir = tmp.path().join("avatars");
      std::fs::create_dir_all(&avatars_dir).unwrap();
      let entry = avatars_dir.join(hash);
      File::create(&entry).unwrap().write_all(b"stale").unwrap();
      // Sleep briefly so the entry exceeds the 1ns TTL.
      std::thread::sleep(Duration::from_millis(5));

      assert!(cache.read_fresh(&entry).unwrap().is_none());
      assert!(!entry.exists(), "expired entry should be removed on read");
    }
  }

  mod validate_hash {
    use super::*;

    #[test]
    fn it_accepts_lowercase_hex() {
      assert!(validate_hash("d41d8cd98f00b204e9800998ecf8427e").is_ok());
    }

    #[test]
    fn it_rejects_empty() {
      assert!(validate_hash("").is_err());
    }

    #[test]
    fn it_rejects_path_separators() {
      assert!(validate_hash("../foo").is_err());
      assert!(validate_hash("a/b").is_err());
    }
  }
}
