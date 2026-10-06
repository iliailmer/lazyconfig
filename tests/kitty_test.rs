use lazyconfig::{
    field::{EditableField, FieldValue},
    kitty::{self, KittyError},
};

const CONFIG: &str = "\
# font_size 99
include onedark.conf
include keys.conf

font_size             15.0
background_opacity 0.95
map ctrl+a new_window
";

fn themes() -> Vec<String> {
    vec!["onedark.conf".into(), "tokyonight.conf".into()]
}

#[test]
fn parses_active_settings_and_ignores_comments() {
    let fields = kitty::fields(CONFIG, &themes()).unwrap();

    assert_eq!(
        fields
            .iter()
            .map(|field| (field.id.as_str(), field.value.clone()))
            .collect::<Vec<_>>(),
        [
            ("font_size", FieldValue::Number(15.0)),
            ("background_opacity", FieldValue::Number(0.95)),
            ("theme", FieldValue::Text("onedark.conf".into())),
        ]
    );
}

#[test]
fn reports_a_missing_font_size() {
    let error = kitty::fields("# font_size 14\n", &[]).unwrap_err();

    assert_eq!(error, KittyError::Missing("font_size".into()));
}

#[test]
fn reports_a_duplicate_font_size() {
    let error = kitty::fields("font_size 14\nfont_size 15\n", &[]).unwrap_err();

    assert_eq!(error, KittyError::Duplicate("font_size".into()));
}

#[test]
fn changes_only_the_font_size_line() {
    let mut fields = kitty::fields(CONFIG, &themes()).unwrap();
    fields[0].value = FieldValue::Number(16.0);

    let changed = kitty::set_value(CONFIG, &fields[0]).unwrap();

    assert_eq!(
        changed,
        CONFIG.replace("font_size             15.0", "font_size             16.0")
    );
}

#[test]
fn changes_only_the_theme_include_line() {
    let mut fields = kitty::fields(CONFIG, &themes()).unwrap();
    fields[2].value = FieldValue::Text("tokyonight.conf".into());

    let changed = kitty::set_value(CONFIG, &fields[2]).unwrap();

    assert_eq!(
        changed,
        CONFIG.replace("include onedark.conf", "include tokyonight.conf")
    );
}

#[test]
fn rejects_a_font_size_of_zero() {
    let field = EditableField::from_number("font_size", "font_size", 0.0);

    let error = kitty::set_value(CONFIG, &field).unwrap_err();

    assert!(matches!(error, KittyError::Invalid { .. }));
}
