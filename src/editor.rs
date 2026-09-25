use std::{path::Path, process::Command};

use thiserror::Error;

use crate::chezmoi::{ChezmoiError, CommandOutput};

pub trait InteractiveCommandRunner {
    fn run_interactive(
        &self,
        program: &str,
        args: &[String],
    ) -> Result<CommandOutput, ChezmoiError>;
}

pub struct SystemEditorRunner;

impl InteractiveCommandRunner for SystemEditorRunner {
    fn run_interactive(
        &self,
        program: &str,
        args: &[String],
    ) -> Result<CommandOutput, ChezmoiError> {
        let status = Command::new(program)
            .args(args)
            .status()
            .map_err(|error| ChezmoiError::Command(error.to_string()))?;

        Ok(CommandOutput {
            success: status.success(),
            stdout: String::new(),
            stderr: String::new(),
        })
    }
}

pub struct SourceEditor<R> {
    program: String,
    args: Vec<String>,
    runner: R,
}

#[derive(Debug, Error)]
pub enum EditorError {
    #[error("could not parse editor command: {0}")]
    Parse(#[from] shell_words::ParseError),
    #[error("editor command is empty")]
    EmptyCommand,
    #[error(transparent)]
    Command(#[from] ChezmoiError),
    #[error("editor failed: {0}")]
    Failed(String),
}

impl<R: InteractiveCommandRunner> SourceEditor<R> {
    pub fn new(command: &str, runner: R) -> Result<Self, EditorError> {
        let mut parts = shell_words::split(command)?;
        if parts.is_empty() {
            return Err(EditorError::EmptyCommand);
        }

        let program = parts.remove(0);
        Ok(Self {
            program,
            args: parts,
            runner,
        })
    }

    pub fn open(&self, source: &Path) -> Result<(), EditorError> {
        let mut args = self.args.clone();
        args.push(source.display().to_string());
        let output = self.runner.run_interactive(&self.program, &args)?;

        if output.success {
            Ok(())
        } else {
            Err(EditorError::Failed(output.stderr))
        }
    }
}
