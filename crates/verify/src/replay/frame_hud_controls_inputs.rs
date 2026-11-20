//! Qualified early HUD bindings and current inventory, without display-history inputs.

use super::frame_hud_inputs::{self, HudInputFault};
use game::production::frame::hud_controls_inputs::{Bindings, CurrentInventory};
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(transparent)]
struct Flag(bool);

#[derive(Deserialize)]
struct Proof {
    format: String,
    old_false_or_incorrect_v3_map_used: Flag,
    old_false_v3_read_flags_changed: Flag,
    physical_read_qualification_claimed_by_this_addendum: Flag,
    standing_or_empty_pad_gate_reused: Flag,
    source_coordinates_role: String,
    require_actual_current_owner_dependency_join: Flag,
    expected_hud_or_callback_outputs_used_as_inputs: Flag,
    typed_current_capabilities: Capabilities,
    current_descriptor: Vec<Descriptor>,
    quiet_event_equivalence: Equivalence,
}

#[derive(Deserialize)]
struct Capabilities {
    early_can_switch: bool,
    control_owner_handler_codes_are_gameplay_selectors: bool,
}

#[derive(Deserialize)]
struct Equivalence {
    no_upstream_health_capacity_or_stock_writer_before_display: bool,
}

#[derive(Deserialize)]
struct Descriptor {
    name: String,
    opportunity: String,
    private_wram_offset: usize,
    quantity_width: usize,
    origin_equal_at_actual_opportunity: bool,
    preceding_cpu_writer_count: u64,
    current_owner: String,
}

const EARLY_FIELDS: [(&str, usize, &str); 13] = [
    ("Controls.pressed", 0x008f, "Controls"),
    ("Controls.held", 0x008b, "Controls"),
    ("Controls.select.binding", 0x09ba, "Controls"),
    ("Controls.cancel.binding", 0x09b8, "Controls"),
    ("Controls.owner.can_switch", 0x0a42, "Controls"),
    ("Controls.pose.can_switch", 0x0a1c, "Controls"),
    ("Display.status.equipment", 0x09a2, "Display"),
    (
        "Display.status.ammunition.Missiles.count",
        0x09c6,
        "Display",
    ),
    (
        "Display.status.ammunition.Missiles.capacity",
        0x09c8,
        "Display",
    ),
    (
        "Display.status.ammunition.SuperMissiles.count",
        0x09ca,
        "Display",
    ),
    (
        "Display.status.ammunition.SuperMissiles.capacity",
        0x09cc,
        "Display",
    ),
    (
        "Display.status.ammunition.PowerBombs.count",
        0x09ce,
        "Display",
    ),
    (
        "Display.status.ammunition.PowerBombs.capacity",
        0x09d0,
        "Display",
    ),
];

fn source(proof: &str) -> Result<Proof, HudInputFault> {
    let proof: Proof = toml::from_str(proof).map_err(|_| HudInputFault::Provenance)?;
    if proof.format != "hud-current-owner-fresh-display-source-v2"
        || proof.old_false_or_incorrect_v3_map_used.0
        || proof.old_false_v3_read_flags_changed.0
        || proof.physical_read_qualification_claimed_by_this_addendum.0
        || proof.standing_or_empty_pad_gate_reused.0
        || proof.source_coordinates_role != "ProvenanceOnly"
        || !proof.require_actual_current_owner_dependency_join.0
        || proof.expected_hud_or_callback_outputs_used_as_inputs.0
        || !proof.typed_current_capabilities.early_can_switch
        || proof
            .typed_current_capabilities
            .control_owner_handler_codes_are_gameplay_selectors
        || !proof
            .quiet_event_equivalence
            .no_upstream_health_capacity_or_stock_writer_before_display
    {
        return Err(HudInputFault::Provenance);
    }
    for (name, offset, opportunity) in EARLY_FIELDS {
        let mut matches = proof
            .current_descriptor
            .iter()
            .filter(|field| field.name == name);
        let field = matches.next().ok_or(HudInputFault::Provenance)?;
        if matches.next().is_some()
            || field.private_wram_offset != offset
            || field.quantity_width != 2
            || field.opportunity != opportunity
            || !field.origin_equal_at_actual_opportunity
            || field.preceding_cpu_writer_count != 0
            || field.current_owner != "ActualCurrentOwnerSeedPreservedByAdmittedPriorProducers"
        {
            return Err(HudInputFault::Provenance);
        }
    }
    Ok(proof)
}

/// Import the early source-qualified Select binding and switch permission.
/// The loader pins the accepted HUD-v2 proof and origin. Actual completed `PoseInput`
/// controls and current gameplay owners are joined by the production qualifier.
/// # Errors
/// Rejects incomplete memory, unsupported early capability or incompatible early descriptors.
pub fn qualified_bindings(bytes: &[u8], proof: &str) -> Result<Bindings, HudInputFault> {
    if bytes.len() != 131_072 {
        return Err(HudInputFault::Length);
    }
    let proof = source(proof)?;
    let select = bytes.get(0x09ba..0x09bc).ok_or(HudInputFault::Length)?;
    Ok(Bindings {
        select: u16::from_le_bytes(select.try_into().map_err(|_| HudInputFault::Length)?),
        can_switch: proof.typed_current_capabilities.early_can_switch,
    })
}

/// Admit current stock, capacities and equipped item meanings into their external owner.
/// The source's no-prior-writer proof extends through the later display opportunity;
/// last drawn HUD values and future refresh results are never inventory inputs.
/// # Errors
/// Rejects incomplete early source provenance or out-of-domain current ammunition.
pub fn qualified_inventory(bytes: &[u8], proof: &str) -> Result<CurrentInventory, HudInputFault> {
    let bindings = qualified_bindings(bytes, proof)?;
    let inputs = frame_hud_inputs::controls(bytes, bindings.can_switch)?;
    Ok(CurrentInventory::from_current(
        inputs.ammunition,
        inputs.equipped,
    ))
}

#[cfg(test)]
#[path = "frame_hud_controls_inputs/tests.rs"]
mod tests;
