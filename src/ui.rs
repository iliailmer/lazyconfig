use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::Line,
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
};

use crate::{
    app::{App, Focus, View},
    catalog::ManagedApp,
};

pub fn render(frame: &mut Frame, app: &App) {
    let [apps_area, right_area] = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(32), Constraint::Percentage(68)])
        .areas(frame.area());
    let selected_id = app.selected_app().map(|app| app.app.id.as_str());
    let apps = app
        .managed_apps()
        .map(|managed| {
            list_item(
                &managed.app.label,
                Some(managed.app.id.as_str()) == selected_id,
            )
        })
        .collect::<Vec<_>>();

    frame.render_widget(
        List::new(apps).block(block(" Configs ", app.focus() == Focus::Configs)),
        apps_area,
    );

    match app.view() {
        View::Details => render_details(frame, app, right_area),
        View::SourcePreview => {
            render_text(frame, " Source change ", source_preview(app), right_area)
        }
        View::Diff { text, .. } => render_text(
            frame,
            " ChezMoi diff ",
            with_notice(app, format!("[a] Apply  [Esc] Close\n\n{text}")),
            right_area,
        ),
    }
}

fn render_details(frame: &mut Frame, app: &App, area: Rect) {
    let fields = app.fields();
    if fields.is_empty() {
        render_text(frame, " Details ", details(app), area);
        return;
    }

    let [details_area, fields_area] = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),
            Constraint::Length(fields.len() as u16 + 2),
        ])
        .areas(area);
    let items = fields
        .iter()
        .enumerate()
        .map(|(index, field)| {
            let changed = if app.is_field_changed(index) {
                "  *"
            } else {
                ""
            };
            let selected = app.focus() == Focus::Fields && index == app.selected_field();

            list_item(
                &format!("{:<24}{}{changed}", field.label, field.value),
                selected,
            )
        })
        .collect::<Vec<_>>();

    render_text(frame, " Details ", details(app), details_area);
    frame.render_widget(
        List::new(items).block(block(" Fields ", app.focus() == Focus::Fields)),
        fields_area,
    );
}

fn render_text(frame: &mut Frame, title: &'static str, text: String, area: Rect) {
    frame.render_widget(
        Paragraph::new(text)
            .block(block(title, false))
            .wrap(Wrap { trim: false }),
        area,
    );
}

fn block(title: &'static str, focused: bool) -> Block<'static> {
    let style = if focused {
        Style::default().add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };

    Block::default()
        .borders(Borders::ALL)
        .border_style(style)
        .title(title)
}

fn list_item(label: &str, selected: bool) -> ListItem<'static> {
    let (marker, style) = if selected {
        ("> ", Style::default().add_modifier(Modifier::BOLD))
    } else {
        ("  ", Style::default())
    };

    ListItem::new(Line::from(format!("{marker}{label}"))).style(style)
}

fn source_preview(app: &App) -> String {
    let text = match app.prepared_change() {
        Ok(Some(change)) => format!(
            "[w] Write source  [Esc] Discard\n\nSource: {}\n\n{}",
            change.source.display(),
            change.changed_lines().join("\n")
        ),
        Ok(None) => "No unsaved change.\n\n[Esc] Close".into(),
        Err(error) => format!("This change is not valid: {error}\n\n[Esc] Discard"),
    };

    with_notice(app, text)
}

fn details(app: &App) -> String {
    let mut text = match app.selected_app() {
        Some(app) => managed_details(app),
        None => "No configured application is managed by ChezMoi.\n\n[d] Discover  [q] Quit".into(),
    };
    let problems = app //  OPTIM: kinda looks like this should not be a repeated call
        .unmanaged_apps()
        .map(|app| app.label.as_str())
        .collect::<Vec<_>>();

    if !app.fields().is_empty() {
        text.push_str("\n[Tab] Switch pane  [+/-] Change value");
    }

    if app.has_unsaved_change() {
        text.push_str("\n\nUnsaved change  [p] Preview  [w] Write source  [Esc] Discard");
    }

    if !problems.is_empty() {
        // OPTIM: same as above
        text.push_str("\n\nNot managed by ChezMoi: ");
        text.push_str(&problems.join(", "));
        text.push_str(". Track these targets before using LazyConfig.");
    }

    with_notice(app, text)
}

fn with_notice(app: &App, mut text: String) -> String {
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
