//! Prospective proof executes production bodies without enabling normal support.
#[test]
fn prospective_lda_absolute_does_not_enter_normal_tables() {
    let instruction =
        recomp::decode::decode_with(|address| [0xad, 0x34, 0x12].get(address as usize).copied(), 0, true, true)
            .unwrap();
    if recomp::step::supported(&instruction) {
        // An approved promotion reruns this suite with production support on.
        assert!(recomp::step::emit_body(&instruction).unwrap().contains("ops::lda"));
        return;
    }
    assert!(recomp::step::emit_body(&instruction).is_err());
    let body = recomp::step::emit_prospective_body(&instruction, "ad").unwrap();
    assert!(body.contains("ops::lda"));
    assert!(recomp::step::emit_prospective_body(&instruction, "69").is_err());
}

#[test]
fn production_support_matches_the_declared_opcodes_at_every_width() {
    let declaration: pipeline_model::timed_pattern::Declaration =
        toml::from_str(recomp::step::PATTERN_DECLARATION).unwrap();
    declaration.validate().unwrap();
    for opcode in 0..=u8::MAX {
        for mx in 0..4 {
            let instruction = recomp::decode::decode_with(
                |address| [opcode, 0x34, 0x12, 0].get(address as usize).copied(),
                0,
                mx & 2 != 0,
                mx & 1 != 0,
            )
            .unwrap();
            let expected = declaration
                .patterns
                .iter()
                .any(|pattern| pattern.opcode_byte().unwrap() == opcode && pattern.mx.contains(&mx));
            assert_eq!(recomp::step::supported(&instruction), expected, "opcode {opcode:02x} MX{mx}");
        }
    }
}
