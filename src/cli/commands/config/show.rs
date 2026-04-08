use clap::Args;

use crate::{
  AppContext,
  cli::Error,
  config,
  ui::{components::FieldList, json},
};

/// Show the current configuration.
#[derive(Args, Debug)]
pub struct Command {
  #[command(flatten)]
  output: json::Flags,
}

impl Command {
  pub async fn call(&self, context: &AppContext) -> Result<(), Error> {
    log::debug!("config show: entry");
    let settings = context.settings();

    self.output.print_entity(settings, "config", || {
      let toml = toml::to_string_pretty(settings).unwrap_or_default();
      let mut fields = FieldList::new();

      if let Some(global) = config::active_global_config_path() {
        fields = fields.field("global config", global.display().to_string());
      } else {
        fields = fields.field("global config", "(none)".to_string());
      }

      let project_paths = config::active_project_config_paths();
      if project_paths.is_empty() {
        fields = fields.field("project config", "(none)".to_string());
      } else {
        for (i, path) in project_paths.iter().enumerate() {
          let label = if i == 0 {
            "project config".to_string()
          } else {
            String::new()
          };
          fields = fields.field(label, path.display().to_string());
        }
      }

      format!("{fields}\n\n{toml}")
    })?;
    Ok(())
  }
}
