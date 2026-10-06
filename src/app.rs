use std::path::PathBuf;

use crate::{
    adapter::{self, AdapterError, FieldSet},
    catalog::{AppState, Catalog, ManagedApp},
    change::PreparedChange,
    domain::{NormalizedTarget, RegisteredApp},
    field::{Direction, EditableField},
};

#[derive(Debug, PartialEq, Eq)]
pub enum Action {
    None,
    Edit(PathBuf),
    Apply(NormalizedTarget),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Focus {
    Configs,
    Fields,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum View {
    Details,
    SourcePreview,
    Diff {
        target: NormalizedTarget,
        text: String,
    },
}

pub struct App {
    catalog: Catalog,
    selected_index: usize,
    notice: Option<String>,
    focus: Focus,
    view: View,
    field_set: Option<FieldSet>,
    fields: Vec<EditableField>,
    selected_field: usize,
}

impl App {
    pub fn new(catalog: Catalog) -> Self {
        Self {
            catalog,
            selected_index: 0,
            notice: None,
            focus: Focus::Configs,
            view: View::Details,
            field_set: None,
            fields: Vec::new(),
            selected_field: 0,
        }
    }

    pub fn selected_app(&self) -> Option<&ManagedApp> {
        self.managed_apps().nth(self.selected_index)
    }

    pub fn replace_catalog(&mut self, catalog: Catalog) {
        let selected_id = self.selected_app().map(|app| app.app.id.clone());
        self.catalog = catalog;
        self.selected_index = selected_id
            .and_then(|id| self.managed_apps().position(|app| app.app.id == id))
            .unwrap_or(0);
    }

    pub fn set_notice(&mut self, notice: String) {
        self.notice = Some(notice);
    }

    pub fn clear_notice(&mut self) {
        self.notice = None;
    }

    pub fn notice(&self) -> Option<&str> {
        self.notice.as_deref()
    }

    pub fn managed_apps(&self) -> impl Iterator<Item = &ManagedApp> {
        self.catalog.apps.iter().filter_map(|app| match app {
            AppState::Managed(app) => Some(app),
            AppState::Unmanaged(_) => None,
        })
    }

    pub fn unmanaged_apps(&self) -> impl Iterator<Item = &RegisteredApp> {
        self.catalog.apps.iter().filter_map(|app| match app {
            AppState::Managed(_) => None,
            AppState::Unmanaged(app) => Some(app),
        })
    }

    pub fn focus(&self) -> Focus {
        self.focus
    }

    pub fn toggle_focus(&mut self) {
        self.focus = match self.focus {
            Focus::Configs if !self.fields.is_empty() => Focus::Fields,
            _ => Focus::Configs,
        };
    }

    pub fn view(&self) -> &View {
        &self.view
    }

    pub fn toggle_source_preview(&mut self) {
        self.view = match self.view {
            View::SourcePreview => View::Details,
            _ => View::SourcePreview,
        };
    }

    pub fn show_diff(&mut self, target: NormalizedTarget, text: String) {
        self.view = View::Diff { target, text };
    }

    pub fn close_view(&mut self) {
        self.view = View::Details;
    }

    pub fn set_field_set(&mut self, field_set: Option<FieldSet>) {
        self.fields = field_set
            .as_ref()
            .map(|set| set.fields.clone())
            .unwrap_or_default();
        self.field_set = field_set;
        self.selected_field = self.selected_field.min(self.fields.len().saturating_sub(1));
        if self.fields.is_empty() {
            self.focus = Focus::Configs;
        }
    }

    pub fn fields(&self) -> &[EditableField] {
        &self.fields
    }

    pub fn selected_field(&self) -> usize {
        self.selected_field
    }

    pub fn is_field_changed(&self, index: usize) -> bool {
        let original = self
            .field_set
            .as_ref()
            .and_then(|set| set.fields.get(index));
        self.fields.get(index) != original
    }

    pub fn has_unsaved_change(&self) -> bool {
        (0..self.fields.len()).any(|index| self.is_field_changed(index))
    }

    pub fn step_field(&mut self, direction: Direction) {
        if self.focus != Focus::Fields {
            return;
        }

        if let Some(field) = self.fields.get_mut(self.selected_field)
            && let Some(value) = field.stepped(direction)
        {
            field.value = value;
        }
    }

    pub fn discard_change(&mut self) {
        if let Some(set) = &self.field_set {
            self.fields = set.fields.clone();
        }
        self.view = View::Details;
    }

    pub fn prepared_change(&self) -> Result<Option<PreparedChange>, AdapterError> {
        let Some(set) = &self.field_set else {
            return Ok(None);
        };
        let mut new_text = set.text.clone();

        for (field, original) in self.fields.iter().zip(&set.fields) {
            if field != original {
                new_text = adapter::changed_text(&set.adapter, &new_text, field)?;
            }
        }

        if new_text == set.text {
            return Ok(None);
        }

        Ok(Some(PreparedChange {
            source: set.source.clone(),
            target: set.target.clone(),
            old_text: set.text.clone(),
            new_text,
        }))
    }

    pub fn select_next(&mut self) {
        self.move_selection(1);
    }

    pub fn select_previous(&mut self) {
        self.move_selection(-1);
    }

    pub fn edit_action(&self) -> Action {
        self.selected_app()
            .map(|app| Action::Edit(app.source.clone()))
            .unwrap_or(Action::None)
    }

    pub fn apply_action(&self) -> Action {
        self.selected_app()
            .map(|app| Action::Apply(app.target.clone()))
            .unwrap_or(Action::None)
    }

    fn move_selection(&mut self, offset: isize) {
        match self.focus {
            Focus::Fields => {
                self.selected_field = wrapped(self.selected_field, self.fields.len(), offset);
            }
            Focus::Configs if self.has_unsaved_change() => {
                self.set_notice("Unsaved change. Press [w] to write or [Esc] to discard.".into());
            }
            Focus::Configs => {
                self.selected_index =
                    wrapped(self.selected_index, self.managed_apps().count(), offset);
            }
        }
    }
}

fn wrapped(index: usize, count: usize, offset: isize) -> usize {
    if count == 0 {
        return 0;
    }

    (index as isize + offset).rem_euclid(count as isize) as usize
}
