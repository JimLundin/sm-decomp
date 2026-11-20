//! Current HUD buttons and status do not capture later display quantities.

#[path = "../src/replay/frame_hud_inputs.rs"]
mod hud_inputs;

use hud_inputs::{HudInputFault, controls, status};
use interface_hud::{CancelButton, SelectButton};

fn put(bytes: &mut [u8], at: usize, value: u16) -> Option<()> {
    bytes
        .get_mut(at..at.saturating_add(2))?
        .copy_from_slice(&value.to_le_bytes());
    Some(())
}

#[test]
fn actual_controls_accept_nonstationary_pad_and_keep_their_current_answer() {
    let mut bytes = vec![0; 131_072];
    put(&mut bytes, 0x008b, 0x0100).unwrap();
    put(&mut bytes, 0x008f, 0x0200).unwrap();
    put(&mut bytes, 0x09b8, 0x0100).unwrap();
    put(&mut bytes, 0x09ba, 0x0200).unwrap();
    let early = controls(&bytes, true).unwrap();
    assert_eq!(early.select, SelectButton::Pressed);
    assert_eq!(early.cancel, CancelButton::Held);
    assert!(early.can_switch);
    put(&mut bytes, 0x008f, 0x0100).unwrap();
    put(&mut bytes, 0x09c2, 117).unwrap();
    let fresh = controls(&bytes, false).unwrap();
    assert_eq!(fresh.cancel, CancelButton::Pressed);
    assert_eq!(fresh.select, SelectButton::NotPressed);
    assert!(!fresh.can_switch);
    assert_eq!(early.select, SelectButton::Pressed);
    assert_eq!(status(&bytes).unwrap().energy.now, 117);
    put(&mut bytes, 0x09ca, 100).unwrap();
    assert_eq!(controls(&bytes, true), Err(HudInputFault::Ammunition));
    assert_eq!(status(&bytes), Err(HudInputFault::Ammunition));
}

#[test]
fn current_status_and_modes_are_checked_without_a_quiet_health_fallback() {
    let mut bytes = vec![0; 131_072];
    put(&mut bytes, 0x09c2, 231).unwrap();
    put(&mut bytes, 0x09c4, 499).unwrap();
    let early = status(&bytes).unwrap();
    put(&mut bytes, 0x09c2, 117).unwrap();
    assert_eq!(early.energy.now, 231);
    assert_eq!(status(&bytes).unwrap().energy.now, 117);
    put(&mut bytes, 0x09c0, 3).unwrap();
    assert_eq!(status(&bytes), Err(HudInputFault::ReserveMode));
    assert_eq!(controls(&bytes[..1], true), Err(HudInputFault::Length));
}

#[test]
#[ignore = "requires corrected independent HUD current-owner proof and qualified origin"]
fn current_owner_admission_rejects_old_aliases_and_false_read_promotions() {
    let path = std::env::var_os("SM_FRAME_HUD_CURRENT_PROOF").expect("HUD current owner proof");
    let bytes = std::fs::read(path).unwrap();
    assert_eq!(
        verify::lock::Sha256::of(&bytes).to_string(),
        "424f58aff2ea37c662cb2e0b44091f0b6ef36d5c2505ea2d1664b80c338827d3"
    );
    let proof = std::str::from_utf8(&bytes).unwrap();
    let path = std::env::var_os("SM_FRAME_CURRENT_ORIGIN").expect("qualified current origin");
    let bytes = std::fs::read(path).unwrap();
    assert_eq!(
        verify::lock::Sha256::of(&bytes).to_string(),
        "bfee6ac13da413a36ad42194a36aa3224af6a84e818092b1e7040ed4782f03db"
    );
    let area = hud_inputs::current_area(&bytes).unwrap();
    let map = interface_hud::HerMap::default();
    let admitted = hud_inputs::qualified(&bytes, proof, area, &map).unwrap();
    assert!(admitted.premises.runs);
    let unavailable = proof.replacen("early_can_switch = true", "early_can_switch = false", 1);
    assert!(unavailable.ne(proof));
    assert!(
        hud_inputs::qualified(&bytes, &unavailable, area, &map)
            .is_err_and(|fault| fault == HudInputFault::Provenance)
    );
    assert!(admitted.status.eq(&status(&bytes).unwrap()));
    let old_alias = proof.replacen(
        "private_wram_offset = 2500",
        "private_wram_offset = 2466",
        1,
    );
    assert!(
        hud_inputs::qualified(&bytes, &old_alias, area, &map)
            .is_err_and(|fault| fault == HudInputFault::Provenance)
    );
    let promoted = proof.replacen(
        "old_false_v3_read_flags_changed = false",
        "old_false_v3_read_flags_changed = true",
        1,
    );
    assert!(
        hud_inputs::qualified(&bytes, &promoted, area, &map)
            .is_err_and(|fault| fault == HudInputFault::Provenance)
    );
    let writer = proof.replacen(
        "preceding_cpu_writer_count = 0",
        "preceding_cpu_writer_count = 1",
        1,
    );
    assert!(
        hud_inputs::qualified(&bytes, &writer, area, &map)
            .is_err_and(|fault| fault == HudInputFault::Provenance)
    );
    let other_arm = proof.replacen(
        "fresh_mini_map_runs = true",
        "fresh_mini_map_runs = false",
        1,
    );
    assert!(
        hud_inputs::qualified(&bytes, &other_arm, area, &map)
            .is_err_and(|fault| fault == HudInputFault::Provenance)
    );
}
