use std::fmt::{self, Display, Formatter};

use yansi::Paint;

use super::{
  super::{atoms::Separator, molecules::EmptyList},
  search_result::Component as SearchResult,
};
use crate::{
  store::model::{artifact, iteration, task},
  ui::style,
};

/// Complete search results view with grouped sections and summary.
pub struct Component {
  artifacts: Vec<artifact::Model>,
  expanded: bool,
  iterations: Vec<iteration::Model>,
  query: String,
  tasks: Vec<task::Model>,
}

impl Component {
  /// Create a new search results view.
  pub fn new(
    query: String,
    tasks: Vec<task::Model>,
    artifacts: Vec<artifact::Model>,
    iterations: Vec<iteration::Model>,
  ) -> Self {
    Self {
      artifacts,
      expanded: false,
      iterations,
      query,
      tasks,
    }
  }

  /// Enable expanded mode to show full description/body per result.
  pub fn expanded(mut self, expanded: bool) -> Self {
    self.expanded = expanded;
    self
  }

  fn total(&self) -> usize {
    self.tasks.len() + self.artifacts.len() + self.iterations.len()
  }
}

impl Display for Component {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    let theme = style::global();

    if self.total() == 0 {
      writeln!(f)?;
      write!(f, "{}", EmptyList::new("results"))?;
      return Ok(());
    }

    // Summary line
    writeln!(f)?;
    writeln!(
      f,
      "  found {} {} for '{}'",
      self.total().to_string().paint(*theme.search_summary()),
      if self.total() == 1 { "result" } else { "results" },
      self.query.paint(*theme.search_query()),
    )?;

    // Tasks section
    if !self.tasks.is_empty() {
      writeln!(f)?;
      writeln!(
        f,
        "  {}",
        Separator::labeled(format!("tasks ({})", self.tasks.len()), *theme.border())
      )?;
      for task in &self.tasks {
        writeln!(f)?;
        let body = if self.expanded {
          let d = task.description();
          if d.is_empty() { None } else { Some(d.to_string()) }
        } else {
          None
        };
        write!(
          f,
          "{}",
          SearchResult::task(task.id().short(), task.title().to_string(), task.status().to_string())
            .body(body)
            .expanded(self.expanded),
        )?;
      }
    }

    // Artifacts section
    if !self.artifacts.is_empty() {
      writeln!(f)?;
      writeln!(
        f,
        "  {}",
        Separator::labeled(format!("artifacts ({})", self.artifacts.len()), *theme.border()),
      )?;
      for artifact in &self.artifacts {
        writeln!(f)?;
        let body = if self.expanded {
          let b = artifact.body();
          if b.is_empty() { None } else { Some(b.to_string()) }
        } else {
          None
        };
        write!(
          f,
          "{}",
          SearchResult::artifact(artifact.id().short(), artifact.title().to_string())
            .body(body)
            .expanded(self.expanded),
        )?;
      }
    }

    // Iterations section
    if !self.iterations.is_empty() {
      writeln!(f)?;
      writeln!(
        f,
        "  {}",
        Separator::labeled(format!("iterations ({})", self.iterations.len()), *theme.border()),
      )?;
      for iteration in &self.iterations {
        writeln!(f)?;
        let body = if self.expanded {
          let d = iteration.description();
          if d.is_empty() { None } else { Some(d.to_string()) }
        } else {
          None
        };
        write!(
          f,
          "{}",
          SearchResult::iteration(
            iteration.id().short(),
            iteration.title().to_string(),
            iteration.status().to_string(),
          )
          .body(body)
          .expanded(self.expanded),
        )?;
      }
    }

    Ok(())
  }
}
