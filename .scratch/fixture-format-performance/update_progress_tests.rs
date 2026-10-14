use std::fs;
fn main() {
    let path = "crates/pipeline/src/progress/tests.rs";
    let mut text = fs::read_to_string(path).unwrap();
    text = text.replace("tools/port/declared.toml", "tools/port/declared.smf");
    text = text.replace("\"family = 'declaration'\\n[[routines]]\\nsymbol = 'First'\\n[[routines]]\\nsymbol = 'HostWrapper'\\n\",",
        "crate::fixture_format::encode(&crate::family_model::Manifest { family: \"declaration\".into(), routines: [\"First\", \"HostWrapper\"].into_iter().map(|symbol| crate::family_model::Routine { symbol: symbol.into(), ..Default::default() }).collect(), ..Default::default() }).unwrap(),");
    let start = text.find("#[test]\nfn declarations_skip_large_fixture_payloads").unwrap();
    let end = text.find("#[test]\nfn refresh_covers_inventory").unwrap();
    text.replace_range(start..end, r#"#[test]
fn binary_declarations_preserve_symbols_and_reject_corruption() {
    let scratch = Scratch::new();
    fs::create_dir_all(scratch.0.join("tools/port")).unwrap();
    let path = scratch.0.join("tools/port/large.smf");
    let manifest = crate::family_model::Manifest {
        family: "large".into(),
        routines: ["First", "First", "Unknown"].into_iter().map(|symbol| crate::family_model::Routine {
            symbol: symbol.into(),
            cases: vec![crate::family_model::Case {
                name: "payload".into(),
                writes: vec![crate::family_model::Write { offset: 0, data: vec![12; 0x20000] }].into(),
                ..Default::default()
            }],
            ..Default::default()
        }).collect(),
        ..Default::default()
    };
    crate::fixture_format::write(&path, &manifest).unwrap();
    let mut routines = registry(&scratch.0).routines;
    let mut notices = Vec::new();
    add_declarations(&scratch.0, &mut routines, &mut notices).unwrap();
    assert_eq!(routines[0].declared_in, ["tools/port/large.smf"]);
    assert!(routines[1].declared_in.is_empty());
    assert_eq!(notices, ["Declaration tools/port/large.smf names Unknown outside the C inventory; it contributes no C coverage."]);
    fs::write(path, b"SMFIXT01truncated").unwrap();
    assert!(add_declarations(&scratch.0, &mut routines, &mut notices).is_err());
}

"#);
    fs::write(path, text).unwrap();
}
