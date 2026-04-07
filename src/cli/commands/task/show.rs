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
  pub async fn call(&self, context: &AppContext) -> Result<(), Error> {
    let conn = context.store().connect().await?;

    let id = repo::resolve::resolve_id(&conn, "tasks", &self.id).await?;
    let task = repo::task::find_by_id(&conn, id.clone())
      .await?
      .ok_or_else(|| Error::Resolve(repo::resolve::Error::NotFound(self.id.clone())))?;

    let tags = repo::tag::for_entity(&conn, EntityType::Task, task.id()).await?;

    let short_id = task.id().short();
    self.output.print_entity(&task, &short_id, || {
      let status_str = task.status().to_string();
      let id_short = task.id().short();
      let mut view = TaskDetail::new(&id_short, task.title(), &status_str).priority(task.priority());

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
