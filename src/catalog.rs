use std::path::{Path, PathBuf};

use crate::{
    chezmoi::{Chezmoi, ChezmoiError, FileStatus},
    domain::{NormalizedTarget, RegisteredApp, TargetPathError},
    registry::Registry,
};

#[derive(Debug, PartialEq, Eq)]
pub struct ManagedApp {
    pub app: RegisteredApp,
    pub target: NormalizedTarget,
    pub source: PathBuf,
    pub status: FileStatus,
}

#[derive(Debug, PartialEq, Eq)]
pub enum AppState {
    Managed(ManagedApp),
    Unmanaged(RegisteredApp),
}

#[derive(Debug, PartialEq, Eq)]
pub struct Catalog {
    pub apps: Vec<AppState>,
}

#[derive(Debug, thiserror::Error)]
pub enum CatalogError {
    #[error(transparent)]
    TargetPath(#[from] TargetPathError),
    #[error(transparent)]
    Chezmoi(#[from] ChezmoiError),
}

impl Catalog {
    pub fn load(
        registry: &Registry,
        home: &Path,
        chezmoi: &dyn Chezmoi,
    ) -> Result<Self, CatalogError> {
        let mut apps = Vec::with_capacity(registry.apps.len());

        for app in &registry.apps {
            if app.ignore {
                continue;
            }
            let target = NormalizedTarget::from_registry_path(&app.target, home)?;
            if !chezmoi.is_managed(&target)? {
                apps.push(AppState::Unmanaged(app.clone()));
                continue;
            }

            apps.push(AppState::Managed(ManagedApp {
                app: app.clone(),
                source: chezmoi.source_path(&target)?,
                status: chezmoi.status(&target)?,
                target,
            }));
        }

        Ok(Self { apps })
    }
}
