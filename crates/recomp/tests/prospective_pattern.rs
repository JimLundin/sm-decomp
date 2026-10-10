//! Prospective proof executes production bodies without enabling normal support.
#[test]
fn prospective_lda_absolute_does_not_enter_normal_tables() {
    let instruction =
        recomp::decode::decode_with(|address| [0xad, 0x34, 0x12].get(address as usize).copied(), 0, true, true)
            .unwrap();
    assert!(!recomp::step::supported(&instruction));
    assert!(recomp::step::emit_body(&instruction).is_err());
    let body = recomp::step::emit_prospective_body(&instruction, "ad").unwrap();
    assert!(body.contains("ops::lda"));
    assert!(recomp::step::emit_prospective_body(&instruction, "69").is_err());
}
