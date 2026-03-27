use std::io;

use yansi::Paint;

use crate::{
  model::{Artifact, Task},
  ui::{
    markdown,
    theme::Theme,
    utils::{format_status, format_tags},
  },
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
    // Title heading
    writeln!(w, "{}", self.task.title.paint(theme.md_heading))?;
    // Status on its own line
    writeln!(w, "{}", format_status(&self.task.status, theme))?;
    // Tags with @ prefix (omitted if empty)
    if !self.task.tags.is_empty() {
      writeln!(w, "{}", format_tags(&self.task.tags, theme))?;
    }

    // Blank line before description
    if !self.task.description.is_empty() {
      writeln!(w)?;
      markdown::render(w, &self.task.description, theme)?;
    }

    // Links section (omitted if empty)
    if !self.task.links.is_empty() {
      writeln!(w)?;
      writeln!(w, "{}", "── Links ──".paint(theme.border))?;
      writeln!(w)?;
      for link in &self.task.links {
        writeln!(w, "- **{}:** {}", link.rel, link.ref_)?;
      }
    }

    // Metadata section (omitted if empty)
    if !self.task.metadata.is_empty() {
      writeln!(w)?;
      writeln!(w, "{}", "── Metadata ──".paint(theme.border))?;
      writeln!(w)?;
      for (key, value) in &self.task.metadata {
        writeln!(w, "- **{key}:** {value}")?;
      }
    }

    Ok(())
  }
}

crate::ui::macros::impl_display_via_write_to!(TaskDetail<'_>, theme);

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

#[cfg(test)]
mod tests {
  use super::*;
  use crate::model::Id;

  mod artifact_detail {
    use super::*;

    fn make_artifact(title: &str, kind: Option<&str>, tags: Vec<&str>, body: &str) -> Artifact {
      let id = Id::new();
      Artifact {
        body: body.to_string(),
        id,
        kind: kind.map(|k| k.to_string()),
        tags: tags.into_iter().map(|t| t.to_string()).collect(),
        title: title.to_string(),
        ..crate::test_helpers::make_test_artifact("zyxwvutsrqponmlkzyxwvutsrqponmlk")
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

    mod strip_leading_title {
      use super::super::strip_leading_title;

      #[test]
      fn it_handles_title_only_body() {
        let body = "# My Title";
        assert_eq!(strip_leading_title(body, "My Title"), "");
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
      fn it_strips_matching_title() {
        let body = "# My Title\nRest of body";
        assert_eq!(strip_leading_title(body, "My Title"), "Rest of body");
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
    }
  }

  mod task_detail {
    use super::*;
    use crate::model::{Link, RelationshipType};

    fn make_task() -> Task {
      Task {
        title: "Test Task".to_string(),
        ..crate::test_helpers::make_test_task("zyxwvutsrqponmlkzyxwvutsrqponmlk")
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

    mod write_to {
      use super::*;

      #[test]
      fn it_writes_styled_title_heading() {
        let task = make_task();
        let detail = TaskDetail::new(&task);
        let mut buf = Vec::new();
        detail.write_to(&mut buf, &Theme::default()).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("Test Task"), "Should contain styled title");
      }

      #[test]
      fn it_writes_status_on_own_line() {
        let task = make_task();
        let detail = TaskDetail::new(&task);
        let mut buf = Vec::new();
        detail.write_to(&mut buf, &Theme::default()).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("open"), "Should contain status text");
        // Status should not be on same line as title
        let lines: Vec<&str> = output.lines().collect();
        assert!(lines.len() >= 2, "Should have at least 2 lines (title + status)");
      }

      #[test]
      fn it_omits_tags_when_empty() {
        let task = make_task();
        let detail = TaskDetail::new(&task);
        let mut buf = Vec::new();
        detail.write_to(&mut buf, &Theme::default()).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(!output.contains("@"), "Should not contain @ tags when empty");
      }

      #[test]
      fn it_writes_tags_with_at_prefix() {
        let mut task = make_task();
        task.tags = vec!["rust".to_string(), "cli".to_string()];
        let detail = TaskDetail::new(&task);
        let mut buf = Vec::new();
        detail.write_to(&mut buf, &Theme::default()).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("@rust"), "Should contain @rust tag");
        assert!(output.contains("@cli"), "Should contain @cli tag");
      }

      #[test]
      fn it_renders_description_with_markdown() {
        let mut task = make_task();
        task.description = "Some **bold** description".to_string();
        let detail = TaskDetail::new(&task);
        let mut buf = Vec::new();
        detail.write_to(&mut buf, &Theme::default()).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("bold"), "Should contain rendered markdown description");
      }

      #[test]
      fn it_omits_description_when_empty() {
        let task = make_task();
        let detail = TaskDetail::new(&task);
        let mut buf = Vec::new();
        detail.write_to(&mut buf, &Theme::default()).unwrap();
        let output = String::from_utf8(buf).unwrap();
        let lines: Vec<&str> = output.lines().collect();
        // With no description, tags, links, or metadata: just title + status
        assert_eq!(lines.len(), 2, "Should only have title and status lines");
      }

      #[test]
      fn it_writes_links_with_separator() {
        let mut task = make_task();
        task.links = vec![Link {
          ref_: "https://example.com".to_string(),
          rel: RelationshipType::RelatesTo,
        }];
        let detail = TaskDetail::new(&task);
        let mut buf = Vec::new();
        detail.write_to(&mut buf, &Theme::default()).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("Links"), "Should contain Links separator");
        assert!(output.contains("https://example.com"), "Should contain link reference");
      }

      #[test]
      fn it_omits_links_when_empty() {
        let task = make_task();
        let detail = TaskDetail::new(&task);
        let mut buf = Vec::new();
        detail.write_to(&mut buf, &Theme::default()).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(
          !output.contains("Links"),
          "Should not contain Links separator when empty"
        );
      }

      #[test]
      fn it_writes_metadata_with_separator() {
        let mut task = make_task();
        task
          .metadata
          .insert("wave".to_string(), toml::Value::String("3".to_string()));
        let detail = TaskDetail::new(&task);
        let mut buf = Vec::new();
        detail.write_to(&mut buf, &Theme::default()).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("Metadata"), "Should contain Metadata separator");
        assert!(output.contains("wave"), "Should contain metadata key");
      }

      #[test]
      fn it_omits_metadata_when_empty() {
        let task = make_task();
        let detail = TaskDetail::new(&task);
        let mut buf = Vec::new();
        detail.write_to(&mut buf, &Theme::default()).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(
          !output.contains("Metadata"),
          "Should not contain Metadata separator when empty"
        );
      }
    }
  }
}
