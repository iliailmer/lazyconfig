use lazyconfig::field::{Direction, EditableField, FieldValue};

#[test]
fn kitty_font_size() {
    let editable = EditableField::from_number("font_size", "font_size", 14.0);
    let golden = EditableField::new("font_size", "font_size", FieldValue::Number(14.0));
    assert_eq!(editable, golden);
}

#[test]
fn number_steps_stay_inside_the_range() {
    let opacity = EditableField::from_number("background_opacity", "opacity", 0.95)
        .with_range(0.05, 0.0, 1.0);

    assert_eq!(
        opacity.stepped(Direction::Down),
        Some(FieldValue::Number(0.9))
    );

    let full =
        EditableField::from_number("background_opacity", "opacity", 1.0).with_range(0.05, 0.0, 1.0);

    assert_eq!(full.stepped(Direction::Up), None);
}

#[test]
fn choice_steps_wrap_to_the_first_choice() {
    let theme = EditableField::from_string("theme", "theme", "tokyonight.conf")
        .with_choices(vec!["onedark.conf".into(), "tokyonight.conf".into()]);

    assert_eq!(
        theme.stepped(Direction::Up),
        Some(FieldValue::Text("onedark.conf".into()))
    );
}
