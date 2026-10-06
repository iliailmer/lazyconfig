use std::{fs, path::Path};

use lazyconfig::{
    change::{ChangeError, PreparedChange},
    domain::NormalizedTarget,
};

fn change(name: &str) -> PreparedChange {
    let source = std::env::temp_dir().join(format!(
        "lazyconfig-change-{name}-{}.conf",
        std::process::id()
    ));
    fs::write(&source, "font_size 15.0\n").unwrap();

    PreparedChange {
        source,
        target: NormalizedTarget::from_registry_path("~/.config/kitty", Path::new("/Users/tester"))
            .unwrap(),
        old_text: "font_size 15.0\n".into(),
        new_text: "font_size 16.0\n".into(),
    }
}

#[test]
fn writes_the_new_text_when_the_source_is_unchanged() {
    let change = change("write");

    change.write().unwrap();

    let written = fs::read_to_string(&change.source).unwrap();
    fs::remove_file(&change.source).unwrap();
    assert_eq!(written, "font_size 16.0\n");
    assert_eq!(
        change.changed_lines(),
        ["-1: font_size 15.0", "+1: font_size 16.0"]
    );
}

#[test]
fn refuses_to_write_when_another_program_changed_the_source() {
    let change = change("refuse");
    fs::write(&change.source, "font_size 20.0\n").unwrap();

    let error = change.write().unwrap_err();

    let kept = fs::read_to_string(&change.source).unwrap();
    fs::remove_file(&change.source).unwrap();
    assert!(matches!(error, ChangeError::SourceChanged(_)));
    assert_eq!(kept, "font_size 20.0\n");
}
