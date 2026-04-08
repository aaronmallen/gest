//! Shared application state handed to every axum handler.

use std::sync::Arc;

use getset::Getters;
use tokio::sync::broadcast::Sender;

use crate::store::{Db, model::primitives::Id};

/// Shared state for the web server.
#[derive(Clone, Getters)]
pub struct AppState {
  #[get = "pub"]
  author_id: Option<Id>,
  #[get = "pub"]
  project_id: Id,
  #[get = "pub"]
  reload_tx: Sender<()>,
  #[get = "pub"]
  store: Arc<Db>,
}

impl AppState {
  /// Create a new web server state.
  pub fn new(store: Arc<Db>, project_id: Id) -> Self {
    let (reload_tx, _) = tokio::sync::broadcast::channel(16);
    Self {
      author_id: None,
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
}
