use std::{fs, path::Path};

use lazyconfig::{
    domain::{Adapter, NormalizedTarget, RegisteredApp},
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
fn accepts_an_empty_registry_for_first_time_discovery() {
    let registry = Registry::parse("").unwrap();

    assert!(registry.apps.is_empty());
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

#[test]
fn appends_discovered_applications_to_the_registry_source() {
    let path = std::env::temp_dir().join(format!(
        "lazyconfig-registry-append-{}.toml",
        std::process::id()
    ));
    fs::write(
        &path,
        r#"
[[app]]
id = "starship"
label = "Starship"
target = "~/.config/starship.toml"
adapter = "starship"
"#,
    )
    .unwrap();

    Registry::append(
        &path,
        &[RegisteredApp {
            id: "nvim".into(),
            label: "Neovim".into(),
            target: "~/.config/nvim".into(),
            adapter: Adapter::Raw,
            ignore: false,
        }],
    )
    .unwrap();

    let registry = Registry::load(&path).unwrap();
    fs::remove_file(&path).unwrap();
    assert_eq!(
        registry
            .apps
            .iter()
            .map(|app| app.id.as_str())
            .collect::<Vec<_>>(),
        ["starship", "nvim"]
    );
}

#[test]
fn defaults_ignore_to_false() {
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

    assert!(!registry.apps[0].ignore);
}

#[test]
fn parses_ignore_as_true() {
    let registry = Registry::parse(
        r#"
  [[app]]
  id = "sketchybar"
  label = "sketchybar"
  target = "~/.config/sketchybar"
  adapter = "raw"
  ignore = true
  "#,
    )
    .unwrap();

    assert!(registry.apps[0].ignore);
}
