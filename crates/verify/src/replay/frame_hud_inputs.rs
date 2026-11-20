//! Independent current HUD quantities without stationary replay gates.

use enum_map::enum_map;
use game::production::frame::hud_stage::{
    DisplayPremises, DisplayReceipt, MutePremises, RefreshEvents,
};
use interface_hud::{
    Ammo, Ammunition, CancelButton, Energy, Equipped, ItemControlInputs, Reserve, ReserveMode,
    SelectButton, Status, Stock,
};
use interface_hud::{Dropped, FrameCount, HerMap, Room};
use serde::Deserialize;

/// Incomplete or out-of-domain current HUD input quantities.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HudInputFault {
    /// Complete current WRAM is required at the independent input edge.
    Length,
    /// Current ammunition exceeds the clean HUD domain.
    Ammunition,
    /// The current reserve mode has no admitted automatic/manual interpretation.
    ReserveMode,
    /// Complete current-owner source provenance is absent or incompatible.
    Provenance,
    /// The provided checked map does not belong to the actual current area.
    Area,
}

#[derive(Deserialize)]
#[serde(transparent)]
struct Flag(bool);

#[derive(Deserialize)]
struct SourceProof {
    format: String,
    old_false_or_incorrect_v3_map_used: Flag,
    old_false_v3_read_flags_changed: Flag,
    physical_read_qualification_claimed_by_this_addendum: Flag,
    standing_or_empty_pad_gate_reused: Flag,
    movement_position_authority: String,
    movement_action_for_mute_authority: String,
    source_coordinates_role: String,
    require_actual_current_owner_dependency_join: Flag,
    expected_hud_or_callback_outputs_used_as_inputs: Flag,
    current_descriptor: Vec<Descriptor>,
    typed_current_capabilities: Capabilities,
    quiet_event_equivalence: EventEquivalence,
    map_content: MapContent,
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

#[derive(Deserialize)]
struct Capabilities {
    early_can_switch: bool,
    fresh_mini_map_runs: bool,
    control_owner_handler_codes_are_gameplay_selectors: bool,
}

#[derive(Deserialize)]
struct EventEquivalence {
    required_actual_hud_selection: String,
    required_actual_hud_mini_map_power: String,
    require_current_event_owner_identity: Flag,
    checked_event_producer_entries_through_display: u64,
    dropped_before: String,
    dropped_after: String,
    collected: String,
    mother_brain_falls: Flag,
    turned_on: Flag,
    no_upstream_health_capacity_or_stock_writer_before_display: Flag,
}

#[derive(Deserialize)]
struct MapContent {
    authority: String,
    future_map_or_hud_buffer_authority: bool,
}

fn source(text: &str) -> Result<SourceProof, HudInputFault> {
    let proof: SourceProof = toml::from_str(text).map_err(|_| HudInputFault::Provenance)?;
    if proof.format != "hud-current-owner-fresh-display-source-v2"
        || proof.old_false_or_incorrect_v3_map_used.0
        || proof.old_false_v3_read_flags_changed.0
        || proof.physical_read_qualification_claimed_by_this_addendum.0
        || proof.standing_or_empty_pad_gate_reused.0
        || proof.movement_position_authority
            != "ActualSelectedMovementOutputAfterMovementBeforePosePlacement"
        || proof.movement_action_for_mute_authority != "ActualSelectedMovementAction"
        || proof.source_coordinates_role != "ProvenanceOnly"
        || !proof.require_actual_current_owner_dependency_join.0
        || proof.expected_hud_or_callback_outputs_used_as_inputs.0
        || !proof.typed_current_capabilities.early_can_switch
        || !proof.typed_current_capabilities.fresh_mini_map_runs
        || proof
            .typed_current_capabilities
            .control_owner_handler_codes_are_gameplay_selectors
        || proof.map_content.authority != "ExistingCheckedRomHerMapForActualCurrentArea"
        || proof.map_content.future_map_or_hud_buffer_authority
    {
        return Err(HudInputFault::Provenance);
    }
    let events = &proof.quiet_event_equivalence;
    if events.required_actual_hud_selection != "None"
        || events.required_actual_hud_mini_map_power != "On"
        || !events.require_current_event_owner_identity.0
        || events.checked_event_producer_entries_through_display != 0
        || events.dropped_before != "Nothing"
        || events.dropped_after != "Nothing"
        || events.collected != "None"
        || events.mother_brain_falls.0
        || events.turned_on.0
        || !events
            .no_upstream_health_capacity_or_stock_writer_before_display
            .0
    {
        return Err(HudInputFault::Provenance);
    }
    descriptors(&proof.current_descriptor)?;
    Ok(proof)
}

const CURRENT_DESCRIPTORS: [(&str, usize, usize, &str); 29] = [
    ("Display.frame", 0x05b5, 1, "Display"),
    ("Display.status.energy.now", 0x09c2, 2, "Display"),
    ("Display.status.energy.capacity", 0x09c4, 2, "Display"),
    ("Display.status.reserve.energy", 0x09d6, 2, "Display"),
    ("Display.status.reserve.mode", 0x09c0, 2, "Display"),
    ("Display.room.cell.x", 0x07a1, 2, "Display"),
    ("Display.room.cell.y", 0x07a3, 2, "Display"),
    ("Display.room.blocks.x", 0x07a5, 2, "Display"),
    ("Display.room.blocks.y", 0x07a7, 2, "Display"),
    ("Display.map_collected", 0x0789, 2, "Display"),
    ("Display.map.current_area", 0x079f, 2, "Display"),
    ("Display.status.equipment", 0x09a2, 2, "Display"),
    (
        "Display.status.ammunition.Missiles.count",
        0x09c6,
        2,
        "Display",
    ),
    (
        "Display.status.ammunition.Missiles.capacity",
        0x09c8,
        2,
        "Display",
    ),
    (
        "Display.status.ammunition.SuperMissiles.count",
        0x09ca,
        2,
        "Display",
    ),
    (
        "Display.status.ammunition.SuperMissiles.capacity",
        0x09cc,
        2,
        "Display",
    ),
    (
        "Display.status.ammunition.PowerBombs.count",
        0x09ce,
        2,
        "Display",
    ),
    (
        "Display.status.ammunition.PowerBombs.capacity",
        0x09d0,
        2,
        "Display",
    ),
    ("Controls.pressed", 0x008f, 2, "Controls"),
    ("Controls.held", 0x008b, 2, "Controls"),
    ("Controls.select.binding", 0x09ba, 2, "Controls"),
    ("Controls.cancel.binding", 0x09b8, 2, "Controls"),
    ("Controls.mute.time_stopped", 0x0a78, 2, "Controls"),
    ("Controls.mute.grapple_out", 0x0d32, 2, "Controls"),
    ("Display.mute.time_stopped", 0x0a78, 2, "Display"),
    ("Display.mute.grapple_out", 0x0d32, 2, "Display"),
    ("Controls.owner.can_switch", 0x0a42, 2, "Controls"),
    ("Controls.pose.can_switch", 0x0a1c, 2, "Controls"),
    ("Display.owner.runs", 0x0a44, 2, "Display"),
];

fn descriptors(fields: &[Descriptor]) -> Result<(), HudInputFault> {
    if fields.len() != CURRENT_DESCRIPTORS.len() {
        return Err(HudInputFault::Provenance);
    }
    for (name, offset, width, opportunity) in CURRENT_DESCRIPTORS {
        let mut matches = fields.iter().filter(|field| field.name == name);
        let field = matches.next().ok_or(HudInputFault::Provenance)?;
        if matches.next().is_some()
            || field.private_wram_offset != offset
            || field.quantity_width != width
            || field.opportunity != opportunity
            || !field.origin_equal_at_actual_opportunity
            || field.preceding_cpu_writer_count != 0
            || field.current_owner != "ActualCurrentOwnerSeedPreservedByAdmittedPriorProducers"
        {
            return Err(HudInputFault::Provenance);
        }
    }
    Ok(())
}

/// Import the independently qualified status and refresh inputs.
///
/// The loader independently pins this proof and origin and loads the supplied area
/// map from the checked player ROM. Actual current owners must be joined again at
/// refresh through `choose_admitted_display`; no position or action is imported here.
/// # Errors
/// Rejects incomplete source predicates, invalid status and a map from another area.
pub fn qualified(
    bytes: &[u8],
    proof: &str,
    area: data::room::Area,
    map: &HerMap,
) -> Result<DisplayReceipt, HudInputFault> {
    complete(bytes)?;
    let proof = source(proof)?;
    if area != current_area(bytes)? {
        return Err(HudInputFault::Area);
    }
    Ok(DisplayReceipt {
        status: status(bytes)?,
        premises: DisplayPremises {
            frame: FrameCount(word(bytes, 0x05b5)?.to_le_bytes()[0]),
            mute: MutePremises {
                time_stopped: word(bytes, 0x0a78)? != 0,
                grapple_out: word(bytes, 0x0d32)? != 0xc4f0,
            },
            runs: proof.typed_current_capabilities.fresh_mini_map_runs,
            events: RefreshEvents {
                before_status_bar: Dropped::Nothing,
                after_status_bar: Dropped::Nothing,
                collected: None,
                mother_brain_falls: false,
                turned_on: false,
            },
            room: Room {
                cell: types::Pair {
                    x: i32::from(word(bytes, 0x07a1)?),
                    y: i32::from(word(bytes, 0x07a3)?).saturating_add(1),
                },
                blocks: types::Pair {
                    x: word(bytes, 0x07a5)?,
                    y: word(bytes, 0x07a7)?,
                },
            },
            map: map.clone(),
            map_collected: word(bytes, 0x0789)? != 0,
        },
    })
}

/// Decode the actual current area for the checked HUD content lookup.
/// # Errors
/// Rejects incomplete memory and areas outside the imported map catalogue.
pub fn current_area(bytes: &[u8]) -> Result<data::room::Area, HudInputFault> {
    complete(bytes)?;
    data::room::Area::try_from(u8::try_from(word(bytes, 0x079f)?).map_err(|_| HudInputFault::Area)?)
        .map_err(|_| HudInputFault::Area)
}

fn word(bytes: &[u8], at: usize) -> Result<u16, HudInputFault> {
    let bytes = bytes
        .get(at..at.saturating_add(2))
        .ok_or(HudInputFault::Length)?;
    Ok(u16::from_le_bytes(
        bytes.try_into().map_err(|_| HudInputFault::Length)?,
    ))
}

const fn complete(bytes: &[u8]) -> Result<(), HudInputFault> {
    if bytes.len() == 131_072 {
        Ok(())
    } else {
        Err(HudInputFault::Length)
    }
}

fn ammunition(bytes: &[u8]) -> Result<Ammunition, HudInputFault> {
    let stock = |at| {
        Ok::<_, HudInputFault>(Stock {
            count: word(bytes, at)?,
            capacity: word(bytes, at.saturating_add(2))?,
        })
    };
    Ammunition::new(enum_map! {
        Ammo::Missiles => stock(0x09c6)?,
        Ammo::SuperMissiles => stock(0x09ca)?,
        Ammo::PowerBombs => stock(0x09ce)?,
    })
    .ok_or(HudInputFault::Ammunition)
}

fn equipped(bytes: &[u8]) -> Result<Equipped, HudInputFault> {
    let equipment = word(bytes, 0x09a2)?;
    Ok(Equipped {
        grapple_beam: equipment & 0x4000 != 0,
        x_ray_scope: equipment & 0x8000 != 0,
    })
}

/// Decode current committed item buttons and availability at their early opportunity.
///
/// The actual caller supplies its independently admitted current switch permission.
/// No display quantity, map, position, future control or selection result is read.
/// # Errors
/// Rejects incomplete current memory and out-of-domain ammunition.
pub fn controls(bytes: &[u8], can_switch: bool) -> Result<ItemControlInputs, HudInputFault> {
    complete(bytes)?;
    let pressed = word(bytes, 0x008f)?;
    let held = word(bytes, 0x008b)?;
    let select_binding = word(bytes, 0x09ba)?;
    let cancel_binding = word(bytes, 0x09b8)?;
    Ok(ItemControlInputs {
        select: if pressed & select_binding == 0 {
            SelectButton::NotPressed
        } else {
            SelectButton::Pressed
        },
        cancel: if pressed & cancel_binding != 0 {
            CancelButton::Pressed
        } else if held & cancel_binding != 0 {
            CancelButton::Held
        } else {
            CancelButton::NotPressed
        },
        can_switch,
        ammunition: ammunition(bytes)?,
        equipped: equipped(bytes)?,
    })
}

/// Decode complete current status quantities for the independently owned status producer.
///
/// Read or borrow this owner at the actual refresh opportunity; a first-cut
/// replacement separately requires unchanged-field source provenance. Actual
/// movement position and other display premises are supplied by their own owners.
/// # Errors
/// Rejects incomplete memory, invalid reserve modes and out-of-domain ammunition.
pub fn status(bytes: &[u8]) -> Result<Status, HudInputFault> {
    complete(bytes)?;
    Ok(Status {
        energy: Energy {
            now: word(bytes, 0x09c2)?,
            capacity: word(bytes, 0x09c4)?,
        },
        reserve: Reserve {
            energy: word(bytes, 0x09d6)?,
            mode: match word(bytes, 0x09c0)? {
                1 => ReserveMode::Auto,
                0 | 2 => ReserveMode::Manual,
                _ => return Err(HudInputFault::ReserveMode),
            },
        },
        ammunition: ammunition(bytes)?,
        equipped: equipped(bytes)?,
    })
}
