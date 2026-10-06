//TODO: is this way of calling command line utility like chezmoi the best way?
use std::{
    path::PathBuf,
    process::{Command, Stdio},
};

use thiserror::Error;

use crate::domain::NormalizedTarget;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FileStatus {
    pub destination_changed: bool,
    pub target_changed: bool,
}

impl FileStatus {
    pub fn clean() -> Self {
        Self::default()
    }
}

#[derive(Debug, Error)]
pub enum ChezmoiError {
    #[error("chezmoi command failed: {0}")]
    Command(String),
    #[error("could not parse chezmoi output: {0}")]
    Parse(#[from] serde_json::Error),
}

#[derive(Clone, Debug)]
pub struct CommandOutput {
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
}

pub trait CommandRunner {
    fn run(&self, program: &str, args: &[String]) -> Result<CommandOutput, ChezmoiError>;
}

pub struct SystemCommandRunner;

impl CommandRunner for SystemCommandRunner {
    fn run(&self, program: &str, args: &[String]) -> Result<CommandOutput, ChezmoiError> {
        let output = Command::new(program)
            .args(args)
            .stdin(Stdio::null())
            .output()
            .map_err(|error| ChezmoiError::Command(error.to_string()))?;

        Ok(CommandOutput {
            success: output.status.success(),
            stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        })
    }
}

pub struct CliChezMoi<R> {
    runner: R,
}

impl<R> CliChezMoi<R> {
    pub fn new(runner: R) -> Self {
        Self { runner }
    }
}

pub trait Chezmoi {
    fn is_managed(&self, target: &NormalizedTarget) -> Result<bool, ChezmoiError>;
    fn source_path(&self, target: &NormalizedTarget) -> Result<PathBuf, ChezmoiError>;
    fn status(&self, target: &NormalizedTarget) -> Result<FileStatus, ChezmoiError>;
    fn diff(&self, target: &NormalizedTarget) -> Result<String, ChezmoiError>;
    fn apply(&self, target: &NormalizedTarget) -> Result<(), ChezmoiError>;
}

impl<R: CommandRunner> CliChezMoi<R> {
    fn run(&self, args: Vec<String>) -> Result<CommandOutput, ChezmoiError> {
        let output = self.runner.run("chezmoi", &args)?;
        if output.success {
            Ok(output)
        } else {
            Err(ChezmoiError::Command(output.stderr))
        }
    }
}

impl<R: CommandRunner> Chezmoi for CliChezMoi<R> {
    fn is_managed(&self, target: &NormalizedTarget) -> Result<bool, ChezmoiError> {
        let target = target.as_path().display().to_string();
        let output = self.run(vec![
            "managed".into(),
            "--format".into(),
            "json".into(),
            "--path-style".into(),
            "absolute".into(),
            target.clone(),
        ])?;
        let output = output.stdout.trim();

        if output.is_empty() {
            return Ok(false);
        }

        if output.starts_with('[') {
            let managed: Vec<String> = serde_json::from_str(output)?;
            return Ok(managed.iter().any(|path| path == &target));
        }

        Ok(output.lines().any(|path| path.trim() == target))
    }

    fn source_path(&self, target: &NormalizedTarget) -> Result<PathBuf, ChezmoiError> {
        let output = self.run(vec![
            "source-path".into(),
            target.as_path().display().to_string(),
        ])?;
        let source = output.stdout.trim();

        if source.is_empty() {
            return Err(ChezmoiError::Command(
                "chezmoi returned an empty source path".into(),
            ));
        }

        Ok(PathBuf::from(source))
    }

    fn status(&self, target: &NormalizedTarget) -> Result<FileStatus, ChezmoiError> {
        let output = self.run(vec![
            "status".into(),
            "--path-style".into(),
            "absolute".into(),
            target.as_path().display().to_string(),
        ])?;
        let mut status = FileStatus::clean();

        for line in output.stdout.lines() {
            let bytes = line.as_bytes();
            if bytes.len() < 2 {
                continue;
            }

            status.destination_changed |= bytes[0] != b' ';
            status.target_changed |= bytes[1] != b' ';
        }

        Ok(status)
    }

    fn diff(&self, target: &NormalizedTarget) -> Result<String, ChezmoiError> {
        let output = self.run(vec![
            "diff".into(),
            "--no-pager".into(),
            "--color=false".into(),
            "--recursive".into(),
            target.as_path().display().to_string(),
        ])?;

        Ok(output.stdout)
    }

    fn apply(&self, target: &NormalizedTarget) -> Result<(), ChezmoiError> {
        self.run(vec!["apply".into(), target.as_path().display().to_string()])?;
        Ok(())
    }
}
