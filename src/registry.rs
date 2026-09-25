use std::{
    collections::HashSet,
    fs,
    io::Write,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::domain::RegisteredApp;

#[derive(Debug, Deserialize)]
pub struct Registry {
    #[serde(default, rename = "app")]
    pub apps: Vec<RegisteredApp>,
}

#[derive(Debug, Error)]
pub enum RegistryError {
    #[error("could not read registry {path}: {source}")]
    Read {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("could not parse registry: {0}")]
    Parse(#[from] toml::de::Error),
    #[error("could not serialize registry entries: {0}")]
    Serialize(#[from] toml::ser::Error),
    #[error("could not write registry {path}: {source}")]
    Write {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("duplicate application id: {0}")]
    DuplicateId(String),
}

impl Registry {
    pub fn load(path: &Path) -> Result<Self, RegistryError> {
        let input = fs::read_to_string(path).map_err(|source| RegistryError::Read {
            path: path.to_path_buf(),
            source,
        })?;

        Self::parse(&input)
    }

    pub fn parse(input: &str) -> Result<Self, RegistryError> {
        let registry = toml::from_str::<Self>(input)?;
        let mut ids = HashSet::new();

        for app in &registry.apps {
            if !ids.insert(&app.id) {
                return Err(RegistryError::DuplicateId(app.id.clone()));
            }
        }

        Ok(registry)
    }

    pub fn append(path: &Path, apps: &[RegisteredApp]) -> Result<(), RegistryError> {
        if apps.is_empty() {
            return Ok(());
        }

        let registry = Self::load(path)?;
        let mut ids = registry
            .apps
            .iter()
            .map(|app| app.id.as_str())
            .collect::<HashSet<_>>();

        for app in apps {
            if !ids.insert(&app.id) {
                return Err(RegistryError::DuplicateId(app.id.clone()));
            }
        }

        let addition = toml::to_string(&AppList { apps })?;
        let mut source = fs::OpenOptions::new()
            .append(true)
            .open(path)
            .map_err(|source| RegistryError::Write {
                path: path.to_path_buf(),
                source,
            })?;
        source
            .write_all(format!("\n{addition}").as_bytes())
            .map_err(|source| RegistryError::Write {
                path: path.to_path_buf(),
                source,
            })
    }
}

#[derive(Serialize)]
struct AppList<'a> {
    #[serde(rename = "app")]
    apps: &'a [RegisteredApp],
}
