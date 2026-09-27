use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Adapter {
    Raw,
    Starship,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub struct RegisteredApp {
    pub id: String,
    pub label: String,
    pub target: String,
    pub adapter: Adapter,
    #[serde(default)]
    pub ignore: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NormalizedTarget(PathBuf);

#[derive(Debug, Error)]
pub enum TargetPathError {
    #[error("target must be absolute or start with ~/")]
    Relative,
}

impl NormalizedTarget {
    pub fn from_registry_path(path: &str, home: &Path) -> Result<Self, TargetPathError> {
        if path == "~" {
            return Ok(Self(home.to_path_buf()));
        }

        if let Some(relative_path) = path.strip_prefix("~/") {
            return Ok(Self(home.join(relative_path)));
        }

        let path = PathBuf::from(path);
        if path.is_absolute() {
            return Ok(Self(path));
        }

        Err(TargetPathError::Relative)
    }

    pub fn as_path(&self) -> &Path {
        &self.0
    }
}
