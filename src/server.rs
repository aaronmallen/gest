//! Built-in web server for browsing gest entities in a browser.

mod assets;
mod handlers;
mod routes;
mod state;
mod templates;

pub use routes::router;
pub use state::ServerState;
