use std::path::Path;

use lazyconfig::{
    chezmoi::{ChezmoiError, CommandOutput},
    editor::{InteractiveCommandRunner, SourceEditor},
};

struct ExpectedRunner;

impl InteractiveCommandRunner for ExpectedRunner {
    fn run_interactive(
        &self,
        program: &str,
        args: &[String],
    ) -> Result<CommandOutput, ChezmoiError> {
        assert_eq!(program, "nvim");
        assert_eq!(
            args,
            [
                "--clean".to_owned(),
                "/source/dot_config/nvim/init.lua".to_owned(),
            ]
        );

        Ok(CommandOutput {
            success: true,
            stdout: String::new(),
            stderr: String::new(),
        })
    }
}

#[test]
fn opens_the_resolved_chezmoi_source_with_the_configured_editor() {
    let editor = SourceEditor::new("nvim --clean", ExpectedRunner).unwrap();

    editor
        .open(Path::new("/source/dot_config/nvim/init.lua"))
        .unwrap();
}
