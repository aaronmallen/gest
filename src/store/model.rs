//! Domain model types persisted by the store.
//!
//! Each submodule defines a `Model` struct mirroring one database table,
//! together with `New`/`Patch`/`Filter` helper structs where applicable.
//! `TryFrom<Row>` implementations decode libsql rows into models; conversion
//! errors surface through [`Error`].

pub(crate) mod artifact;
pub(crate) mod author;
pub(crate) mod entity_tag;
pub(crate) mod iteration;
pub(crate) mod note;
/// Core domain types and primitive value objects.
pub mod primitives;
pub(crate) mod project;
pub(crate) mod project_workspace;
pub(crate) mod relationship;
pub(crate) mod tag;
pub(crate) mod task;
pub(crate) mod transaction;

/// An author who creates notes, events, and other actions.
pub use author::Model as Author;
/// A tracked project root directory.
pub use project::Model as Project;
/// A workspace directory within a project.
pub use project_workspace::Model as ProjectWorkspace;
/// A deduplicated label that can be attached to any entity.
pub use tag::Model as Tag;
