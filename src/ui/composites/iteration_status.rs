use std::fmt::{self, Display, Formatter};

use yansi::Paint;

use crate::{
  store::IterationProgress,
  ui::{
    atoms::{id::Id, label::Label, value::Value},
    theme::Theme,
  },
};

/// Fixed padding width for field labels.
const LABEL_PAD: usize = 12;

/// Renders a styled status card for an iteration's progress.
pub struct IterationStatus<'a> {
  id: &'a str,
  progress: &'a IterationProgress,
  status: &'a str,
  theme: &'a Theme,
  title: &'a str,
}

impl<'a> IterationStatus<'a> {
  pub fn new(id: &'a str, title: &'a str, status: &'a str, progress: &'a IterationProgress, theme: &'a Theme) -> Self {
    Self {
      id,
      title,
      status,
      progress,
      theme,
    }
  }

  fn phase_line(&self) -> String {
    match self.progress.active_phase {
      Some(phase) => format!(
        "Phase {}: {}/{} done",
        phase, self.progress.phase_progress.done, self.progress.phase_progress.total,
      ),
      None => "none".to_string(),
    }
  }

  fn overall_line(&self) -> String {
    format!(
      "{}/{}",
      self.progress.overall_progress.done, self.progress.overall_progress.total,
    )
  }

  fn counts_line(&self) -> String {
    let sep = format!("{}", " \u{00b7} ".paint(self.theme.muted));
    let blocked = format!(
      "{} {}",
      self
        .progress
        .blocked
        .to_string()
        .paint(self.theme.iteration_detail_count_blocked),
      "blocked".paint(self.theme.iteration_detail_count_blocked),
    );
    let in_progress = format!(
      "{} {}",
      self
        .progress
        .in_progress
        .to_string()
        .paint(self.theme.iteration_detail_count_in_progress),
      "in progress".paint(self.theme.iteration_detail_count_in_progress),
    );
    format!("{in_progress}{sep}{blocked}")
  }
}

impl Display for IterationStatus<'_> {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    let id = Id::new(self.id, self.theme);

    let title_label = Label::new("title", self.theme.iteration_detail_label).pad_to(LABEL_PAD);
    let title_value = Value::new(self.title, self.theme.iteration_detail_value);

    let status_label = Label::new("status", self.theme.iteration_detail_label).pad_to(LABEL_PAD);
    let status_value = Value::new(self.status, self.theme.iteration_detail_value);

    let phase_label = Label::new("phase", self.theme.iteration_detail_label).pad_to(LABEL_PAD);
    let phase_value = Value::new(
      format!(
        "{} / {}",
        self.progress.active_phase.map_or("-".to_string(), |p| p.to_string()),
        self.progress.total_phases
      ),
      self.theme.iteration_detail_value,
    );

    let progress_label = Label::new("progress", self.theme.iteration_detail_label).pad_to(LABEL_PAD);
    let progress_value = Value::new(self.phase_line(), self.theme.iteration_detail_value);

    let overall_label = Label::new("overall", self.theme.iteration_detail_label).pad_to(LABEL_PAD);
    let overall_value = Value::new(self.overall_line(), self.theme.iteration_detail_value);

    let counts_label = Label::new("activity", self.theme.iteration_detail_label).pad_to(LABEL_PAD);
    let counts_value = self.counts_line();

    let assignees_label = Label::new("assignees", self.theme.iteration_detail_label).pad_to(LABEL_PAD);
    let assignees_text = if self.progress.assignees.is_empty() {
      "none".to_string()
    } else {
      self.progress.assignees.join(", ")
    };
    let assignees_value = Value::new(assignees_text, self.theme.iteration_detail_value);

    writeln!(f, "{id}")?;
    writeln!(f)?;
    writeln!(f, "  {title_label}{title_value}")?;
    writeln!(f, "  {status_label}{status_value}")?;
    writeln!(f, "  {phase_label}{phase_value}")?;
    writeln!(f, "  {progress_label}{progress_value}")?;
    writeln!(f, "  {overall_label}{overall_value}")?;
    writeln!(f, "  {counts_label}{counts_value}")?;
    write!(f, "  {assignees_label}{assignees_value}")
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::store::{IterationProgress, OverallProgress, PhaseProgress};

  fn theme() -> Theme {
    Theme::default()
  }

  fn render(status: &IterationStatus) -> String {
    yansi::disable();
    let out = status.to_string();
    yansi::enable();
    out
  }

  fn sample_progress() -> IterationProgress {
    IterationProgress {
      active_phase: Some(2),
      total_phases: 4,
      phase_progress: PhaseProgress {
        done: 3,
        total: 5,
      },
      blocked: 1,
      in_progress: 2,
      assignees: vec!["agent-a".to_string(), "agent-b".to_string()],
      overall_progress: OverallProgress {
        done: 8,
        total: 15,
      },
    }
  }

  mod display {
    use super::*;

    #[test]
    fn it_renders_id_on_first_line() {
      let t = theme();
      let progress = sample_progress();
      let view = IterationStatus::new("q1ebvmxp", "Q1 Benchmark", "active", &progress, &t);
      let out = render(&view);
      let first_line = out.lines().next().unwrap();
      assert!(first_line.contains("q1ebvmxp"));
    }

    #[test]
    fn it_renders_title_and_status() {
      let t = theme();
      let progress = sample_progress();
      let view = IterationStatus::new("q1ebvmxp", "Q1 Benchmark", "active", &progress, &t);
      let out = render(&view);
      assert!(out.contains("Q1 Benchmark"));
      assert!(out.contains("active"));
    }

    #[test]
    fn it_renders_phase_fraction() {
      let t = theme();
      let progress = sample_progress();
      let view = IterationStatus::new("q1ebvmxp", "Q1 Benchmark", "active", &progress, &t);
      let out = render(&view);
      assert!(out.contains("2 / 4"), "should show active/total phases");
    }

    #[test]
    fn it_renders_phase_progress() {
      let t = theme();
      let progress = sample_progress();
      let view = IterationStatus::new("q1ebvmxp", "Q1 Benchmark", "active", &progress, &t);
      let out = render(&view);
      assert!(out.contains("Phase 2: 3/5 done"));
    }

    #[test]
    fn it_renders_overall_progress() {
      let t = theme();
      let progress = sample_progress();
      let view = IterationStatus::new("q1ebvmxp", "Q1 Benchmark", "active", &progress, &t);
      let out = render(&view);
      assert!(out.contains("8/15"));
    }

    #[test]
    fn it_renders_activity_counts() {
      let t = theme();
      let progress = sample_progress();
      let view = IterationStatus::new("q1ebvmxp", "Q1 Benchmark", "active", &progress, &t);
      let out = render(&view);
      assert!(out.contains("2 in progress"));
      assert!(out.contains("1 blocked"));
    }

    #[test]
    fn it_renders_assignees() {
      let t = theme();
      let progress = sample_progress();
      let view = IterationStatus::new("q1ebvmxp", "Q1 Benchmark", "active", &progress, &t);
      let out = render(&view);
      assert!(out.contains("agent-a, agent-b"));
    }

    #[test]
    fn it_renders_no_active_phase() {
      let t = theme();
      let progress = IterationProgress {
        active_phase: None,
        total_phases: 0,
        phase_progress: PhaseProgress {
          done: 0,
          total: 0,
        },
        blocked: 0,
        in_progress: 0,
        assignees: vec![],
        overall_progress: OverallProgress {
          done: 0,
          total: 0,
        },
      };
      let view = IterationStatus::new("q1ebvmxp", "Empty", "active", &progress, &t);
      let out = render(&view);
      assert!(out.contains("- / 0"), "should show dash for no active phase");
      assert!(
        out.contains("none"),
        "should show none for phase progress and assignees"
      );
    }
  }
}
