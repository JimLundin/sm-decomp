//! Typed imports of separately qualified actual unselected preparation completions.
use game::production::frame::borrowed::{ActorEligibilityOrigin, ActorSlot};
use game::production::frame::borrowed_preparation::{
    ActorEligibilityReceipt, ActorSlotIdentity, DebugAction, DebugCompletionReceipt, DebugMode,
    DebugOrigin, PaletteCompletionReceipt, PaletteDisposition, PaletteOrigin, PaletteSlot,
    PhysicalPadEdges,
};
use game::production::frame::random::RandomSeed;
use serde::Deserialize;

/// Invalid current origin, external completion or physical result dependency.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PreparationFault {
    /// Malformed external projection.
    Format,
    /// Incomplete current origin or completed content.
    Length,
    /// Missing or changed complete actual dependency.
    Dependency,
    /// Unsupported owner arm, terminal fact or consumption contract.
    Unsupported,
}
#[derive(Deserialize)]
#[serde(transparent)]
struct ContractFlag(bool);
#[derive(Deserialize)]
struct Contract {
    format: String,
    owner: String,
    requires_actual_current_dependency_equality: ContractFlag,
    read_only_at_actual_full_operation_completion: ContractFlag,
    consume_once: ContractFlag,
    expected_callbacks_used_as_inputs: ContractFlag,
}
impl Contract {
    fn validate(&self, owner: &str) -> Result<(), PreparationFault> {
        if self.format != "borrowed-prelude-opportunity-source-v1"
            || self.owner != owner
            || !self.requires_actual_current_dependency_equality.0
            || !self.read_only_at_actual_full_operation_completion.0
            || !self.consume_once.0
            || self.expected_callbacks_used_as_inputs.0
        {
            return Err(PreparationFault::Unsupported);
        }
        Ok(())
    }
}
#[derive(Deserialize)]
struct ActorProjection {
    #[serde(flatten)]
    contract: Contract,
    required_policy: String,
    required_slots: Vec<String>,
    remaining_slots: String,
    terminal_active_slot_indices: Vec<u8>,
    terminal_interactive_slot_indices: Vec<u8>,
    ai_or_geometry_authorized: ContractFlag,
}

/// Import actual completed membership vectors, never traverse to generate membership.
/// Revalidate raw current eligibility, consume once after the full actual operation,
/// and retain actual slot identity/order; these lists authorize no AI or geometry.
/// # Errors
/// Rejects malformed projections, changed eligibility and unsupported membership facts.
pub fn actors(
    current: &ActorEligibilityOrigin,
    projection: &str,
) -> Result<ActorEligibilityReceipt, PreparationFault> {
    let receipt: ActorProjection =
        toml::from_str(projection).map_err(|_| PreparationFault::Format)?;
    receipt.contract.validate("ActorEligibility")?;
    if receipt.required_policy != "PermitMarkedOffscreen"
        || receipt.remaining_slots != "ThirtyEmpty"
        || receipt.required_slots.as_slice() != ["OccupiedIntangible", "OccupiedTangible"]
        || receipt.ai_or_geometry_authorized.0
        || receipt.terminal_active_slot_indices.as_slice() != [0, 1]
        || receipt.terminal_interactive_slot_indices.as_slice() != [1]
    {
        return Err(PreparationFault::Unsupported);
    }
    let mut expected = [ActorSlot::Empty; 32];
    let (first, remaining) = expected
        .split_first_mut()
        .ok_or(PreparationFault::Dependency)?;
    *first = ActorSlot::Occupied {
        deleted: false,
        process_offscreen: true,
        intangible: true,
    };
    let second = remaining.first_mut().ok_or(PreparationFault::Dependency)?;
    *second = ActorSlot::Occupied {
        deleted: false,
        process_offscreen: true,
        intangible: false,
    };
    if !current.permit_marked_offscreen || current.slots != expected {
        return Err(PreparationFault::Dependency);
    }
    Ok(ActorEligibilityReceipt {
        dependency: current.clone(),
        active: receipt
            .terminal_active_slot_indices
            .into_iter()
            .map(ActorSlotIdentity)
            .collect(),
        interactive: receipt
            .terminal_interactive_slot_indices
            .into_iter()
            .map(ActorSlotIdentity)
            .collect(),
    })
}
/// Import the actual current random seed from complete initial WRAM.
/// # Errors
/// Rejects incomplete current origin memory.
pub fn random_origin(bytes: &[u8]) -> Result<RandomSeed, PreparationFault> {
    if bytes.len() != 131_072 {
        return Err(PreparationFault::Length);
    }
    Ok(RandomSeed(u16::from_le_bytes([
        bytes.get(1509).copied().ok_or(PreparationFault::Length)?,
        bytes.get(1510).copied().ok_or(PreparationFault::Length)?,
    ])))
}

fn word(bytes: &[u8], at: usize) -> Result<u16, PreparationFault> {
    Ok(u16::from_le_bytes([
        bytes.get(at).copied().ok_or(PreparationFault::Length)?,
        bytes
            .get(at.wrapping_add(1))
            .copied()
            .ok_or(PreparationFault::Length)?,
    ]))
}

/// Import actual physical debug inputs, before the later owner operation.
/// # Errors
/// Rejects incomplete origins and unqualified debug modes or physical button premises.
pub fn debug_origin(bytes: &[u8]) -> Result<DebugOrigin, PreparationFault> {
    if bytes.len() != 131_072 {
        return Err(PreparationFault::Length);
    }
    let controller_one = PhysicalPadEdges(word(bytes, 1477)?);
    let controller_two = PhysicalPadEdges(word(bytes, 145)?);
    if word(bytes, 6236)? != 0 || controller_two.0 != 0 || controller_one.0 & 0x00c0 != 0 {
        return Err(PreparationFault::Dependency);
    }
    Ok(DebugOrigin {
        mode: DebugMode::Normal,
        controller_one,
        controller_two,
    })
}

/// Import actual palette owner enable and ordered physical slot occupancy.
/// No current or future palette bytes become object eligibility inputs.
/// # Errors
/// Rejects incomplete origins, disabled owners and occupied or unknown palette slots.
pub fn palette_origin(bytes: &[u8]) -> Result<PaletteOrigin, PreparationFault> {
    if bytes.len() != 131_072 {
        return Err(PreparationFault::Length);
    }
    if word(bytes, 7801)? & 0x8000 == 0 {
        return Err(PreparationFault::Dependency);
    }
    for at in [7819, 7817, 7815, 7813, 7811, 7809, 7807, 7805] {
        if word(bytes, at)? != 0 {
            return Err(PreparationFault::Dependency);
        }
    }
    Ok(PaletteOrigin {
        enabled: true,
        slots: [PaletteSlot::Empty; 8],
    })
}

#[derive(Deserialize)]
struct DebugProjection {
    #[serde(flatten)]
    contract: Contract,
    required_mode: String,
    terminal_mode: String,
    terminal_debug_action: String,
    terminal_skip_processing: ContractFlag,
    physical_debug_buttons_are_movement_controls: ContractFlag,
}

/// Import the actual debug no-action completion from its qualified external owner.
/// Physical pad edges remain independent of movement-control mapping.
/// # Errors
/// Rejects malformed contracts, unsupported physical debug premises and terminal actions.
pub fn debug(
    current: DebugOrigin,
    projection: &str,
) -> Result<DebugCompletionReceipt, PreparationFault> {
    let receipt: DebugProjection =
        toml::from_str(projection).map_err(|_| PreparationFault::Format)?;
    receipt.contract.validate("Debug")?;
    if receipt.required_mode != "Normal"
        || receipt.terminal_mode != "Normal"
        || receipt.terminal_debug_action != "None"
        || receipt.terminal_skip_processing.0
        || receipt.physical_debug_buttons_are_movement_controls.0
    {
        return Err(PreparationFault::Unsupported);
    }
    if current.controller_two.0 != 0 || current.controller_one.0 & 0x00c0 != 0 {
        return Err(PreparationFault::Dependency);
    }
    Ok(DebugCompletionReceipt {
        dependency: current,
        terminal: DebugMode::Normal,
        action: DebugAction::None,
        skip_processing: false,
    })
}
#[derive(Deserialize)]
struct PaletteProjection {
    #[serde(flatten)]
    contract: Contract,
    required_owner: String,
    required_slots: String,
    terminal_palette: String,
    palette_replaced_with_future_buffer: ContractFlag,
    active_palette_objects_authorized: ContractFlag,
}

/// Import actual empty-object completion while retaining the current actual palette.
/// No palette buffer is copied, generated or replaced by this receipt.
/// # Errors
/// Rejects malformed contracts, changed eligibility and unsupported palette replacement.
pub fn palette(
    current: PaletteOrigin,
    projection: &str,
) -> Result<PaletteCompletionReceipt, PreparationFault> {
    let receipt: PaletteProjection =
        toml::from_str(projection).map_err(|_| PreparationFault::Format)?;
    receipt.contract.validate("PaletteObjects")?;
    if receipt.required_owner != "Enabled"
        || receipt.required_slots != "EightEmptyDescending"
        || receipt.terminal_palette != "ActualCurrentPaletteUnchanged"
        || receipt.palette_replaced_with_future_buffer.0
        || receipt.active_palette_objects_authorized.0
    {
        return Err(PreparationFault::Unsupported);
    }
    if !current.enabled || current.slots != [PaletteSlot::Empty; 8] {
        return Err(PreparationFault::Dependency);
    }
    Ok(PaletteCompletionReceipt {
        dependency: current,
        terminal: PaletteDisposition::RetainCurrent,
    })
}
