use lazyconfig::{field::FieldValue, starship};

#[test]
fn comments_survive_a_boolean_change() {
    let config = "\
# Prompt layout
add_newline = false # keep the prompt compact

[character]
success_symbol = \"[>](green)\" # shown after success
";
    let mut fields = starship::fields(config).unwrap();
    assert_eq!(fields[0].value, FieldValue::Bool(false));
    fields[0].value = FieldValue::Bool(true);

    let changed = starship::set_value(config, &fields[0]).unwrap();

    assert_eq!(
        changed,
        config.replace("add_newline = false", "add_newline = true")
    );
}

#[test]
fn a_missing_add_newline_uses_the_starship_default_and_can_be_added() {
    let config = "[character]\nsuccess_symbol = \"[>](green)\"\n";
    let mut fields = starship::fields(config).unwrap();
    assert_eq!(fields[0].value, FieldValue::Bool(true));
    fields[0].value = FieldValue::Bool(false);

    let changed = starship::set_value(config, &fields[0]).unwrap();

    assert_eq!(
        changed,
        "add_newline = false\n[character]\nsuccess_symbol = \"[>](green)\"\n"
    );
}
