//! Shared application state handed to every axum handler.

use std::sync::Arc;

use getset::Getters;
use tokio::sync::broadcast::Sender;

use crate::store::{Db, avatar_cache::AvatarCache, model::primitives::Id};

/// Shared state for the web server.
#[derive(Clone, Getters)]
pub struct AppState {
  #[get = "pub"]
  author_id: Option<Id>,
  #[get = "pub"]
  avatar_cache: Arc<AvatarCache>,
  #[get = "pub"]
  project_id: Id,
  #[get = "pub"]
  reload_tx: Sender<()>,
  #[get = "pub"]
  store: Arc<Db>,
}

impl AppState {
  /// Create a new web server state.
  ///
  /// The avatar cache defaults to a process-scoped temp directory; production
  /// callers should override it via [`Self::with_avatar_cache`] so entries are
  /// persisted under the user's configured `storage.cache_dir`.
  pub fn new(store: Arc<Db>, project_id: Id) -> Self {
    let (reload_tx, _) = tokio::sync::broadcast::channel(16);
    let avatar_cache = Arc::new(AvatarCache::new(std::env::temp_dir().join("gest-avatar-cache")));
    Self {
      author_id: None,
      avatar_cache,
      project_id,
      reload_tx,
      store,
    }
  }

  /// Set the resolved author ID.
  pub fn with_author_id(mut self, id: Id) -> Self {
    self.author_id = Some(id);
    self
  }

  /// Set the avatar cache used by the `/avatars/:hash` handler.
  pub fn with_avatar_cache(mut self, cache: Arc<AvatarCache>) -> Self {
    self.avatar_cache = cache;
    self
  }
}
