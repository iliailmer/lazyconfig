use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Modifier, Style},
    text::Line,
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
};

use crate::{app::App, catalog::ManagedApp};

pub fn render(frame: &mut Frame, app: &App) {
    let [apps_area, details_area] = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(32), Constraint::Percentage(68)])
        .areas(frame.area());
    let selected_id = app.selected_app().map(|app| app.app.id.as_str());
    let apps = app
        .managed_apps()
        .map(|managed| {
            let marker = if Some(managed.app.id.as_str()) == selected_id {
                "> "
            } else {
                "  "
            };
            let style = if Some(managed.app.id.as_str()) == selected_id {
                Style::default().add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            ListItem::new(Line::from(format!("{marker}{}", managed.app.label))).style(style)
        })
        .collect::<Vec<_>>();

    frame.render_widget(
        List::new(apps).block(Block::default().borders(Borders::ALL).title(" Configs ")),
        apps_area,
    );
    frame.render_widget(
        Paragraph::new(details(app))
            .block(Block::default().borders(Borders::ALL).title(" Details "))
            .wrap(Wrap { trim: false }),
        details_area,
    );
}

fn details(app: &App) -> String {
    let mut text = match app.selected_app() {
        Some(app) => managed_details(app),
        None => "No configured application is managed by ChezMoi.\n\n[d] Discover  [q] Quit".into(),
    };
    let problems = app
        .unmanaged_apps()
        .map(|app| app.label.as_str())
        .collect::<Vec<_>>();

    if !problems.is_empty() {
        text.push_str("\n\nNot managed by ChezMoi: ");
        text.push_str(&problems.join(", "));
        text.push_str(". Track these targets before using LazyConfig.");
    }

    if let Some(notice) = app.notice() {
        text.push_str("\n\n");
        text.push_str(notice);
    }

    text
}

fn managed_details(app: &ManagedApp) -> String {
    let status = match (app.status.destination_changed, app.status.target_changed) {
        (false, false) => "clean",
        (true, false) => "destination changed",
        (false, true) => "apply pending",
        (true, true) => "destination changed; apply pending",
    };

    format!(
        "{}\n\nTarget: {}\nSource: {}\nStatus: {}\n\n[e] Edit source  [a] Apply  [d] Discover  [r] Refresh  [q] Quit",
        app.app.label,
        app.target.as_path().display(),
        app.source.display(),
        status
    )
}
