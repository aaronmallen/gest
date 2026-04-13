//! Content digest computation and `sync_digests` cache helpers.
//!
//! [`compute`] is a thin wrapper over SHA-256 used by the per-entity adapters
//! to decide whether an on-disk file needs rewriting. [`is_unchanged`] and
//! [`record`] are the read/write halves of the `sync_digests` cache, keyed by
//! `(project_id, repo-relative path)`.

use chrono::Utc;
use libsql::Connection;
use sha2::{Digest, Sha256};

use crate::store::{Error, model::primitives::Id};

/// Compute the SHA-256 digest of `content`, returned as a lowercase hex string.
pub fn compute(content: &[u8]) -> String {
  let mut hasher = Sha256::new();
  hasher.update(content);
  hasher.finalize().iter().map(|b| format!("{b:02x}")).collect()
}

/// Return `true` if the cached digest for `(project_id, relative_path)` matches
/// `digest`. Returns `false` when no entry exists or when the entry is stale.
pub async fn is_unchanged(
  conn: &Connection,
  project_id: &Id,
  relative_path: &str,
  digest: &str,
) -> Result<bool, Error> {
  let mut rows = conn
    .query(
      "SELECT digest FROM sync_digests WHERE relative_path = ?1 AND project_id = ?2",
      [relative_path.to_string(), project_id.to_string()],
    )
    .await?;
  let Some(row) = rows.next().await? else {
    return Ok(false);
  };
  let cached: String = row.get(0)?;
  Ok(cached == digest)
}

/// Remove the cached digest for `relative_path` so that a follow-up
/// `undo` + export cycle sees the cache as stale and rewrites the file.
pub async fn invalidate(conn: &Connection, project_id: &Id, relative_path: &str) -> Result<(), Error> {
  conn
    .execute(
      "DELETE FROM sync_digests WHERE relative_path = ?1 AND project_id = ?2",
      [relative_path.to_owned(), project_id.to_string()],
    )
    .await?;
  Ok(())
}

/// Insert or update the cached digest for `(project_id, relative_path)`.
pub async fn record(conn: &Connection, project_id: &Id, relative_path: &str, digest: &str) -> Result<(), Error> {
  conn
    .execute(
      "INSERT INTO sync_digests (project_id, relative_path, digest, synced_at) \
        VALUES (?1, ?2, ?3, ?4) \
        ON CONFLICT(project_id, relative_path) DO UPDATE SET digest = ?3, synced_at = ?4",
      [
        project_id.to_string(),
        relative_path.to_string(),
        digest.to_string(),
        Utc::now().to_rfc3339(),
      ],
    )
    .await?;
  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;

  mod compute {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_returns_64_char_hex_string() {
      let d = compute(b"test");

      assert_eq!(d.len(), 64);
      assert!(d.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn it_returns_consistent_digest() {
      let d1 = compute(b"hello world");
      let d2 = compute(b"hello world");

      assert_eq!(d1, d2);
    }

    #[test]
    fn it_returns_different_digest_for_different_content() {
      let d1 = compute(b"hello");
      let d2 = compute(b"world");

      assert_ne!(d1, d2);
    }
  }
}
