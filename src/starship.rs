use thiserror::Error;
use toml_edit::{DocumentMut, Item, Value};

use crate::field::{EditableField, FieldValue};

const ADD_NEWLINE: &str = "add_newline";

#[derive(Debug, Error)]
pub enum StarshipError {
    #[error("could not parse starship.toml: {0}")]
    Parse(#[from] toml_edit::TomlError),
    #[error("{key} has an invalid value: {value}")]
    Invalid { key: String, value: String },
    #[error("LazyConfig cannot edit the Starship setting {0}")]
    Unsupported(String),
}

pub fn fields(text: &str) -> Result<Vec<EditableField>, StarshipError> {
    let document = text.parse::<DocumentMut>()?;
    let add_newline = match document.get(ADD_NEWLINE) {
        None => true,
        Some(item) => item.as_bool().ok_or_else(|| StarshipError::Invalid {
            key: ADD_NEWLINE.into(),
            value: item.to_string().trim().into(),
        })?,
    };

    Ok(vec![EditableField::from_bool(
        ADD_NEWLINE,
        ADD_NEWLINE,
        add_newline,
    )])
}

pub fn set_value(text: &str, field: &EditableField) -> Result<String, StarshipError> {
    let (ADD_NEWLINE, FieldValue::Bool(flag)) = (field.id.as_str(), &field.value) else {
        return Err(StarshipError::Unsupported(field.id.clone()));
    };
    let mut document = text.parse::<DocumentMut>()?;

    match document.get_mut(ADD_NEWLINE).and_then(Item::as_value_mut) {
        Some(value) => {
            let decor = value.decor().clone();
            *value = Value::from(*flag);
            *value.decor_mut() = decor;
        }
        None => document[ADD_NEWLINE] = toml_edit::value(*flag),
    }

    Ok(document.to_string())
}
