//! Data-access repositories for domain models.
//!
//! Each submodule wraps a single table (or tightly coupled set of tables) and
//! exposes free functions that take a [`libsql::Connection`] so the caller
//! controls transaction boundaries.

pub mod artifact;
pub mod author;
pub mod entity;
pub mod iteration;
pub mod note;
pub mod project;
pub mod relationship;
pub mod resolve;
pub mod search;
pub mod tag;
pub mod task;
pub mod transaction;
