use std::{fs, io, path::Path};

pub const EXAMPLE_REGISTRY: &str = include_str!("../examples/config.toml");

#[derive(Debug, PartialEq, Eq)]
pub enum InitOutcome {
    Created,
    AlreadyExists,
}

pub fn init(registry_path: &Path) -> io::Result<InitOutcome> {
    if registry_path.exists() {
        return Ok(InitOutcome::AlreadyExists);
    }

    if let Some(directory) = registry_path.parent() {
        fs::create_dir_all(directory)?;
    }
    fs::write(registry_path, EXAMPLE_REGISTRY)?;

    Ok(InitOutcome::Created)
}

pub fn track_command(registry_path: &Path) -> String {
    format!(
        "chezmoi add {}",
        shell_words::quote(&registry_path.display().to_string())
    )
}
