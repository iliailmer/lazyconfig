use std::{fs, path::Path};

use lazyconfig::{
    domain::{Adapter, NormalizedTarget, RegisteredApp},
    registry::Registry,
};

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
fn rejects_a_relative_target() {
    let error = NormalizedTarget::from_registry_path(".config/nvim", Path::new("/Users/tester"))
        .unwrap_err();

    assert_eq!(
        error.to_string(),
        "target must be absolute or start with ~/"
    );
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
