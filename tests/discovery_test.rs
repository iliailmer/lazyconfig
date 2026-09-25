use std::{
    cell::RefCell,
    collections::HashSet,
    fs,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use lazyconfig::{
    chezmoi::{Chezmoi, ChezmoiError, FileStatus},
    discovery::FffDiscovery,
    domain::NormalizedTarget,
    registry::Registry,
};

struct ManagedOnlyChezmoi {
    targets: HashSet<PathBuf>,
}

struct SyncChezmoi {
    targets: HashSet<PathBuf>,
    source: PathBuf,
    applied: RefCell<Vec<PathBuf>>,
}

impl Chezmoi for SyncChezmoi {
    fn is_managed(&self, target: &NormalizedTarget) -> Result<bool, ChezmoiError> {
        Ok(self.targets.contains(target.as_path()))
    }

    fn source_path(&self, _: &NormalizedTarget) -> Result<PathBuf, ChezmoiError> {
        Ok(self.source.clone())
    }

    fn status(&self, _: &NormalizedTarget) -> Result<FileStatus, ChezmoiError> {
        unreachable!()
    }

    fn apply(&self, target: &NormalizedTarget) -> Result<(), ChezmoiError> {
        self.applied
            .borrow_mut()
            .push(target.as_path().to_path_buf());
        Ok(())
    }
}

impl Chezmoi for ManagedOnlyChezmoi {
    fn is_managed(&self, target: &NormalizedTarget) -> Result<bool, ChezmoiError> {
        Ok(self.targets.contains(target.as_path()))
    }

    fn source_path(&self, _: &NormalizedTarget) -> Result<PathBuf, ChezmoiError> {
        unreachable!()
    }

    fn status(&self, _: &NormalizedTarget) -> Result<FileStatus, ChezmoiError> {
        unreachable!()
    }

    fn apply(&self, _: &NormalizedTarget) -> Result<(), ChezmoiError> {
        unreachable!()
    }
}

struct TestHome(PathBuf);

impl TestHome {
    fn new() -> Self {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("lazyconfig-discovery-{suffix}"));
        fs::create_dir_all(&path).unwrap();
        Self(path)
    }
}

impl Drop for TestHome {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}

#[test]
fn finds_known_configuration_locations_from_the_fff_index() {
    let home = TestHome::new();
    fs::create_dir_all(home.0.join(".config/nvim")).unwrap();
    fs::create_dir_all(home.0.join(".config/kitty")).unwrap();
    fs::write(home.0.join(".config/nvim/init.lua"), "return {}\n").unwrap();
    fs::write(home.0.join(".config/kitty/kitty.conf"), "font_size 14\n").unwrap();
    fs::write(
        home.0.join(".config/starship.toml"),
        "add_newline = false\n",
    )
    .unwrap();

    let configs = FffDiscovery::find(&home.0).unwrap();
    let ids = configs
        .iter()
        .map(|config| config.id.as_str())
        .collect::<Vec<_>>();

    assert_eq!(ids, ["nvim", "kitty", "starship"]);
}

#[test]
fn returns_only_new_chezmoi_managed_configurations() {
    let home = TestHome::new();
    fs::create_dir_all(home.0.join(".config/nvim")).unwrap();
    fs::create_dir_all(home.0.join(".config/kitty")).unwrap();
    fs::write(home.0.join(".config/nvim/init.lua"), "return {}\n").unwrap();
    fs::write(home.0.join(".config/kitty/kitty.conf"), "font_size 14\n").unwrap();
    let registry = Registry::parse(
        r#"
[[app]]
id = "starship"
label = "Starship"
target = "~/.config/starship.toml"
adapter = "starship"
"#,
    )
    .unwrap();
    let chezmoi = ManagedOnlyChezmoi {
        targets: HashSet::from([home.0.join(".config/nvim")]),
    };

    let configs = FffDiscovery::find_managed(&home.0, &registry, &chezmoi).unwrap();

    assert_eq!(configs.len(), 1);
    assert_eq!(configs[0].id, "nvim");
}

#[test]
fn returns_a_new_managed_top_level_configuration() {
    let home = TestHome::new();
    fs::create_dir_all(home.0.join(".config/lazygit")).unwrap();
    fs::write(
        home.0.join(".config/lazygit/config.yml"),
        "gui:\n  theme:\n    activeBorderColor:\n      - blue\n",
    )
    .unwrap();
    let registry = Registry::parse("").unwrap();
    let chezmoi = ManagedOnlyChezmoi {
        targets: HashSet::from([home.0.join(".config/lazygit")]),
    };

    let configs = FffDiscovery::find_managed(&home.0, &registry, &chezmoi).unwrap();

    assert_eq!(configs.len(), 1);
    assert_eq!(configs[0].id, "lazygit");
    assert_eq!(configs[0].target, "~/.config/lazygit");
}

#[test]
fn adds_discovered_applications_to_the_chezmoi_registry_source() {
    let home = TestHome::new();
    fs::create_dir_all(home.0.join(".config/nvim")).unwrap();
    fs::create_dir_all(home.0.join("source")).unwrap();
    fs::write(home.0.join(".config/nvim/init.lua"), "return {}\n").unwrap();
    let registry_source = home.0.join("source/lazyconfig.toml");
    fs::write(
        &registry_source,
        r#"
[[app]]
id = "starship"
label = "Starship"
target = "~/.config/starship.toml"
adapter = "starship"
"#,
    )
    .unwrap();
    let registry = Registry::load(&registry_source).unwrap();
    let registry_target = home.0.join(".config/lazyconfig/config.toml");
    let chezmoi = SyncChezmoi {
        targets: HashSet::from([home.0.join(".config/nvim"), registry_target.clone()]),
        source: registry_source.clone(),
        applied: RefCell::new(Vec::new()),
    };
    let registry_target =
        NormalizedTarget::from_registry_path(registry_target.to_str().unwrap(), &home.0).unwrap();

    let added = FffDiscovery::sync(&home.0, &registry_target, &registry, &chezmoi).unwrap();

    assert_eq!(
        added.iter().map(|app| app.id.as_str()).collect::<Vec<_>>(),
        ["nvim"]
    );
    assert_eq!(
        Registry::load(&registry_source)
            .unwrap()
            .apps
            .iter()
            .map(|app| app.id.as_str())
            .collect::<Vec<_>>(),
        ["starship", "nvim"]
    );
    assert_eq!(
        chezmoi.applied.borrow().as_slice(),
        &[registry_target.as_path().to_path_buf()]
    );
}

#[test]
fn refuses_to_update_an_unmanaged_lazyconfig_registry() {
    let home = TestHome::new();
    fs::create_dir_all(home.0.join(".config/nvim")).unwrap();
    fs::create_dir_all(home.0.join("source")).unwrap();
    fs::write(home.0.join(".config/nvim/init.lua"), "return {}\n").unwrap();
    let registry_source = home.0.join("source/lazyconfig.toml");
    fs::write(
        &registry_source,
        r#"
[[app]]
id = "starship"
label = "Starship"
target = "~/.config/starship.toml"
adapter = "starship"
"#,
    )
    .unwrap();
    let registry = Registry::load(&registry_source).unwrap();
    let registry_target = NormalizedTarget::from_registry_path(
        home.0
            .join(".config/lazyconfig/config.toml")
            .to_str()
            .unwrap(),
        &home.0,
    )
    .unwrap();
    let chezmoi = SyncChezmoi {
        targets: HashSet::from([home.0.join(".config/nvim")]),
        source: registry_source,
        applied: RefCell::new(Vec::new()),
    };

    let error = FffDiscovery::sync(&home.0, &registry_target, &registry, &chezmoi).unwrap_err();

    assert_eq!(
        error.to_string(),
        "LazyConfig registry is not managed by ChezMoi"
    );
}
