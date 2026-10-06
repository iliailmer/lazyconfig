use std::path::{Path, PathBuf};

use lazyconfig::{
    adapter::FieldSet,
    app::App,
    catalog::{AppState, Catalog, ManagedApp},
    chezmoi::FileStatus,
    domain::{Adapter, NormalizedTarget, RegisteredApp},
    field::Direction,
    kitty,
    ui::render,
};
use ratatui::{Terminal, backend::TestBackend};

#[test]
fn renders_the_selected_application_and_its_source_path() {
    let mut app = App::new(Catalog {
        apps: vec![AppState::Managed(ManagedApp {
            app: RegisteredApp {
                id: "nvim".into(),
                label: "Neovim".into(),
                target: "~/.config/nvim".into(),
                adapter: Adapter::Raw,
                ignore: false,
            },
            target: NormalizedTarget::from_registry_path(
                "~/.config/nvim",
                Path::new("/Users/tester"),
            )
            .unwrap(),
            source: PathBuf::from("/source/dot_config/nvim"),
            status: FileStatus::clean(),
        })],
    });
    app.set_notice("Discovery added: Neovim".into());
    let backend = TestBackend::new(100, 20);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal.draw(|frame| render(frame, &app)).unwrap();

    let buffer = terminal.backend().buffer();
    let rendered: String = (0..buffer.area.height)
        .flat_map(|y| (0..buffer.area.width).map(move |x| buffer.cell((x, y)).unwrap().symbol()))
        .collect();

    assert!(rendered.contains("Neovim"));
    assert!(rendered.contains("/source/dot_config/nvim"));
    assert!(rendered.contains("Discovery added: Neovim"));
}

#[test]
fn renders_fields_and_marks_an_unsaved_change() {
    let target =
        NormalizedTarget::from_registry_path("~/.config/kitty", Path::new("/Users/tester"))
            .unwrap();
    let mut app = App::new(Catalog {
        apps: vec![AppState::Managed(ManagedApp {
            app: RegisteredApp {
                id: "kitty".into(),
                label: "Kitty".into(),
                target: "~/.config/kitty".into(),
                adapter: Adapter::Kitty,
                ignore: false,
            },
            target: target.clone(),
            source: PathBuf::from("/source/dot_config/kitty"),
            status: FileStatus::clean(),
        })],
    });
    app.set_field_set(Some(FieldSet {
        adapter: Adapter::Kitty,
        source: PathBuf::from("/source/dot_config/kitty/kitty.conf"),
        target,
        text: "font_size 14\n".into(),
        fields: kitty::fields("font_size 14\n", &[]).unwrap(),
    }));
    app.toggle_focus();
    app.step_field(Direction::Down);
    let backend = TestBackend::new(100, 24);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal.draw(|frame| render(frame, &app)).unwrap();

    let buffer = terminal.backend().buffer();
    let rendered: String = (0..buffer.area.height)
        .flat_map(|y| (0..buffer.area.width).map(move |x| buffer.cell((x, y)).unwrap().symbol()))
        .collect();

    assert!(rendered.contains("Fields"));
    assert!(rendered.contains("font_size"));
    assert!(rendered.contains("13.0  *"));
    assert!(rendered.contains("Unsaved change"));
}
