use std::{fs, io, path::Path};

use thiserror::Error;

use crate::field::{EditableField, FieldValue, Step};

pub const CONFIG_FILE: &str = "kitty.conf";

const FONT_SIZE: &str = "font_size";
const BACKGROUND_OPACITY: &str = "background_opacity";
const THEME: &str = "theme";
const INCLUDE: &str = "include";

#[derive(Debug, Error, PartialEq)]
pub enum KittyError {
    #[error("{0} is missing from kitty.conf")]
    Missing(String),
    #[error("{0} appears more than once in kitty.conf")]
    Duplicate(String),
    #[error("{key} has an invalid value: {value}")]
    Invalid { key: String, value: String },
    #[error("LazyConfig cannot edit the Kitty setting {0}")]
    Unsupported(String),
}

pub fn fields(text: &str, themes: &[String]) -> Result<Vec<EditableField>, KittyError> {
    let font_size =
        single(text, FONT_SIZE)?.ok_or_else(|| KittyError::Missing(FONT_SIZE.into()))?;
    let mut fields = vec![
        EditableField::from_number(FONT_SIZE, FONT_SIZE, number(FONT_SIZE, font_size)?)
            .with_range(1.0, 1.0, 200.0),
    ];

    if let Some(opacity) = single(text, BACKGROUND_OPACITY)? {
        fields.push(
            EditableField::from_number(
                BACKGROUND_OPACITY,
                BACKGROUND_OPACITY,
                number(BACKGROUND_OPACITY, opacity)?,
            )
            .with_range(0.05, 0.0, 1.0),
        );
    }

    let theme_includes = values(text, INCLUDE)
        .filter(|value| themes.iter().any(|theme| theme == value))
        .collect::<Vec<_>>();
    match theme_includes.as_slice() {
        [] => {}
        [theme] => {
            fields.push(EditableField::from_string(THEME, THEME, theme).with_choices(themes.into()))
        }
        _ => return Err(KittyError::Duplicate("theme include".into())),
    }

    Ok(fields)
}

pub fn set_value(text: &str, field: &EditableField) -> Result<String, KittyError> {
    let new_value = field.value.to_string();

    match (field.id.as_str(), &field.value, &field.step) {
        (FONT_SIZE, FieldValue::Number(size), _) if size.is_finite() && *size > 0.0 => {
            replace(text, FONT_SIZE, FONT_SIZE, |_| true, &new_value)
        }
        (BACKGROUND_OPACITY, FieldValue::Number(opacity), _) if (0.0..=1.0).contains(opacity) => {
            replace(
                text,
                BACKGROUND_OPACITY,
                BACKGROUND_OPACITY,
                |_| true,
                &new_value,
            )
        }
        (THEME, FieldValue::Text(theme), Step::Choices(themes)) if themes.contains(theme) => {
            replace(
                text,
                INCLUDE,
                "theme include",
                |value| themes.iter().any(|theme| theme == value),
                &new_value,
            )
        }
        (FONT_SIZE | BACKGROUND_OPACITY | THEME, _, _) => Err(KittyError::Invalid {
            key: field.id.clone(),
            value: new_value,
        }),
        _ => Err(KittyError::Unsupported(field.id.clone())),
    }
}

pub fn theme_names(directory: &Path) -> io::Result<Vec<String>> {
    let mut themes = Vec::new();

    for entry in fs::read_dir(directory)? {
        let path = entry?.path();
        let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        if name == CONFIG_FILE || !name.ends_with(".conf") || !path.is_file() {
            continue;
        }

        let text = fs::read_to_string(&path)?;
        if values(&text, "background").next().is_some()
            || values(&text, "foreground").next().is_some()
        {
            themes.push(name.to_owned());
        }
    }

    themes.sort();
    Ok(themes)
}

fn setting(line: &str) -> Option<(&str, &str)> {
    let line = line.trim();
    if line.starts_with('#') {
        return None;
    }

    let (key, value) = line.split_once(char::is_whitespace)?;
    Some((key, value.trim()))
}

fn values<'a>(text: &'a str, key: &'a str) -> impl Iterator<Item = &'a str> {
    text.lines()
        .filter_map(setting)
        .filter(move |(found, _)| *found == key)
        .map(|(_, value)| value)
}

fn single<'a>(text: &'a str, key: &'a str) -> Result<Option<&'a str>, KittyError> {
    let mut found = values(text, key);
    let first = found.next();

    if found.next().is_some() {
        return Err(KittyError::Duplicate(key.into()));
    }

    Ok(first)
}

fn number(key: &str, value: &str) -> Result<f64, KittyError> {
    value
        .parse::<f64>()
        .ok()
        .filter(|number| number.is_finite())
        .ok_or_else(|| KittyError::Invalid {
            key: key.into(),
            value: value.into(),
        })
}

fn replace(
    text: &str,
    key: &str,
    name: &str,
    owns: impl Fn(&str) -> bool,
    new_value: &str,
) -> Result<String, KittyError> {
    let mut changed = String::with_capacity(text.len() + new_value.len());
    let mut count = 0;

    for line in text.split_inclusive('\n') {
        let body = line.trim_end_matches(['\n', '\r']);
        match setting(body) {
            Some((found, value)) if found == key && owns(value) => {
                count += 1;
                let value_start = body.trim_end().len() - value.len();
                changed.push_str(&body[..value_start]);
                changed.push_str(new_value);
                changed.push_str(&line[body.len()..]);
            }
            _ => changed.push_str(line),
        }
    }

    match count {
        0 => Err(KittyError::Missing(name.into())),
        1 => Ok(changed),
        _ => Err(KittyError::Duplicate(name.into())),
    }
}
