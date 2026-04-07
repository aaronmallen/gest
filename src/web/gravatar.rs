//! Gravatar URL helper.

/// Build a Gravatar URL from an email address (or `None`).
pub(crate) fn url(email: Option<&str>) -> Option<String> {
  let email = email?.trim().to_lowercase();
  let hash = format!("{:x}", md5::compute(email.as_bytes()));
  Some(format!("https://www.gravatar.com/avatar/{hash}?s=64&d=retro"))
}
