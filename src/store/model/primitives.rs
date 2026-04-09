//! Primitive value types used across the domain model.

mod author_type;
mod entity_type;
mod id;
mod iteration_status;
mod priority;
mod relationship_type;
mod task_status;

/// Distinguishes human authors from automated agents.
pub use author_type::Primitive as AuthorType;
/// Tags a model value with the domain entity kind it refers to.
pub use entity_type::Primitive as EntityType;
/// 128-bit identifier shared by every domain model.
pub use id::Primitive as Id;
/// Lifecycle status tracked on each iteration.
pub use iteration_status::Primitive as IterationStatus;
/// Relative importance assigned to a task.
pub use priority::Primitive as Priority;
/// Semantic kind of relationship between two entities.
pub use relationship_type::Primitive as RelationshipType;
/// Lifecycle status tracked on each task.
pub use task_status::Primitive as TaskStatus;
