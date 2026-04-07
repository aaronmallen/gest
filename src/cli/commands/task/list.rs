use clap::Args;

use crate::{
  AppContext,
  cli::Error,
  store::{
    model::{
      primitives::{EntityType, TaskStatus},
      task::Filter,
    },
    repo,
  },
  ui::{
    components::{TaskEntry, TaskListView, min_unique_prefix},
    json,
  },
};

/// List tasks in the current project.
#[derive(Args, Debug)]
pub struct Command {
  /// Show all tasks, including resolved.
  #[arg(long, short)]
  all: bool,
  /// Filter by assigned author name.
  #[arg(long)]
  assigned_to: Option<String>,
  /// Filter by status.
  #[arg(long, short)]
  status: Option<TaskStatus>,
  /// Filter by tag.
  #[arg(long, short)]
  tag: Option<String>,
  #[command(flatten)]
  output: json::Flags,
}

impl Command {
  pub async fn call(&self, context: &AppContext) -> Result<(), Error> {
    let project_id = context.project_id().as_ref().ok_or(Error::UninitializedProject)?;
    let conn = context.store().connect().await?;

    let filter = Filter {
      all: self.all,
      assigned_to: self.assigned_to.clone(),
      status: self.status,
      tag: self.tag.clone(),
    };

    let tasks = repo::task::all(&conn, project_id, &filter).await?;

    let id_shorts: Vec<String> = tasks.iter().map(|t| t.id().short().to_string()).collect();

    if self.output.json {
      let json = serde_json::to_string_pretty(&tasks)?;
      println!("{json}");
      return Ok(());
    }

    if self.output.quiet {
      for id in &id_shorts {
        println!("{id}");
      }
      return Ok(());
    }

    let id_refs: Vec<&str> = id_shorts.iter().map(|s| s.as_str()).collect();
    let prefix_len = min_unique_prefix(&id_refs);

    let mut entries = Vec::new();
    for (task, id_short) in tasks.iter().zip(id_shorts.iter()) {
      let tags = repo::tag::for_entity(&conn, EntityType::Task, task.id()).await?;
      entries.push(TaskEntry {
        blocked_by: None,
        blocking: false,
        id: id_short.clone(),
        priority: task.priority(),
        status: task.status().to_string(),
        tags,
        title: task.title().to_string(),
      });
    }

    println!("{}", TaskListView::new(entries, prefix_len));

    Ok(())
  }
}
