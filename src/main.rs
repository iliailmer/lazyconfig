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
    app::{Action, App},
    catalog::Catalog,
    chezmoi::{Chezmoi, CliChezMoi, SystemCommandRunner},
    discovery::FffDiscovery,
    domain::NormalizedTarget,
    editor::{SourceEditor, SystemEditorRunner},
    registry::Registry,
    ui,
};
use ratatui::{Terminal, backend::CrosstermBackend};

type AppTerminal = Terminal<CrosstermBackend<io::Stdout>>;

fn main() -> Result<(), Box<dyn Error>> {
    let home = home_directory()?;
    let registry_path = home.join(".config/lazyconfig/config.toml");

    if !registry_path.exists() {
        println!(
            "No LazyConfig registry at {}. Create it, then add ChezMoi-managed applications.",
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
    let catalog = Catalog::load(&registry, &home, &chezmoi)?;
    let mut app = App::new(catalog);
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

        match key.code {
            KeyCode::Char('q') => return Ok(()),
            KeyCode::Char('j') | KeyCode::Down => app.select_next(),
            KeyCode::Char('k') | KeyCode::Up => app.select_previous(),
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
            KeyCode::Char('a') => {
                if let Action::Apply(target) = app.apply_action() {
                    chezmoi.apply(&target)?;
                    refresh_catalog(app, registry, home, chezmoi)?;
                }
            }
            KeyCode::Char('e') => {
                if let Action::Edit(source) = app.edit_action() {
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

fn refresh_catalog(
    app: &mut App,
    registry: &Registry,
    home: &Path,
    chezmoi: &CliChezMoi<SystemCommandRunner>,
) -> Result<(), Box<dyn Error>> {
    app.replace_catalog(Catalog::load(registry, home, chezmoi)?);
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
