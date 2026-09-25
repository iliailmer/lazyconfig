use std::path::PathBuf;

use crate::{
    catalog::{AppState, Catalog, ManagedApp},
    domain::{NormalizedTarget, RegisteredApp},
};

#[derive(Debug, PartialEq, Eq)]
pub enum Action {
    None,
    Edit(PathBuf),
    Apply(NormalizedTarget),
}

pub struct App {
    catalog: Catalog,
    selected_index: usize,
    notice: Option<String>,
}

impl App {
    pub fn new(catalog: Catalog) -> Self {
        Self {
            catalog,
            selected_index: 0,
            notice: None,
        }
    }

    pub fn selected_app(&self) -> Option<&ManagedApp> {
        self.managed_apps().nth(self.selected_index)
    }

    pub fn replace_catalog(&mut self, catalog: Catalog) {
        self.catalog = catalog;
        self.selected_index = 0;
    }

    pub fn set_notice(&mut self, notice: String) {
        self.notice = Some(notice);
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

    pub fn select_next(&mut self) {
        let managed_count = self.managed_count();

        if managed_count > 0 {
            self.selected_index = (self.selected_index + 1) % managed_count;
        }
    }

    pub fn select_previous(&mut self) {
        let managed_count = self.managed_count();

        if managed_count > 0 {
            self.selected_index = (self.selected_index + managed_count - 1) % managed_count;
        }
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

    fn managed_count(&self) -> usize {
        self.managed_apps().count()
    }
}
