use clap::Args;

use crate::{
  cli::{self, AppContext},
  model::ArtifactFilter,
  store,
  ui::{
    composites::empty_list::EmptyList,
    views::artifact::{ArtifactListView, ArtifactViewData},
  },
};

/// List artifacts, optionally filtered by type, tag, or archive status.
#[derive(Debug, Args)]
pub struct Command {
  /// Show only archived artifacts.
  #[arg(long)]
  pub archived: bool,
  /// Output as JSON.
  #[arg(short, long)]
  pub json: bool,
  /// Filter by artifact type (e.g. spec, adr, rfc).
  #[arg(short = 'k', long = "type")]
  pub kind: Option<String>,
  /// Include archived artifacts alongside active ones.
  #[arg(short = 'a', long = "all")]
  pub show_all: bool,
  /// Filter by tag.
  #[arg(long)]
  pub tag: Option<String>,
}

impl Command {
  /// Fetch, filter, and display the artifact list (or JSON output).
  pub fn call(&self, ctx: &AppContext) -> cli::Result<()> {
    let layout = &ctx.layout;
    let theme = &ctx.theme;
    let filter = ArtifactFilter {
      kind: self.kind.clone(),
      only_archived: self.archived,
      show_all: self.show_all,
      tag: self.tag.clone(),
    };

    let artifacts = store::list_artifacts(layout, &filter)?;

    if self.json {
      let json = serde_json::to_string_pretty(&artifacts)?;
      println!("{json}");
      return Ok(());
    }

    if artifacts.is_empty() {
      println!("{}", EmptyList::new("artifacts", theme));
      return Ok(());
    }

    let total = artifacts.len();
    let archived = artifacts.iter().filter(|a| a.archived_at.is_some()).count();

    let data: Vec<ArtifactViewData> = artifacts
      .into_iter()
      .map(|a| ArtifactViewData {
        id: a.id.to_string(),
        title: a.title,
        kind: a.kind,
        tags: a.tags,
        is_archived: a.archived_at.is_some(),
      })
      .collect();

    let view = ArtifactListView::new(total, archived, theme).artifacts(data);
    println!("{view}");

    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{
    store,
    test_helpers::{make_test_artifact, make_test_context},
  };

  mod call {
    use super::*;

    #[test]
    fn it_filters_by_tag() {
      let dir = tempfile::tempdir().unwrap();
      let ctx = make_test_context(dir.path());

      let mut a1 = make_test_artifact("zyxwvutsrqponmlkzyxwvutsrqponmlk");
      a1.tags = vec!["spec".to_string()];
      store::write_artifact(&ctx.layout, &a1).unwrap();

      let a2 = make_test_artifact("kkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkk");
      store::write_artifact(&ctx.layout, &a2).unwrap();

      let cmd = Command {
        json: false,
        show_all: false,
        archived: false,
        kind: None,
        tag: Some("spec".to_string()),
      };

      cmd.call(&ctx).unwrap();
    }

    #[test]
    fn it_handles_empty_list() {
      let dir = tempfile::tempdir().unwrap();
      let ctx = make_test_context(dir.path());

      let cmd = Command {
        json: false,
        show_all: false,
        archived: false,
        kind: None,
        tag: None,
      };

      cmd.call(&ctx).unwrap();
    }

    #[test]
    fn it_includes_archived_with_show_all() {
      let dir = tempfile::tempdir().unwrap();
      let ctx = make_test_context(dir.path());

      let a = make_test_artifact("zyxwvutsrqponmlkzyxwvutsrqponmlk");
      store::write_artifact(&ctx.layout, &a).unwrap();
      store::archive_artifact(&ctx.layout, &a.id).unwrap();

      let cmd = Command {
        json: false,
        show_all: true,
        archived: false,
        kind: None,
        tag: None,
      };

      cmd.call(&ctx).unwrap();
    }

    #[test]
    fn it_lists_artifacts() {
      let dir = tempfile::tempdir().unwrap();
      let ctx = make_test_context(dir.path());
      let artifact = make_test_artifact("zyxwvutsrqponmlkzyxwvutsrqponmlk");
      store::write_artifact(&ctx.layout, &artifact).unwrap();

      let cmd = Command {
        json: false,
        show_all: false,
        archived: false,
        kind: None,
        tag: None,
      };

      cmd.call(&ctx).unwrap();
    }

    #[test]
    fn it_outputs_json() {
      let dir = tempfile::tempdir().unwrap();
      let ctx = make_test_context(dir.path());
      let artifact = make_test_artifact("zyxwvutsrqponmlkzyxwvutsrqponmlk");
      store::write_artifact(&ctx.layout, &artifact).unwrap();

      let cmd = Command {
        json: true,
        show_all: false,
        archived: false,
        kind: None,
        tag: None,
      };

      cmd.call(&ctx).unwrap();
    }
  }
}
