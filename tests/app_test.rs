use std::path::{Path, PathBuf};

use lazyconfig::{
    app::{Action, App},
    catalog::{AppState, Catalog, ManagedApp},
    chezmoi::FileStatus,
    domain::{Adapter, NormalizedTarget, RegisteredApp},
};

fn managed_app(id: &str, source: &str) -> AppState {
    AppState::Managed(ManagedApp {
        app: RegisteredApp {
            id: id.into(),
            label: id.into(),
            target: format!("~/.config/{id}"),
            adapter: Adapter::Raw,
        },
        target: NormalizedTarget::from_registry_path(
            &format!("~/.config/{id}"),
            Path::new("/Users/tester"),
        )
        .unwrap(),
        source: PathBuf::from(source),
        status: FileStatus::clean(),
    })
}

#[test]
fn moves_selection_between_managed_applications_only() {
    let mut app = App::new(Catalog {
        apps: vec![
            managed_app("nvim", "/source/nvim"),
            AppState::Unmanaged(RegisteredApp {
                id: "ghostty".into(),
                label: "Ghostty".into(),
                target: "~/.config/ghostty".into(),
                adapter: Adapter::Raw,
            }),
            managed_app("starship", "/source/starship.toml"),
        ],
    });

    app.select_next();

    assert_eq!(app.selected_app().unwrap().app.id, "starship");
}

#[test]
fn reverse_selection_wraps_to_the_last_managed_application() {
    let mut app = App::new(Catalog {
        apps: vec![
            managed_app("nvim", "/source/nvim"),
            managed_app("starship", "/source/starship.toml"),
        ],
    });

    app.select_previous();

    assert_eq!(app.selected_app().unwrap().app.id, "starship");
}

#[test]
fn edit_action_uses_the_selected_managed_source_path() {
    let app = App::new(Catalog {
        apps: vec![managed_app("nvim", "/source/nvim/init.lua")],
    });

    assert_eq!(
        app.edit_action(),
        Action::Edit(PathBuf::from("/source/nvim/init.lua"))
    );
}

#[test]
fn apply_action_uses_the_selected_managed_target() {
    let app = App::new(Catalog {
        apps: vec![managed_app("nvim", "/source/nvim/init.lua")],
    });

    assert_eq!(
        app.apply_action(),
        Action::Apply(
            NormalizedTarget::from_registry_path("~/.config/nvim", Path::new("/Users/tester"),)
                .unwrap()
        )
    );
}

#[test]
fn edit_action_is_unavailable_when_no_application_is_managed() {
    let app = App::new(Catalog {
        apps: vec![AppState::Unmanaged(RegisteredApp {
            id: "ghostty".into(),
            label: "Ghostty".into(),
            target: "~/.config/ghostty".into(),
            adapter: Adapter::Raw,
        })],
    });

    assert_eq!(app.edit_action(), Action::None);
}

#[test]
fn replacing_the_catalog_resets_selection_to_the_first_managed_application() {
    let mut app = App::new(Catalog {
        apps: vec![
            managed_app("nvim", "/source/nvim"),
            managed_app("starship", "/source/starship.toml"),
        ],
    });
    app.select_next();

    app.replace_catalog(Catalog {
        apps: vec![managed_app("kitty", "/source/kitty.conf")],
    });

    assert_eq!(app.selected_app().unwrap().app.id, "kitty");
}
