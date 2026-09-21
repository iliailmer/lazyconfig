use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
};

use serde::Deserialize;
use thiserror::Error;

use crate::domain::RegisteredApp;

#[derive(Debug, Deserialize)]
pub struct Registry {
    #[serde(rename = "app")]
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
}
