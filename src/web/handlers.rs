//! HTTP handlers for the web dashboard, organized by domain.

pub mod api;
pub mod artifact;
pub mod dashboard;
pub mod iteration;
pub mod search;
pub mod task;

pub use api::api_render_markdown;
pub use artifact::{
  artifact_archive, artifact_create_form, artifact_create_submit, artifact_detail, artifact_detail_fragment,
  artifact_edit_form, artifact_list, artifact_list_fragment, artifact_note_add, artifact_update,
};
pub use dashboard::{dashboard, dashboard_fragment, not_found};
pub use iteration::{
  iteration_board, iteration_board_fragment, iteration_detail, iteration_detail_fragment, iteration_list,
  iteration_list_fragment,
};
pub use search::{api_search, search};
pub use task::{
  note_add, task_create_form, task_create_submit, task_detail, task_detail_fragment, task_edit_form, task_list,
  task_list_fragment, task_update,
};
