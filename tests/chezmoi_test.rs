use std::path::Path;

use lazyconfig::{
    chezmoi::{Chezmoi, ChezmoiError, CliChezMoi, CommandOutput, CommandRunner, FileStatus},
    domain::NormalizedTarget,
};

struct ExpectedRunner {
    expected_args: Vec<String>,
    stdout: &'static str,
}

impl CommandRunner for ExpectedRunner {
    fn run(&self, program: &str, args: &[String]) -> Result<CommandOutput, ChezmoiError> {
        assert_eq!(program, "chezmoi");
        assert_eq!(args, self.expected_args);
        Ok(CommandOutput {
            success: true,
            stdout: self.stdout.into(),
            stderr: String::new(),
        })
    }
}

fn nvim_target() -> NormalizedTarget {
    NormalizedTarget::from_registry_path("~/.config/nvim", Path::new("/Users/tester")).unwrap()
}

#[test]
fn recognizes_a_managed_target_from_line_output() {
    let client = CliChezMoi::new(ExpectedRunner {
        expected_args: vec![
            "managed".into(),
            "--format".into(),
            "json".into(),
            "--path-style".into(),
            "absolute".into(),
            "/Users/tester/.config/nvim".into(),
        ],
        stdout: "/Users/tester/.config/nvim\n/Users/tester/.config/nvim/init.lua\n",
    });

    assert!(client.is_managed(&nvim_target()).unwrap());
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
        stdout: " M /Users/tester/.config/nvim\n",
    });

    assert_eq!(
        client.status(&nvim_target()).unwrap(),
        FileStatus {
            destination_changed: false,
            target_changed: true,
        }
    );
}

#[test]
fn applies_only_the_selected_target() {
    let client = CliChezMoi::new(ExpectedRunner {
        expected_args: vec!["apply".into(), "/Users/tester/.config/nvim".into()],
        stdout: "",
    });

    client.apply(&nvim_target()).unwrap();
}

#[test]
fn reads_the_diff_of_only_the_selected_target() {
    let client = CliChezMoi::new(ExpectedRunner {
        expected_args: vec![
            "diff".into(),
            "--no-pager".into(),
            "--color=false".into(),
            "--recursive".into(),
            "/Users/tester/.config/nvim".into(),
        ],
        stdout: "-font_size 15.0\n+font_size 16.0\n",
    });

    assert_eq!(
        client.diff(&nvim_target()).unwrap(),
        "-font_size 15.0\n+font_size 16.0\n"
    );
}
