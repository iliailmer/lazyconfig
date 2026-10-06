use std::{
    env,
    error::Error,
    io,
    path::{Path, PathBuf},
};

use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use lazyconfig::{
    adapter,
    app::{Action, App, View},
    catalog::Catalog,
    chezmoi::{Chezmoi, CliChezMoi, SystemCommandRunner},
    discovery::FffDiscovery,
    domain::NormalizedTarget,
    editor::{SourceEditor, SystemEditorRunner},
    field::Direction,
    registry::Registry,
    setup::{self, InitOutcome},
    ui,
};
use ratatui::{Terminal, backend::CrosstermBackend};

type AppTerminal = Terminal<CrosstermBackend<io::Stdout>>;

fn main() -> Result<(), Box<dyn Error>> {
    let home = home_directory()?;
    let registry_path = home.join(".config/lazyconfig/config.toml");

    match env::args().nth(1).as_deref() {
        None => {}
        Some("init") => return init(&registry_path),
        Some(argument) => return Err(format!("unknown argument: {argument}").into()),
    }

    if !registry_path.exists() {
        println!(
            "No LazyConfig registry at {}. Run `lazyconfig init` to create it.",
            registry_path.display()
        );
        return Ok(());
    }

    let mut registry = Registry::load(&registry_path)?;
    let registry_target = NormalizedTarget::from_registry_path(
        registry_path
            .to_str()
            .ok_or("registry path is not valid UTF-8")?,
        &home,
    )?;
    let chezmoi = CliChezMoi::new(SystemCommandRunner);
    if !chezmoi.is_managed(&registry_target)? {
        print_track_instruction(&registry_path);
        return Ok(());
    }

    let catalog = Catalog::load(&registry, &home, &chezmoi)?;
    let mut app = App::new(catalog);
    reload_fields(&mut app, &chezmoi);
    let mut terminal = enter_terminal()?;
    let result = event_loop(
        &mut terminal,
        &mut registry,
        &registry_path,
        &registry_target,
        &home,
        &chezmoi,
        &mut app,
    );

    restore_terminal(&mut terminal)?;
    result
}

fn event_loop(
    terminal: &mut AppTerminal,
    registry: &mut Registry,
    registry_path: &Path,
    registry_target: &NormalizedTarget,
    home: &Path,
    chezmoi: &CliChezMoi<SystemCommandRunner>,
    app: &mut App,
) -> Result<(), Box<dyn Error>> {
    loop {
        terminal.draw(|frame| ui::render(frame, app))?;

        let Event::Key(key) = event::read()? else {
            continue;
        };

        app.clear_notice();

        match key.code {
            KeyCode::Char('q') => return Ok(()),
            KeyCode::Char('j') | KeyCode::Down => {
                app.select_next();
                reload_fields_after_move(app, chezmoi);
            }
            KeyCode::Char('k') | KeyCode::Up => {
                app.select_previous();
                reload_fields_after_move(app, chezmoi);
            }
            KeyCode::Tab => app.toggle_focus(),
            KeyCode::Char('+') | KeyCode::Char('=') => app.step_field(Direction::Up),
            KeyCode::Char('-') => app.step_field(Direction::Down),
            KeyCode::Char('p') => {
                if app.has_unsaved_change() {
                    app.toggle_source_preview();
                } else {
                    app.set_notice("No unsaved change.".into());
                }
            }
            KeyCode::Char('w') => write_source(app, registry, home, chezmoi)?,
            KeyCode::Esc => {
                if matches!(app.view(), View::Diff { .. }) {
                    app.close_view();
                } else {
                    app.discard_change();
                }
            }
            KeyCode::Char('r') => refresh_catalog(app, registry, home, chezmoi)?,
            KeyCode::Char('d') => {
                match FffDiscovery::sync(home, registry_target, registry, chezmoi) {
                    Ok(apps) => {
                        if apps.is_empty() {
                            app.set_notice("Discovery found no new managed configurations.".into());
                        } else {
                            *registry = Registry::load(registry_path)?;
                            refresh_catalog(app, registry, home, chezmoi)?;
                            let labels = apps
                                .iter()
                                .map(|app| app.label.as_str())
                                .collect::<Vec<_>>();
                            app.set_notice(format!("Discovery added: {}", labels.join(", ")));
                        }
                    }
                    Err(error) => app.set_notice(format!("Discovery failed: {error}")),
                }
            }
            KeyCode::Char('a') => apply(app, registry, home, chezmoi)?,
            KeyCode::Char('e') => {
                if app.has_unsaved_change() {
                    app.set_notice(
                        "Unsaved change. Press [w] to write or [Esc] to discard.".into(),
                    );
                } else if let Action::Edit(source) = app.edit_action() {
                    suspend_terminal(terminal)?;
                    let editor = env::var("EDITOR").unwrap_or_else(|_| "nvim".into());
                    let edit_result = SourceEditor::new(&editor, SystemEditorRunner)
                        .and_then(|editor| editor.open(&source));
                    resume_terminal(terminal)?;
                    edit_result?;
                    refresh_catalog(app, registry, home, chezmoi)?;
                }
            }
            _ => {}
        }
    }
}

fn write_source(
    app: &mut App,
    registry: &Registry,
    home: &Path,
    chezmoi: &CliChezMoi<SystemCommandRunner>,
) -> Result<(), Box<dyn Error>> {
    let change = match app.prepared_change() {
        Ok(Some(change)) => change,
        Ok(None) => {
            app.set_notice("No unsaved change.".into());
            return Ok(());
        }
        Err(error) => {
            app.set_notice(format!("This change is not valid: {error}"));
            return Ok(());
        }
    };

    if let Err(error) = change.write() {
        app.set_notice(format!("Source was not written: {error}"));
        return Ok(());
    }

    refresh_catalog(app, registry, home, chezmoi)?;
    match chezmoi.diff(&change.target) {
        Ok(diff) if diff.trim().is_empty() => {
            app.close_view();
            app.set_notice("Source written. The target does not change.".into());
        }
        Ok(diff) => app.show_diff(change.target, diff),
        Err(error) => {
            app.close_view();
            app.set_notice(format!("Source written. Diff failed: {error}"));
        }
    }

    Ok(())
}

fn apply(
    app: &mut App,
    registry: &Registry,
    home: &Path,
    chezmoi: &CliChezMoi<SystemCommandRunner>,
) -> Result<(), Box<dyn Error>> {
    if let View::Diff { target, .. } = app.view() {
        let target = target.clone();
        match chezmoi.apply(&target) {
            Ok(()) => {
                refresh_catalog(app, registry, home, chezmoi)?;
                app.close_view();
                app.set_notice("Applied.".into());
            }
            Err(error) => app.set_notice(format!("Apply failed: {error}")),
        }
        return Ok(());
    }

    if app.has_unsaved_change() {
        app.set_notice("Write the source first. Press [w].".into());
        return Ok(());
    }

    let Some(selected) = app.selected_app() else {
        return Ok(());
    };
    if selected.status.destination_changed {
        app.set_notice(
            "The target was changed outside ChezMoi. Resolve it with `chezmoi merge` or `chezmoi apply` in a shell."
                .into(),
        );
        return Ok(());
    }

    let Action::Apply(target) = app.apply_action() else {
        return Ok(());
    };
    match chezmoi.diff(&target) {
        Ok(diff) if diff.trim().is_empty() => app.set_notice("Nothing to apply.".into()),
        Ok(diff) => app.show_diff(target, diff),
        Err(error) => app.set_notice(format!("Diff failed: {error}")),
    }

    Ok(())
}

fn reload_fields(app: &mut App, chezmoi: &dyn Chezmoi) {
    let loaded = match app.selected_app() {
        Some(selected) => adapter::load(selected, chezmoi),
        None => Ok(None),
    };

    match loaded {
        Ok(field_set) => app.set_field_set(field_set),
        Err(error) => {
            app.set_field_set(None);
            app.set_notice(format!("Fields are not available: {error}"));
        }
    }
}

fn reload_fields_after_move(app: &mut App, chezmoi: &dyn Chezmoi) {
    if !app.has_unsaved_change() && matches!(app.view(), View::Details) {
        reload_fields(app, chezmoi);
    }
}

fn init(registry_path: &Path) -> Result<(), Box<dyn Error>> {
    match setup::init(registry_path)? {
        InitOutcome::Created => println!("Created {}.", registry_path.display()),
        InitOutcome::AlreadyExists => println!("{} already exists.", registry_path.display()),
    }
    print_track_instruction(registry_path);

    Ok(())
}

fn print_track_instruction(registry_path: &Path) {
    println!(
        "LazyConfig needs ChezMoi to manage its registry. If it is not tracked yet, run:\n\n    {}\n\nThen start lazyconfig and press [d] to discover managed configurations.",
        setup::track_command(registry_path)
    );
}

fn refresh_catalog(
    app: &mut App,
    registry: &Registry,
    home: &Path,
    chezmoi: &CliChezMoi<SystemCommandRunner>,
) -> Result<(), Box<dyn Error>> {
    app.replace_catalog(Catalog::load(registry, home, chezmoi)?);
    reload_fields(app, chezmoi);
    Ok(())
}

fn home_directory() -> Result<PathBuf, Box<dyn Error>> {
    env::var_os("HOME")
        .map(PathBuf::from)
        .ok_or_else(|| "HOME is not set".into())
}

fn enter_terminal() -> io::Result<AppTerminal> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    Terminal::new(CrosstermBackend::new(stdout))
}

fn suspend_terminal(terminal: &mut AppTerminal) -> io::Result<()> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()
}

fn resume_terminal(terminal: &mut AppTerminal) -> io::Result<()> {
    enable_raw_mode()?;
    execute!(terminal.backend_mut(), EnterAlternateScreen)?;
    terminal.clear()
}

fn restore_terminal(terminal: &mut AppTerminal) -> io::Result<()> {
    suspend_terminal(terminal)
}
