//! Reusable UI components organized by granularity (atoms, molecules, views).

mod atoms;
mod molecules;
mod views;

pub use atoms::{Id, min_unique_prefix};
pub use molecules::{Banner, EmptyList, ErrorMessage, FieldList, SuccessMessage, UpdateNotice};
pub use views::{
  ArtifactDetail, ArtifactEntry, ArtifactListView, GraphTask, IterationDetail, IterationEntry, IterationGraphView,
  IterationListView, MetaGet, ProjectListRow, SearchResults, TaskCounts, TaskDetail, TaskEntry, TaskListView,
};
