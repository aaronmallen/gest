use clap::Args;

use crate::{
  AppContext,
  cli::Error,
  store::{model::primitives::EntityType, repo},
  ui::{components::TaskDetail, json},
};

/// Show a task by ID or prefix.
#[derive(Args, Debug)]
pub struct Command {
  /// The task ID or prefix.
  id: String,
  #[command(flatten)]
  output: json::Flags,
}

impl Command {
  /// Resolve the task and render its details, description, and tags.
  pub async fn call(&self, context: &AppContext) -> Result<(), Error> {
    log::debug!("task show: entry");
    let conn = context.store().connect().await?;

    let project_id = context.project_id().as_ref().ok_or(Error::UninitializedProject)?;

    let id = repo::resolve::resolve_id(&conn, "tasks", &self.id).await?;
    let task = repo::task::find_required_by_id(&conn, id).await?;

    let tags = repo::tag::for_entity(&conn, EntityType::Task, task.id()).await?;

    // Highlighted prefix tracks the task's own pool: an active task gets
    // the active-only minimum, while a terminal task uses the all-rows
    // minimum so the prefix in the heading round-trips through `task show`.
    let prefix_len = if task.status().is_terminal() {
      repo::task::shortest_all_prefix(&conn, project_id).await?
    } else {
      repo::task::shortest_active_prefix(&conn, project_id).await?
    };

    let short_id = task.id().short();
    self.output.print_entity(&task, &short_id, || {
      let status_str = task.status().to_string();
      let id_short = task.id().short();
      let mut view = TaskDetail::new(&id_short, task.title(), &status_str)
        .id_prefix_len(prefix_len)
        .priority(task.priority());

      if !tags.is_empty() {
        view = view.tags(&tags);
      }

      if !task.description().is_empty() {
        view = view.body(Some(task.description()));
      }

      format!("{view}")
    })?;
    Ok(())
  }
}
