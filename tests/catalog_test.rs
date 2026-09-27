use std::path::{Path, PathBuf};

use lazyconfig::{
    catalog::{AppState, Catalog},
    chezmoi::{Chezmoi, ChezmoiError, FileStatus},
    domain::NormalizedTarget,
    registry::Registry,
};

struct FakeChezmoi {
    managed: bool,
}

impl Chezmoi for FakeChezmoi {
    fn is_managed(&self, _: &NormalizedTarget) -> Result<bool, ChezmoiError> {
        Ok(self.managed)
    }

    fn source_path(&self, _: &NormalizedTarget) -> Result<PathBuf, ChezmoiError> {
        Ok(PathBuf::from("/source/dot_config/nvim"))
    }

    fn status(&self, _: &NormalizedTarget) -> Result<FileStatus, ChezmoiError> {
        Ok(FileStatus::clean())
    }

    fn apply(&self, _: &NormalizedTarget) -> Result<(), ChezmoiError> {
        Ok(())
    }
}

struct NeverCalledChezmoi;

impl Chezmoi for NeverCalledChezmoi {
    fn is_managed(&self, _: &NormalizedTarget) -> Result<bool, ChezmoiError> {
        panic!("ignored apps must not call ChezMoi")
    }

    fn source_path(&self, _: &NormalizedTarget) -> Result<PathBuf, ChezmoiError> {
        panic!("ignored apps must not resolve a source path")
    }

    fn status(&self, _: &NormalizedTarget) -> Result<FileStatus, ChezmoiError> {
        panic!("ignored apps must not check status")
    }

    fn apply(&self, _: &NormalizedTarget) -> Result<(), ChezmoiError> {
        panic!("ignored apps must not apply")
    }
}

fn registry_with_nvim() -> Registry {
    Registry::parse(
        r#"
[[app]]
id = "nvim"
label = "Neovim"
target = "~/.config/nvim"
adapter = "raw"
"#,
    )
    .unwrap()
}

#[test]
fn managed_application_has_a_resolved_source_and_status() {
    let catalog = Catalog::load(
        &registry_with_nvim(),
        Path::new("/Users/tester"),
        &FakeChezmoi { managed: true },
    )
    .unwrap();

    let AppState::Managed(app) = &catalog.apps[0] else {
        panic!("expected a managed application");
    };

    assert_eq!(
        app.target.as_path(),
        Path::new("/Users/tester/.config/nvim")
    );
    assert_eq!(app.source, PathBuf::from("/source/dot_config/nvim"));
    assert_eq!(app.status, FileStatus::clean());
}

#[test]
fn unmanaged_application_is_reported_without_a_source_path() {
    let catalog = Catalog::load(
        &registry_with_nvim(),
        Path::new("/Users/tester"),
        &FakeChezmoi { managed: false },
    )
    .unwrap();

    let AppState::Unmanaged(app) = &catalog.apps[0] else {
        panic!("expected an unmanaged application");
    };

    assert_eq!(app.id, "nvim");
}

fn ignored_registry() -> Registry {
    Registry::parse(
        r#"
[[app]]
id = "nvim"
label = "Neovim"
target = "~/.config/nvim"
adapter = "raw"
ignore = true
"#,
    )
    .unwrap()
}
#[test]
fn ignored_application_is_not_checked_by_chezmoi() {
    let catalog = Catalog::load(
        &ignored_registry(),
        Path::new("/Users/tester"),
        &NeverCalledChezmoi,
    )
    .unwrap();

    assert!(catalog.apps.is_empty());
}
