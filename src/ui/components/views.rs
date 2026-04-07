//! View-level UI components — full entity displays composed from atoms and molecules.

mod artifact_detail;
mod artifact_list;
mod artifact_list_row;
mod create;
mod iteration_detail;
mod iteration_graph;
mod iteration_list;
mod iteration_list_row;
mod link;
mod meta_get;
mod meta_set;
mod note_change;
mod project_list_row;
pub(super) mod search_result;
mod search_results;
mod state_change;
mod tag_change;
mod task_detail;
mod task_list;
mod task_list_row;
mod undo;
mod update;

pub use artifact_detail::Component as ArtifactDetail;
pub use artifact_list::{ArtifactEntry, Component as ArtifactListView};
pub use iteration_detail::{Component as IterationDetail, TaskCounts};
pub use iteration_graph::{Component as IterationGraphView, GraphTask};
pub use iteration_list::{Component as IterationListView, IterationEntry};
pub use meta_get::Component as MetaGet;
pub use project_list_row::Component as ProjectListRow;
pub use search_results::Component as SearchResults;
pub use task_detail::Component as TaskDetail;
pub use task_list::{Component as TaskListView, TaskEntry};
