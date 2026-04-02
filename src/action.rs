//! Action abstraction layer providing entity traits for generic operations.

pub mod link;
mod resolve_author;
mod resolve_note_prefix;
mod set_status;
mod tag;
mod traits;
mod untag;

pub use resolve_author::resolve_author;
pub use resolve_note_prefix::resolve_note_prefix;
pub use set_status::set_status;
pub use tag::tag;
pub use traits::{HasStatus, Linkable, Resolvable, Storable, Taggable};
pub use untag::untag;
