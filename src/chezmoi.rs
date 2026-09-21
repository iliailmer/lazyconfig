use std::{path::PathBuf, process::Command};

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
        let managed: Vec<String> = serde_json::from_str(&output.stdout)?;

        Ok(managed.iter().any(|path| path == &target))
    }

    fn source_path(&self, target: &NormalizedTarget) -> Result<PathBuf, ChezmoiError> {
        let output = self.run(vec![
            "source-path".into(),
            target.as_path().display().to_string(),
        ])?;
        let source = output.stdout.trim();

        if source.is_empty() {
            return Err(ChezmoiError::Command("chezmoi returned an empty source path".into()));
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
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::{
        Chezmoi, ChezmoiError, CliChezMoi, CommandOutput, CommandRunner, FileStatus,
    };
    use crate::domain::NormalizedTarget;

    struct ExpectedRunner {
        expected_args: Vec<String>,
        output: CommandOutput,
    }

    impl CommandRunner for ExpectedRunner {
        fn run(&self, program: &str, args: &[String]) -> Result<CommandOutput, ChezmoiError> {
            assert_eq!(program, "chezmoi");
            assert_eq!(args, self.expected_args);
            Ok(self.output.clone())
        }
    }

    #[test]
    fn recognizes_a_managed_target_from_json_output() {
        let client = CliChezMoi::new(ExpectedRunner {
            expected_args: vec![
                "managed".into(),
                "--format".into(),
                "json".into(),
                "--path-style".into(),
                "absolute".into(),
                "/Users/tester/.config/nvim".into(),
            ],
            output: CommandOutput {
                success: true,
                stdout: "[\"/Users/tester/.config/nvim\"]".into(),
                stderr: String::new(),
            },
        });
        let target = NormalizedTarget::from_registry_path(
            "~/.config/nvim",
            Path::new("/Users/tester"),
        )
        .unwrap();

        assert!(client.is_managed(&target).unwrap());
    }

    #[test]
    fn resolves_a_source_path_from_chezmoi_output() {
        let client = CliChezMoi::new(ExpectedRunner {
            expected_args: vec![
                "source-path".into(),
                "/Users/tester/.config/nvim".into(),
            ],
            output: CommandOutput {
                success: true,
                stdout: "/Users/tester/.local/share/chezmoi/dot_config/nvim\n".into(),
                stderr: String::new(),
            },
        });
        let target = NormalizedTarget::from_registry_path(
            "~/.config/nvim",
            Path::new("/Users/tester"),
        )
        .unwrap();

        assert_eq!(
            client.source_path(&target).unwrap(),
            Path::new("/Users/tester/.local/share/chezmoi/dot_config/nvim")
        );
    }

    #[test]
    fn reads_a_pending_target_change_from_status_output() {
        let client = CliChezMoi::new(ExpectedRunner {
            expected_args: vec![
                "status".into(),
                "--path-style".into(),
                "absolute".into(),
                "/Users/tester/.config/nvim".into(),
            ],
            output: CommandOutput {
                success: true,
                stdout: " M /Users/tester/.config/nvim\n".into(),
                stderr: String::new(),
            },
        });
        let target = NormalizedTarget::from_registry_path(
            "~/.config/nvim",
            Path::new("/Users/tester"),
        )
        .unwrap();

        assert_eq!(
            client.status(&target).unwrap(),
            FileStatus {
                destination_changed: false,
                target_changed: true,
            }
        );
    }

    #[test]
    fn system_runner_captures_successful_command_output() {
        let output = super::SystemCommandRunner
            .run("/usr/bin/printf", &["ready".into()])
            .unwrap();

        assert!(output.success);
        assert_eq!(output.stdout, "ready");
        assert!(output.stderr.is_empty());
    }

    #[test]
    fn applies_only_the_selected_target() {
        let client = CliChezMoi::new(ExpectedRunner {
            expected_args: vec!["apply".into(), "/Users/tester/.config/nvim".into()],
            output: CommandOutput {
                success: true,
                stdout: String::new(),
                stderr: String::new(),
            },
        });
        let target = NormalizedTarget::from_registry_path(
            "~/.config/nvim",
            Path::new("/Users/tester"),
        )
        .unwrap();

        client.apply(&target).unwrap();
    }
}
