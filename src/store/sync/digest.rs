use sha2::{Digest, Sha256};

/// Compute the SHA-256 digest of the given content, returned as a hex string.
pub fn compute(content: &[u8]) -> String {
  let mut hasher = Sha256::new();
  hasher.update(content);
  hasher.finalize().iter().map(|b| format!("{b:02x}")).collect()
}

#[cfg(test)]
mod tests {
  use super::*;

  mod compute {
    use pretty_assertions::assert_eq;

    use super::*;

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

    #[test]
    fn it_returns_64_char_hex_string() {
      let d = compute(b"test");
      assert_eq!(d.len(), 64);
      assert!(d.chars().all(|c| c.is_ascii_hexdigit()));
    }
  }
}
