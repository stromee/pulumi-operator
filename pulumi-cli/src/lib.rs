use derivative::Derivative;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::process::{ExitStatus, Output};
use tokio::io::{AsyncBufReadExt, BufReader};
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

  pub async fn login(&self, options: LoginOptions) -> ExitStatus {
    let mut command = Command::new("pulumi");
    command.arg("login").arg(options.url);

    self.spawn(command).await
  }

  pub async fn up(&self, options: UpOptions) -> ExitStatus {
    let mut command = Command::new("pulumi");
    command.arg("up");

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

    self.spawn(command).await
  }

  pub async fn spawn(&self, mut command: Command) -> ExitStatus {
    command.stdout(std::process::Stdio::piped());
    command.stderr(std::process::Stdio::piped());
    command.current_dir(&self.workdir);

    let mut child = command.spawn().unwrap();

    let stdout = child.stdout.take().expect("Failed to open stdout");
    let stderr = child.stderr.take().expect("Failed to open stderr");

    let stdout_reader = BufReader::new(stdout);
    let stderr_reader = BufReader::new(stderr);

    let stdout_handle = tokio::spawn(async move {
      let mut lines = stdout_reader.lines();
      while let Some(line) = lines.next_line().await.unwrap() {
        log::info!("{}", line);
      }
    });

    let stderr_handle = tokio::spawn(async move {
      let mut lines = stderr_reader.lines();
      while let Some(line) = lines.next_line().await.unwrap() {
        log::error!("{}", line);
      }
    });

    tokio::try_join!(stdout_handle, stderr_handle).unwrap();

    child.wait().await.unwrap()
  }

  pub async fn destroy(&self, options: DestroyOptions) -> ExitStatus {
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

    self.spawn(command).await
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

pub struct LoginOptions {
  pub url: String,
}
