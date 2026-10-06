use std::{
    collections::{BTreeSet, HashSet},
    path::Path,
};

use fff_search::{FFFMode, FilePicker, FilePickerOptions};
use thiserror::Error;

use crate::{
    chezmoi::{Chezmoi, ChezmoiError},
    domain::{Adapter, NormalizedTarget, RegisteredApp, TargetPathError},
    registry::{Registry, RegistryError},
};

pub struct FffDiscovery;

#[derive(Debug, Error)]
pub enum DiscoveryError {
    #[error(transparent)]
    Search(#[from] fff_search::Error),
    #[error(transparent)]
    TargetPath(#[from] TargetPathError),
    #[error(transparent)]
    Chezmoi(#[from] ChezmoiError),
    #[error(transparent)]
    Registry(#[from] RegistryError),
    #[error("LazyConfig registry is not managed by ChezMoi")]
    UnmanagedRegistry,
}

impl FffDiscovery {
    pub fn find(home: &Path) -> Result<Vec<RegisteredApp>, DiscoveryError> {
        let config_root = home.join(".config");
        if !config_root.exists() {
            return Ok(Vec::new());
        }

        let mut picker = FilePicker::new(FilePickerOptions {
            base_path: config_root.display().to_string(),
            mode: FFFMode::Ai,
            watch: false,
            ..Default::default()
        })?;
        picker.collect_files()?;

        let files = picker
            .get_files()
            .iter()
            .map(|file| file.relative_path(&picker))
            .collect::<BTreeSet<_>>();
        let directories = picker
            .get_dirs()
            .iter()
            .map(|directory| directory.relative_path(&picker))
            .collect::<BTreeSet<_>>();
        let mut top_level_entries = files
            .iter()
            .chain(&directories)
            .filter_map(|path| path.split('/').next())
            .filter(|entry| !entry.is_empty() && *entry != "lazyconfig")
            .collect::<BTreeSet<_>>();

        let mut configs = Vec::new();
        for entry in ["nvim", "kitty", "starship.toml"] {
            if top_level_entries.remove(entry) {
                configs.push(discovered_app(entry));
            }
        }
        configs.extend(top_level_entries.into_iter().map(discovered_app));

        Ok(configs)
    }

    pub fn find_managed(
        home: &Path,
        registry: &Registry,
        chezmoi: &dyn Chezmoi,
    ) -> Result<Vec<RegisteredApp>, DiscoveryError> {
        let registered_ids = registry
            .apps
            .iter()
            .map(|app| app.id.as_str())
            .collect::<HashSet<_>>();
        let mut managed = Vec::new();

        for app in Self::find(home)? {
            if registered_ids.contains(app.id.as_str()) {
                continue;
            }

            let target = NormalizedTarget::from_registry_path(&app.target, home)?;
            if chezmoi.is_managed(&target)? {
                managed.push(app);
            }
        }

        Ok(managed)
    }

    pub fn sync(
        home: &Path,
        registry_target: &NormalizedTarget,
        registry: &Registry,
        chezmoi: &dyn Chezmoi,
    ) -> Result<Vec<RegisteredApp>, DiscoveryError> {
        if !chezmoi.is_managed(registry_target)? {
            return Err(DiscoveryError::UnmanagedRegistry);
        }

        let apps = Self::find_managed(home, registry, chezmoi)?;
        if apps.is_empty() {
            return Ok(apps);
        }

        let registry_source = chezmoi.source_path(registry_target)?;
        Registry::append(&registry_source, &apps)?;
        chezmoi.apply(registry_target)?;

        Ok(apps)
    }
}

fn discovered_app(entry: &str) -> RegisteredApp {
    match entry {
        "nvim" => RegisteredApp {
            id: "nvim".into(),
            label: "Neovim".into(),
            target: "~/.config/nvim".into(),
            adapter: Adapter::Raw,
            ignore: false,
        },
        "kitty" => RegisteredApp {
            id: "kitty".into(),
            label: "Kitty".into(),
            target: "~/.config/kitty".into(),
            adapter: Adapter::Kitty,
            ignore: false,
        },
        "starship.toml" => RegisteredApp {
            id: "starship".into(),
            label: "Starship".into(),
            target: "~/.config/starship.toml".into(),
            adapter: Adapter::Starship,
            ignore: false,
        },
        entry => RegisteredApp {
            id: entry.into(),
            label: entry.into(),
            target: format!("~/.config/{entry}"),
            adapter: Adapter::Raw,
            ignore: false,
        },
    }
}
