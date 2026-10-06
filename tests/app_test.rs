use std::path::{Path, PathBuf};

use lazyconfig::{
    adapter::FieldSet,
    app::{Action, App, Focus},
    catalog::{AppState, Catalog, ManagedApp},
    chezmoi::FileStatus,
    domain::{Adapter, NormalizedTarget, RegisteredApp},
    field::{Direction, EditableField, FieldValue},
    kitty,
};

fn managed_app(id: &str, source: &str) -> AppState {
    AppState::Managed(ManagedApp {
        app: RegisteredApp {
            id: id.into(),
            label: id.into(),
            target: format!("~/.config/{id}"),
            adapter: Adapter::Raw,
            ignore: false,
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
                ignore: false,
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

fn kitty_app() -> App {
    let text = "font_size 15.0\nbackground_opacity 0.95\n";
    let mut app = App::new(Catalog {
        apps: vec![
            managed_app("kitty", "/source/kitty"),
            managed_app("nvim", "/source/nvim"),
        ],
    });
    app.set_field_set(Some(FieldSet {
        adapter: Adapter::Kitty,
        source: PathBuf::from("/source/kitty/kitty.conf"),
        target: NormalizedTarget::from_registry_path("~/.config/kitty", Path::new("/Users/tester"))
            .unwrap(),
        text: text.into(),
        fields: kitty::fields(text, &[]).unwrap(),
    }));
    app
}

#[test]
fn tab_moves_focus_to_fields_and_selection_then_moves_between_fields() {
    let mut app = kitty_app();

    app.toggle_focus();
    app.select_next();

    assert_eq!(app.focus(), Focus::Fields);
    assert_eq!(app.selected_field(), 1);
    assert_eq!(app.selected_app().unwrap().app.id, "kitty");
}

#[test]
fn stepping_a_field_prepares_a_change_and_keeps_the_original_text() {
    let mut app = kitty_app();
    app.toggle_focus();

    app.step_field(Direction::Up);

    assert_eq!(app.fields()[0].value, FieldValue::Number(16.0));
    assert!(app.has_unsaved_change());
    let change = app.prepared_change().unwrap().unwrap();
    assert_eq!(change.old_text, "font_size 15.0\nbackground_opacity 0.95\n");
    assert_eq!(change.new_text, "font_size 16.0\nbackground_opacity 0.95\n");
    assert_eq!(change.source, PathBuf::from("/source/kitty/kitty.conf"));
}

#[test]
fn an_unsaved_change_blocks_config_selection_until_it_is_discarded() {
    let mut app = kitty_app();
    app.toggle_focus();
    app.step_field(Direction::Up);
    app.toggle_focus();

    app.select_next();
    assert_eq!(app.selected_app().unwrap().app.id, "kitty");

    app.discard_change();
    assert_eq!(
        app.fields()[0],
        EditableField::from_number("font_size", "font_size", 15.0).with_range(1.0, 1.0, 200.0)
    );
    app.select_next();
    assert_eq!(app.selected_app().unwrap().app.id, "nvim");
}
