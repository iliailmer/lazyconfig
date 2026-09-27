use std::path::{Path, PathBuf};

use lazyconfig::{
    app::App,
    catalog::{AppState, Catalog, ManagedApp},
    chezmoi::FileStatus,
    domain::{Adapter, NormalizedTarget, RegisteredApp},
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
fn renders_discovery_help_when_no_application_is_managed() {
    let app = App::new(Catalog { apps: Vec::new() });
    let backend = TestBackend::new(100, 20);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal.draw(|frame| render(frame, &app)).unwrap();

    let buffer = terminal.backend().buffer();
    let rendered: String = (0..buffer.area.height)
        .flat_map(|y| (0..buffer.area.width).map(move |x| buffer.cell((x, y)).unwrap().symbol()))
        .collect();

    assert!(rendered.contains("[d] Discover"));
}
