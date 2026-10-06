use std::{fs, path::PathBuf};

use thiserror::Error;

use crate::{
    catalog::ManagedApp,
    chezmoi::{Chezmoi, ChezmoiError},
    domain::{Adapter, NormalizedTarget},
    field::EditableField,
    kitty::{self, KittyError},
    starship::{self, StarshipError},
};

#[derive(Clone, Debug, PartialEq)]
pub struct FieldSet {
    pub adapter: Adapter,
    pub source: PathBuf,
    pub target: NormalizedTarget,
    pub text: String,
    pub fields: Vec<EditableField>,
}

#[derive(Debug, Error)]
pub enum AdapterError {
    #[error("could not read {path}: {source}")]
    Read {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error(transparent)]
    Chezmoi(#[from] ChezmoiError),
    #[error(transparent)]
    Kitty(#[from] KittyError),
    #[error(transparent)]
    Starship(#[from] StarshipError),
    #[error("the raw adapter has no fields")]
    NoFields,
}

pub fn load(app: &ManagedApp, chezmoi: &dyn Chezmoi) -> Result<Option<FieldSet>, AdapterError> {
    let (source, text, fields) = match app.app.adapter {
        Adapter::Raw => return Ok(None),
        Adapter::Kitty => {
            let source = chezmoi.source_path(&app.target.join(kitty::CONFIG_FILE))?;
            let text = read(&source)?;
            let themes =
                kitty::theme_names(app.target.as_path()).map_err(|source| AdapterError::Read {
                    path: app.target.as_path().to_path_buf(),
                    source,
                })?;
            let fields = kitty::fields(&text, &themes)?;
            (source, text, fields)
        }
        Adapter::Starship => {
            let text = read(&app.source)?;
            let fields = starship::fields(&text)?;
            (app.source.clone(), text, fields)
        }
    };

    Ok(Some(FieldSet {
        adapter: app.app.adapter.clone(),
        source,
        target: app.target.clone(),
        text,
        fields,
    }))
}

pub fn changed_text(
    adapter: &Adapter,
    text: &str,
    field: &EditableField,
) -> Result<String, AdapterError> {
    match adapter {
        Adapter::Raw => Err(AdapterError::NoFields),
        Adapter::Kitty => Ok(kitty::set_value(text, field)?),
        Adapter::Starship => Ok(starship::set_value(text, field)?),
    }
}

fn read(path: &PathBuf) -> Result<String, AdapterError> {
    fs::read_to_string(path).map_err(|source| AdapterError::Read {
        path: path.clone(),
        source,
    })
}
