use std::sync::Arc;

use tokio::sync::broadcast::Sender;

use crate::store::{Db, model::primitives::Id};

/// Shared state for the web server.
#[derive(Clone)]
pub struct AppState {
  author_id: Option<Id>,
  project_id: Id,
  reload_tx: Sender<()>,
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

  /// The resolved author ID for the current user, if available.
  pub fn author_id(&self) -> Option<&Id> {
    self.author_id.as_ref()
  }

  /// Set the resolved author ID.
  pub fn with_author_id(mut self, id: Id) -> Self {
    self.author_id = Some(id);
    self
  }

  /// The current project ID.
  pub fn project_id(&self) -> &Id {
    &self.project_id
  }

  /// Broadcast channel for SSE reload notifications.
  pub fn reload_tx(&self) -> &Sender<()> {
    &self.reload_tx
  }

  /// The database connection handle.
  pub fn store(&self) -> &Arc<Db> {
    &self.store
  }
}
