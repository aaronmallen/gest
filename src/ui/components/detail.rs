use std::io;

use crate::{
  model::{Artifact, Task},
  ui::{theme::Theme, utils::format_status},
};

/// Detail view for an artifact, matching the output of `artifact show`.
pub struct ArtifactDetail<'a> {
  artifact: &'a Artifact,
}

impl<'a> ArtifactDetail<'a> {
  pub fn new(artifact: &'a Artifact) -> Self {
    Self {
      artifact,
    }
  }

  /// Write the detail view to the given writer.
  pub fn write_to(&self, w: &mut impl io::Write) -> io::Result<()> {
    writeln!(w, "# {}", self.artifact.title)?;
    if let Some(ref kind) = self.artifact.kind {
      writeln!(w, "**Type:** {kind}")?;
    }
    if !self.artifact.tags.is_empty() {
      writeln!(w, "**Tags:** {}", self.artifact.tags.join(", "))?;
    }
    if !self.artifact.body.is_empty() {
      writeln!(w)?;
      write!(w, "{}", self.artifact.body)?;
      if !self.artifact.body.ends_with('\n') {
        writeln!(w)?;
      }
    }
    Ok(())
  }
}

crate::ui::macros::impl_display_via_write_to!(ArtifactDetail<'_>);

/// Detail view for a task, matching the output of `task show`.
pub struct TaskDetail<'a> {
  task: &'a Task,
}

impl<'a> TaskDetail<'a> {
  pub fn new(task: &'a Task) -> Self {
    Self {
      task,
    }
  }

  /// Write the detail view to the given writer.
  pub fn write_to(&self, w: &mut impl io::Write, theme: &Theme) -> io::Result<()> {
    writeln!(w, "# {}", self.task.title)?;
    writeln!(w)?;
    writeln!(w, "**Status:** {}", format_status(&self.task.status, theme))?;
    writeln!(w, "**ID:** {}", self.task.id)?;
    writeln!(
      w,
      "**Created:** {}",
      self.task.created_at.format("%Y-%m-%d %H:%M:%S UTC")
    )?;
    writeln!(
      w,
      "**Updated:** {}",
      self.task.updated_at.format("%Y-%m-%d %H:%M:%S UTC")
    )?;

    if let Some(resolved_at) = self.task.resolved_at {
      writeln!(w, "**Resolved:** {}", resolved_at.format("%Y-%m-%d %H:%M:%S UTC"))?;
    }

    if !self.task.tags.is_empty() {
      writeln!(w, "**Tags:** {}", self.task.tags.join(", "))?;
    }

    if !self.task.description.is_empty() {
      writeln!(w)?;
      writeln!(w, "## Description")?;
      writeln!(w)?;
      writeln!(w, "{}", self.task.description)?;
    }

    if !self.task.links.is_empty() {
      writeln!(w)?;
      writeln!(w, "## Links")?;
      writeln!(w)?;
      for link in &self.task.links {
        writeln!(w, "- **{}:** {}", link.rel, link.ref_)?;
      }
    }

    if !self.task.metadata.is_empty() {
      writeln!(w)?;
      writeln!(w, "## Metadata")?;
      writeln!(w)?;
      for (key, value) in &self.task.metadata {
        writeln!(w, "- **{key}:** {value}")?;
      }
    }

    Ok(())
  }
}

crate::ui::macros::impl_display_via_write_to!(TaskDetail<'_>, theme);

#[cfg(test)]
mod tests {
  use chrono::Utc;

  use super::*;
  use crate::model::{Id, Status};

  mod artifact_detail {
    use super::*;

    fn make_artifact(title: &str, kind: Option<&str>, tags: Vec<&str>, body: &str) -> Artifact {
      let now = Utc::now();
      Artifact {
        archived_at: None,
        body: body.to_string(),
        created_at: now,
        id: Id::new(),
        kind: kind.map(|k| k.to_string()),
        metadata: yaml_serde::Mapping::new(),
        tags: tags.into_iter().map(|t| t.to_string()).collect(),
        title: title.to_string(),
        updated_at: now,
      }
    }

    mod display {
      use super::*;

      #[test]
      fn it_delegates_to_write_to() {
        let artifact = make_artifact("Test", Some("note"), vec!["a"], "body");
        let detail = ArtifactDetail::new(&artifact);
        let display = detail.to_string();
        let mut buf = Vec::new();
        detail.write_to(&mut buf).unwrap();
        let write_output = String::from_utf8(buf).unwrap();
        assert_eq!(display, write_output.trim_end());
      }
    }

    mod write_to {
      use super::*;

      #[test]
      fn it_omits_body_when_empty() {
        let artifact = make_artifact("My Artifact", None, vec![], "");
        let detail = ArtifactDetail::new(&artifact);
        let mut buf = Vec::new();
        detail.write_to(&mut buf).unwrap();
        let output = String::from_utf8(buf).unwrap();
        // Should only contain the title line
        let lines: Vec<&str> = output.lines().collect();
        assert_eq!(lines.len(), 1, "Should only have the title line");
      }

      #[test]
      fn it_omits_tags_when_empty() {
        let artifact = make_artifact("My Artifact", None, vec![], "");
        let detail = ArtifactDetail::new(&artifact);
        let mut buf = Vec::new();
        detail.write_to(&mut buf).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(!output.contains("**Tags:**"), "Should not contain tags line");
      }

      #[test]
      fn it_omits_type_when_absent() {
        let artifact = make_artifact("My Artifact", None, vec![], "");
        let detail = ArtifactDetail::new(&artifact);
        let mut buf = Vec::new();
        detail.write_to(&mut buf).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(!output.contains("**Type:**"), "Should not contain type line");
      }

      #[test]
      fn it_writes_body_when_present() {
        let artifact = make_artifact("My Artifact", None, vec![], "Some body text");
        let detail = ArtifactDetail::new(&artifact);
        let mut buf = Vec::new();
        detail.write_to(&mut buf).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("Some body text"), "Should contain body");
      }

      #[test]
      fn it_writes_tags_when_present() {
        let artifact = make_artifact("My Artifact", None, vec!["rust", "cli"], "");
        let detail = ArtifactDetail::new(&artifact);
        let mut buf = Vec::new();
        detail.write_to(&mut buf).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("**Tags:** rust, cli"), "Should contain tags");
      }

      #[test]
      fn it_writes_title() {
        let artifact = make_artifact("My Artifact", None, vec![], "");
        let detail = ArtifactDetail::new(&artifact);
        let mut buf = Vec::new();
        detail.write_to(&mut buf).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("# My Artifact"), "Should contain title heading");
      }

      #[test]
      fn it_writes_type_when_present() {
        let artifact = make_artifact("My Artifact", Some("note"), vec![], "");
        let detail = ArtifactDetail::new(&artifact);
        let mut buf = Vec::new();
        detail.write_to(&mut buf).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("**Type:** note"), "Should contain type");
      }
    }
  }

  mod task_detail {
    use super::*;

    fn make_task() -> Task {
      let now = Utc::now();
      Task {
        resolved_at: None,
        created_at: now,
        description: String::new(),
        id: "zyxwvutsrqponmlkzyxwvutsrqponmlk".parse().unwrap(),
        links: vec![],
        metadata: toml::Table::new(),
        status: Status::Open,
        tags: vec![],
        title: "Test Task".to_string(),
        updated_at: now,
      }
    }

    mod display {
      use super::*;

      #[test]
      fn it_delegates_to_write_to() {
        let mut task = make_task();
        task.description = "Some desc".to_string();
        task.tags = vec!["test".to_string()];
        let detail = TaskDetail::new(&task);
        let display = detail.to_string();
        let mut buf = Vec::new();
        detail.write_to(&mut buf, &Theme::default()).unwrap();
        let write_output = String::from_utf8(buf).unwrap();
        assert_eq!(display, write_output.trim_end());
      }
    }
  }
}
