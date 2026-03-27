use std::io;

use crate::{
  model::{Artifact, Task},
  ui::{markdown, theme::Theme, utils::format_status},
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
  pub fn write_to(&self, w: &mut impl io::Write, theme: &Theme) -> io::Result<()> {
    use yansi::Paint;

    writeln!(w, "{}", self.artifact.title.paint(theme.md_heading))?;
    if let Some(ref kind) = self.artifact.kind {
      writeln!(w, "{}", kind.paint(theme.muted))?;
    }
    if !self.artifact.tags.is_empty() {
      let formatted: Vec<String> = self.artifact.tags.iter().map(|t| format!("@{t}")).collect();
      writeln!(w, "{}", formatted.join(" ").paint(theme.tag))?;
    }
    if !self.artifact.body.is_empty() {
      writeln!(w)?;
      let body = strip_leading_title(&self.artifact.body, &self.artifact.title);
      markdown::render(w, &body, theme)?;
    }
    Ok(())
  }
}

/// Strip a leading `# Title` line from the body if it matches the artifact title,
/// to avoid duplication since the title is already shown as a styled heading.
fn strip_leading_title(body: &str, title: &str) -> String {
  let expected = format!("# {title}");
  if let Some(rest) = body.strip_prefix(&expected)
    && (rest.is_empty() || rest.starts_with('\n'))
  {
    return rest.strip_prefix('\n').unwrap_or(rest).to_string();
  }
  body.to_string()
}

crate::ui::macros::impl_display_via_write_to!(ArtifactDetail<'_>, theme);

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
        yansi::disable();
        let artifact = make_artifact("Test", Some("note"), vec!["a"], "body");
        let detail = ArtifactDetail::new(&artifact);
        let display = detail.to_string();
        let mut buf = Vec::new();
        detail.write_to(&mut buf, &Theme::default()).unwrap();
        let write_output = String::from_utf8(buf).unwrap();
        assert_eq!(display, write_output.trim_end());
      }
    }

    mod write_to {
      use super::*;

      #[test]
      fn it_omits_body_when_empty() {
        yansi::disable();
        let artifact = make_artifact("My Artifact", None, vec![], "");
        let detail = ArtifactDetail::new(&artifact);
        let mut buf = Vec::new();
        detail.write_to(&mut buf, &Theme::default()).unwrap();
        let output = String::from_utf8(buf).unwrap();
        // Should only contain the title line
        let lines: Vec<&str> = output.lines().collect();
        assert_eq!(lines.len(), 1, "Should only have the title line");
      }

      #[test]
      fn it_omits_tags_when_empty() {
        yansi::disable();
        let artifact = make_artifact("My Artifact", None, vec![], "");
        let detail = ArtifactDetail::new(&artifact);
        let mut buf = Vec::new();
        detail.write_to(&mut buf, &Theme::default()).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(!output.contains("@"), "Should not contain tags");
      }

      #[test]
      fn it_omits_type_when_absent() {
        yansi::disable();
        let artifact = make_artifact("My Artifact", None, vec![], "");
        let detail = ArtifactDetail::new(&artifact);
        let mut buf = Vec::new();
        detail.write_to(&mut buf, &Theme::default()).unwrap();
        let output = String::from_utf8(buf).unwrap();
        let lines: Vec<&str> = output.lines().collect();
        // Only title line, no kind line
        assert_eq!(lines.len(), 1, "Should not contain kind line");
      }

      #[test]
      fn it_writes_body_when_present() {
        yansi::disable();
        let artifact = make_artifact("My Artifact", None, vec![], "Some body text");
        let detail = ArtifactDetail::new(&artifact);
        let mut buf = Vec::new();
        detail.write_to(&mut buf, &Theme::default()).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("Some body text"), "Should contain body");
      }

      #[test]
      fn it_writes_tags_when_present() {
        yansi::disable();
        let artifact = make_artifact("My Artifact", None, vec!["rust", "cli"], "");
        let detail = ArtifactDetail::new(&artifact);
        let mut buf = Vec::new();
        detail.write_to(&mut buf, &Theme::default()).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("@rust @cli"), "Should contain @-prefixed tags");
      }

      #[test]
      fn it_writes_title() {
        yansi::disable();
        let artifact = make_artifact("My Artifact", None, vec![], "");
        let detail = ArtifactDetail::new(&artifact);
        let mut buf = Vec::new();
        detail.write_to(&mut buf, &Theme::default()).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("My Artifact"), "Should contain title");
      }

      #[test]
      fn it_writes_type_when_present() {
        yansi::disable();
        let artifact = make_artifact("My Artifact", Some("note"), vec![], "");
        let detail = ArtifactDetail::new(&artifact);
        let mut buf = Vec::new();
        detail.write_to(&mut buf, &Theme::default()).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("note"), "Should contain kind");
      }

      #[test]
      fn it_strips_leading_title_from_body() {
        yansi::disable();
        let artifact = make_artifact("My Artifact", None, vec![], "# My Artifact\nBody content here");
        let detail = ArtifactDetail::new(&artifact);
        let mut buf = Vec::new();
        detail.write_to(&mut buf, &Theme::default()).unwrap();
        let output = String::from_utf8(buf).unwrap();
        // Title should appear once (as heading), not duplicated from body
        let count = output.matches("My Artifact").count();
        assert_eq!(count, 1, "Title should not be duplicated");
        assert!(output.contains("Body content here"), "Should contain rest of body");
      }
    }

    mod strip_leading_title {
      use super::super::strip_leading_title;

      #[test]
      fn it_strips_matching_title() {
        let body = "# My Title\nRest of body";
        assert_eq!(strip_leading_title(body, "My Title"), "Rest of body");
      }

      #[test]
      fn it_preserves_body_when_title_differs() {
        let body = "# Different Title\nRest of body";
        assert_eq!(strip_leading_title(body, "My Title"), body);
      }

      #[test]
      fn it_preserves_body_without_heading() {
        let body = "Just some text";
        assert_eq!(strip_leading_title(body, "My Title"), body);
      }

      #[test]
      fn it_handles_title_only_body() {
        let body = "# My Title";
        assert_eq!(strip_leading_title(body, "My Title"), "");
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
