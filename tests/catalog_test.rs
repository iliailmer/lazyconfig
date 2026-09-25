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
