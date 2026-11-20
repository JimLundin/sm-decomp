use super::*;
use std::fmt::Write as _;

fn proof() -> String {
    let mut text = String::from(
        "format = \"hud-current-owner-fresh-display-source-v2\"\n\
         old_false_or_incorrect_v3_map_used = false\nold_false_v3_read_flags_changed = false\n\
         physical_read_qualification_claimed_by_this_addendum = false\n\
         standing_or_empty_pad_gate_reused = false\nsource_coordinates_role = \"ProvenanceOnly\"\n\
         require_actual_current_owner_dependency_join = true\n\
         expected_hud_or_callback_outputs_used_as_inputs = false\n\
         [typed_current_capabilities]\nearly_can_switch = true\n\
         control_owner_handler_codes_are_gameplay_selectors = false\n\
         [quiet_event_equivalence]\nno_upstream_health_capacity_or_stock_writer_before_display = true\n",
    );
    for (name, at, opportunity) in EARLY_FIELDS {
        write!(
            text,
            "\n[[current_descriptor]]\nname = {name:?}\nopportunity = {opportunity:?}\n\
            private_wram_offset = {at}\nquantity_width = 2\n\
            origin_equal_at_actual_opportunity = true\npreceding_cpu_writer_count = 0\n\
            current_owner = \"ActualCurrentOwnerSeedPreservedByAdmittedPriorProducers\"\n"
        )
        .unwrap();
    }
    text
}

fn put(bytes: &mut [u8], at: usize, value: u16) {
    bytes
        .get_mut(at..at.saturating_add(2))
        .unwrap()
        .copy_from_slice(&value.to_le_bytes());
}

#[test]
fn early_binding_and_inventory_use_current_source_without_hud_display_history() {
    let mut bytes = vec![0; 131_072];
    put(&mut bytes, 0x09ba, 0x0400);
    put(&mut bytes, 0x09c6, 7);
    put(&mut bytes, 0x09c8, 11);
    put(&mut bytes, 0x09a2, 0x4000);
    assert_eq!(
        qualified_bindings(&bytes, &proof()).unwrap(),
        Bindings {
            select: 0x0400,
            can_switch: true
        }
    );
    let inventory = qualified_inventory(&bytes, &proof()).unwrap();
    assert_eq!(inventory.missiles.count, 7);
    assert_eq!(inventory.missiles.capacity, 11);
    assert!(inventory.grapple_beam);
    assert_eq!(
        qualified_bindings(&[], &proof()),
        Err(HudInputFault::Length)
    );
}

#[test]
fn missing_stale_binding_and_future_inventory_permissions_reject() {
    let bytes = vec![0; 131_072];
    let value: toml::Value = toml::from_str(&proof()).unwrap();
    let mut changed = value.clone();
    changed
        .get_mut("current_descriptor")
        .unwrap()
        .as_array_mut()
        .unwrap()
        .first_mut()
        .unwrap()
        .as_table_mut()
        .unwrap()
        .insert(
            "preceding_cpu_writer_count".to_owned(),
            toml::Value::Integer(1),
        );
    assert_eq!(
        qualified_bindings(&bytes, &toml::to_string(&changed).unwrap()),
        Err(HudInputFault::Provenance)
    );
    let mut changed = value.clone();
    changed
        .get_mut("current_descriptor")
        .unwrap()
        .as_array_mut()
        .unwrap()
        .pop();
    assert_eq!(
        qualified_inventory(&bytes, &toml::to_string(&changed).unwrap()),
        Err(HudInputFault::Provenance)
    );
    let mut changed = value;
    changed
        .get_mut("quiet_event_equivalence")
        .unwrap()
        .as_table_mut()
        .unwrap()
        .insert(
            "no_upstream_health_capacity_or_stock_writer_before_display".to_owned(),
            toml::Value::Boolean(false),
        );
    assert_eq!(
        qualified_inventory(&bytes, &toml::to_string(&changed).unwrap()),
        Err(HudInputFault::Provenance)
    );
}

#[test]
#[ignore = "requires accepted independent HUD current-source proof and origin"]
fn accepted_early_controls_source_packet_imports() {
    let proof = std::fs::read(std::env::var_os("SM_FRAME_HUD_CURRENT_PROOF").unwrap()).unwrap();
    assert_eq!(
        crate::lock::Sha256::of(&proof).to_string(),
        "424f58aff2ea37c662cb2e0b44091f0b6ef36d5c2505ea2d1664b80c338827d3"
    );
    let bytes = std::fs::read(std::env::var_os("SM_FRAME_CURRENT_ORIGIN").unwrap()).unwrap();
    assert_eq!(
        crate::lock::Sha256::of(&bytes).to_string(),
        "bfee6ac13da413a36ad42194a36aa3224af6a84e818092b1e7040ed4782f03db"
    );
    let proof = std::str::from_utf8(&proof).unwrap();
    assert!(qualified_bindings(&bytes, proof).is_ok());
    assert!(qualified_inventory(&bytes, proof).is_ok());
}
