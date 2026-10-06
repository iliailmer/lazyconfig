use std::fs;

use lazyconfig::setup::{self, InitOutcome};

#[test]
fn init_creates_the_example_registry_and_never_overwrites_it() {
    let directory = std::env::temp_dir().join(format!("lazyconfig-setup-{}", std::process::id()));
    let registry = directory.join("lazyconfig/config.toml");

    assert_eq!(setup::init(&registry).unwrap(), InitOutcome::Created);
    fs::write(&registry, "# mine\n").unwrap();
    assert_eq!(setup::init(&registry).unwrap(), InitOutcome::AlreadyExists);

    let kept = fs::read_to_string(&registry).unwrap();
    fs::remove_dir_all(&directory).unwrap();
    assert_eq!(kept, "# mine\n");
    assert_eq!(
        setup::track_command(&registry),
        format!("chezmoi add {}", registry.display())
    );
}
