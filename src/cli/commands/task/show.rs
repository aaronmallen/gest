use clap::Args;

use crate::{
  AppContext,
  cli::Error,
  store::{model::primitives::EntityType, repo},
  ui::{components::TaskDetail, envelope::Envelope, json},
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

    let id = repo::resolve::resolve_id(&conn, repo::resolve::Table::Tasks, &self.id).await?;
    let task = repo::task::find_required_by_id(&conn, id).await?;

    let envelope = Envelope::load_one(&conn, EntityType::Task, task.id(), &task, true).await?;

    let prefix_map = repo::task::per_id_prefix_lengths(&conn, project_id).await?;
    let prefix_len = prefix_map.get(&task.id().to_string()).copied().unwrap_or(1);

    let tags = repo::tag::for_entity(&conn, EntityType::Task, task.id()).await?;

    let short_id = task.id().short();
    self.output.print_envelope(&envelope, &short_id, || {
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
