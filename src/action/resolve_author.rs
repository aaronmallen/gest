use crate::{
  cli,
  model::{event::AuthorInfo, note::AuthorType},
};

/// Resolve the current author identity from git config.
///
/// When `agent` is `true` the returned [`AuthorInfo`] uses [`AuthorType::Agent`];
/// otherwise it uses [`AuthorType::Human`]. If `git config user.name` is not
/// set, the author falls back to `"unknown"`.
pub fn resolve_author(agent: bool) -> cli::Result<AuthorInfo> {
  let author_type = if agent { AuthorType::Agent } else { AuthorType::Human };

  match crate::cli::git::resolve_author() {
    Some(a) => Ok(AuthorInfo {
      author: a.name,
      author_email: a.email,
      author_type,
    }),
    None => Ok(AuthorInfo {
      author: "unknown".to_string(),
      author_email: None,
      author_type,
    }),
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  mod resolve_author_fn {
    use super::*;

    #[test]
    fn it_returns_human_author_type() {
      let info = resolve_author(false).unwrap();
      assert_eq!(info.author_type, AuthorType::Human);
    }

    #[test]
    fn it_returns_agent_author_type() {
      let info = resolve_author(true).unwrap();
      assert_eq!(info.author_type, AuthorType::Agent);
    }
  }
}
