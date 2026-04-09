//! Avatar URL helper.
//!
//! Hashes an email the same way Gravatar does and returns a first-party
//! `/avatars/{hash}` URL served by [`crate::web::handlers::avatar`]. The
//! dashboard never references `gravatar.com` directly: all avatar requests go
//! through the local proxy so browsing activity is not leaked to a third
//! party and the dashboard works in air-gapped deployments.

/// Build a first-party avatar URL from an email address (or `None`).
///
/// Returns `None` when the email is absent so templates can conditionally
/// omit the `<img>` tag. The hash format matches Gravatar's scheme (lowercase
/// MD5 of the trimmed, lowercased email) so the local [`AvatarCache`] can use
/// the same identifier when fetching upstream bytes on a cache miss.
///
/// [`AvatarCache`]: crate::store::avatar_cache::AvatarCache
pub(crate) fn url(email: Option<&str>) -> Option<String> {
  let email = email?.trim().to_lowercase();
  let hash = format!("{:x}", md5::compute(email.as_bytes()));
  Some(format!("/avatars/{hash}"))
}

#[cfg(test)]
mod tests {
  use super::*;

  mod template_audit {
    use std::{fs, path::Path};

    /// Recursively walk a template directory and fail if any file still
    /// references `gravatar.com`. This guards against regressions that
    /// bypass the [`url`] helper — for example, hard-coding an `<img src=>`
    /// somewhere new — by turning the security audit item into a
    /// machine-checked invariant.
    fn scan(dir: &Path) {
      for entry in fs::read_dir(dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_dir() {
          scan(&path);
        } else if path.extension().and_then(|s| s.to_str()) == Some("html") {
          let content = fs::read_to_string(&path).unwrap();
          assert!(
            !content.contains("gravatar.com"),
            "template {} still references gravatar.com",
            path.display()
          );
        }
      }
    }

    #[test]
    fn it_finds_no_gravatar_com_references_under_the_templates_tree() {
      let templates = Path::new(env!("CARGO_MANIFEST_DIR")).join("templates");

      scan(&templates);
    }
  }

  mod url {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_hashes_the_email_into_a_local_avatar_path() {
      let href = url(Some("Test@Example.COM")).unwrap();

      // Lowercase MD5 of "test@example.com".
      assert_eq!(href, "/avatars/55502f40dc8b7c769880b10874abc9d0");
    }

    #[test]
    fn it_never_returns_a_gravatar_com_url() {
      let href = url(Some("anyone@example.com")).unwrap();

      assert!(!href.contains("gravatar.com"));
      assert!(href.starts_with("/avatars/"));
    }

    #[test]
    fn it_returns_none_when_no_email_is_provided() {
      assert!(url(None).is_none());
    }
  }
}
