use derivative::Derivative;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::process::Output;
use tokio::process::Command;

pub struct PulumiCLI {
  workdir: PathBuf,
}

impl PulumiCLI {
  pub fn new(workdir: impl AsRef<Path>) -> Self {
    PulumiCLI {
      workdir: workdir.as_ref().to_path_buf(),
    }
  }

  pub async fn up(&self, options: UpOptions) -> Output {
    let mut command = Command::new("pulumi");
    command.current_dir(&self.workdir).arg("up");

    if let Some(config) = &options.config {
      command.arg("--config").arg(config);
    }
    if let Some(config_file) = &options.config_file {
      command.arg("--config-file").arg(config_file);
    }
    if options.debug {
      command.arg("--debug");
    }
    if options.diff {
      command.arg("--diff");
    }
    if options.expect_no_changes {
      command.arg("--expect-no-changes");
    }
    if let Some(message) = &options.message {
      command.arg("--message").arg(message);
    }
    if let Some(parallel) = options.parallel {
      command.arg("--parallel").arg(parallel.to_string());
    }
    if options.refresh.unwrap_or(true) {
      command.arg("--refresh");
    }
    if options.skip_preview {
      command.arg("--skip-preview");
    }
    if let Some(stack) = &options.stack {
      command.arg("--stack").arg(stack);
    }
    if options.yes {
      command.arg("--yes");
    }
    if options.show_config {
      command.arg("--show-config");
    }
    if options.show_full_output.unwrap_or(true) {
      command.arg("--show-full-output");
    }
    if options.show_reads {
      command.arg("--show-reads");
    }
    if options.show_replacement_steps {
      command.arg("--show-replacement-steps");
    }
    if options.show_sames {
      command.arg("--show-sames");
    }

    command.output().await.unwrap()
  }

  pub async fn destroy(&self, options: DestroyOptions) -> Output {
    let mut command = Command::new("pulumi");
    command.arg("destroy");

    if let Some(stack) = &options.stack {
      command.arg("--stack").arg(stack);
    }
    if options.yes {
      command.arg("--yes");
    }
    if options.skip_preview {
      command.arg("--skip-preview");
    }

    command.output().await.unwrap()
  }
}

#[derive(Derivative)]
#[derivative(Debug, Default)]
pub struct UpOptions {
  pub config: Option<String>,
  pub config_file: Option<String>,
  pub debug: bool,
  pub diff: bool,
  pub expect_no_changes: bool,
  pub message: Option<String>,
  pub parallel: Option<i32>,
  pub refresh: Option<bool>,
  pub skip_preview: bool,
  pub stack: Option<String>,
  #[derivative(Default(value = "true"))]
  pub yes: bool,
  pub show_config: bool,
  pub show_full_output: Option<bool>,
  pub show_reads: bool,
  pub show_replacement_steps: bool,
  pub show_sames: bool,
}

pub struct DestroyOptions {
  pub stack: Option<String>,
  pub yes: bool,
  pub skip_preview: bool,
}
