use std::{fs, path::Path};

use lazyconfig::{
    domain::NormalizedTarget,
    registry::{Registry, RegistryError},
};

#[test]
fn parses_a_registered_raw_application() {
    let registry = Registry::parse(
        r#"
[[app]]
id = "nvim"
label = "Neovim"
target = "~/.config/nvim"
adapter = "raw"
"#,
    )
    .unwrap();

    assert_eq!(registry.apps.len(), 1);
    assert_eq!(registry.apps[0].id, "nvim");
    assert_eq!(registry.apps[0].label, "Neovim");
    assert_eq!(registry.apps[0].target, "~/.config/nvim");
}

#[test]
fn rejects_duplicate_application_ids() {
    let error = Registry::parse(
        r#"
[[app]]
id = "nvim"
label = "Neovim"
target = "~/.config/nvim"
adapter = "raw"

[[app]]
id = "nvim"
label = "Neovim backup"
target = "~/.config/nvim-backup"
adapter = "raw"
"#,
    )
    .unwrap_err();

    assert_eq!(error.to_string(), "duplicate application id: nvim");
}

#[test]
fn expands_a_home_relative_target_before_invoking_chezmoi() {
    let target =
        NormalizedTarget::from_registry_path("~/.config/nvim", Path::new("/Users/tester")).unwrap();

    assert_eq!(target.as_path(), Path::new("/Users/tester/.config/nvim"));
}

#[test]
fn retains_an_absolute_target() {
    let target = NormalizedTarget::from_registry_path(
        "/Users/tester/.config/starship.toml",
        Path::new("/Users/tester"),
    )
    .unwrap();

    assert_eq!(
        target.as_path(),
        Path::new("/Users/tester/.config/starship.toml")
    );
}

#[test]
fn rejects_a_relative_target() {
    let error = NormalizedTarget::from_registry_path(".config/nvim", Path::new("/Users/tester"))
        .unwrap_err();

    assert_eq!(
        error.to_string(),
        "target must be absolute or start with ~/"
    );
}

#[test]
fn loads_registered_applications_from_a_toml_file() {
    let path =
        std::env::temp_dir().join(format!("lazyconfig-registry-{}.toml", std::process::id()));
    fs::write(
        &path,
        r#"
[[app]]
id = "kitty"
label = "Kitty"
target = "~/.config/kitty"
adapter = "raw"
"#,
    )
    .unwrap();

    let registry = Registry::load(&path).unwrap();

    fs::remove_file(&path).unwrap();
    assert_eq!(registry.apps[0].id, "kitty");
}

#[test]
fn reports_a_missing_registry_file() {
    let path = std::env::temp_dir().join(format!(
        "lazyconfig-missing-registry-{}.toml",
        std::process::id()
    ));

    let error = Registry::load(&path).unwrap_err();

    assert!(matches!(error, RegistryError::Read { .. }));
}
