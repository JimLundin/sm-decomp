use super::*;

#[test]
#[ignore = "requires explicit pinned ROM, accepted Inside source and independent origin"]
fn actual_source_current_geometry_and_complete_owner_are_imported_once() {
    let (native, origin, proof, templates) = fixture();
    let diagnostic: Source = toml::from_str(&proof).unwrap();
    for field in diagnostic
        .current_guard
        .iter()
        .filter(|field| field.name != "CurrentBodyHalfHeight")
    {
        assert!(
            word(&origin, field.private_wram_offset).unwrap() == field.required_current_encoding,
            "initial current field {}",
            field.name
        );
    }
    for field in &diagnostic.room_guard {
        let end = field
            .private_wram_offset
            .checked_add(field.quantity_width)
            .unwrap();
        assert!(
            crate::lock::Sha256::of(origin.get(field.private_wram_offset..end).unwrap())
                == crate::lock::Sha256::try_from(field.required_current_sha256.as_str()).unwrap(),
            "room field {}",
            field.name
        );
    }
    let (room, _) = contract(&origin, &native, &templates, &proof).unwrap();
    assert_eq!(room.validate(), Ok(()));
    assert!(whole_template(&templates).is_ok());
    let source: Source = toml::from_str(&proof).unwrap();
    assert_eq!(
        (
            source.current_guard.len(),
            source.body_join.len(),
            source.room_guard.len(),
            source.query_binding.len(),
            source.physical_portion.len()
        ),
        (4, 2, 5, 3, 14)
    );
    // Initial body bytes may precede the completed producer; they cannot seed its body.
    let mut before_body = origin.clone();
    for field in &diagnostic.body_join {
        *before_body.get_mut(field.private_wram_offset).unwrap() ^= 1;
    }
    assert!(contract(&before_body, &native, &templates, &proof).is_ok());
    let mut wrong = origin.clone();
    *wrong
        .get_mut(source.current_guard.first().unwrap().private_wram_offset)
        .unwrap() ^= 1;
    assert!(matches!(
        contract(&wrong, &native, &templates, &proof),
        Err(Fault::CurrentSource)
    ));
    let mut wrong = source;
    wrong
        .query_binding
        .first_mut()
        .unwrap()
        .source_type_read_private_wram_offset = 0;
    let scalar = |name: &str| {
        wrong
            .current_guard
            .iter()
            .chain(&wrong.body_join)
            .find(|field| field.name == name)
            .unwrap()
            .required_current_encoding
    };
    let width = word(
        &origin,
        wrong
            .room_guard
            .iter()
            .find(|field| field.name == "ROOM_WIDTH_IN_BLOCKS")
            .unwrap()
            .private_wram_offset,
    )
    .unwrap();
    assert_eq!(
        raw_queries(
            &origin,
            &wrong,
            [scalar("CurrentCentreX"), scalar("CurrentCentreY")],
            scalar("SAMUS_Y_RADIUS"),
            width
        ),
        Err(Fault::CurrentSource)
    );
}

fn fixture() -> (super::super::movement::Content, Vec<u8>, String, Templates) {
    let native = super::super::movement::Content::load(&std::path::PathBuf::from(
        std::env::var_os("SM_ROM").expect("explicit ROM required"),
    ))
    .unwrap();
    let origin = std::fs::read(
        std::env::var_os("SM_INSIDE_ORIGIN").expect("explicit Inside origin required"),
    )
    .unwrap();
    let proof = std::fs::read_to_string(
        std::env::var_os("SM_INSIDE_CONTRACT").expect("explicit Inside proof required"),
    )
    .unwrap();
    let directory = std::path::PathBuf::from(
        std::env::var_os("SM_INSIDE_PROFILES").expect("explicit Inside profiles required"),
    );
    let texts = [
        "profiles.toml",
        "portions.toml",
        "publication-bindings.toml",
        "sample-bindings.toml",
        "materialization-bindings.toml",
        "arithmetic-receipts.toml",
    ]
    .map(|name| std::fs::read_to_string(directory.join(name)).unwrap());
    let templates =
        super::super::frame_origin::profiles::decode(texts.each_ref().map(String::as_str)).unwrap();
    (native, origin, proof, templates)
}
