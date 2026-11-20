//! Typed imports of separately qualified actual unselected preparation completions.
use game::production::frame::borrowed::{ActorEligibilityOrigin, ActorSlot, SpriteBuilderOrigin};
use game::production::frame::borrowed_preparation::{
    ActorEligibilityReceipt, ActorSlotIdentity, BuilderCompletionReceipt, DebugAction,
    DebugCompletionReceipt, DebugMode, DebugOrigin, DeferredRandomReceipt,
    PaletteCompletionReceipt, PaletteDisposition, PaletteOrigin, PaletteSlot, PhysicalPadEdges,
    RandomSeed, RequiredFirstSeedProduct, RequiredSecondSeedProductLow,
};
use serde::Deserialize;
use std::fmt::Write;

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
    /// Separate terminal content does not match its admitted source member.
    Content,
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
struct BuilderProjection {
    #[serde(flatten)]
    contract: Contract,
    dependency_builder_content: Vec<u8>,
    terminal_builder_content_sha256: String,
    terminal_cursor: String,
    terminal_prepared_flags: String,
    terminal_top_definition: String,
    terminal_bottom_definition: String,
    source_observed_low_backing_preserved: ContractFlag,
    adapter_clears_builder: ContractFlag,
}

/// Import the actual external builder result; never clear or generate builder contents.
/// Admission pins the separate terminal member and complete source opportunity.
/// Apply only at the full actual operation completion, revalidate the complete current
/// builder and consume once; later sprite appends use this actual terminal owner.
/// # Errors
/// Rejects malformed contracts, changed current backing and invalid terminal content.
pub fn builder(
    current: &SpriteBuilderOrigin,
    projection: &str,
    terminal_content: &[u8],
) -> Result<BuilderCompletionReceipt, PreparationFault> {
    let receipt: BuilderProjection =
        toml::from_str(projection).map_err(|_| PreparationFault::Format)?;
    receipt.contract.validate("BeginSpriteList")?;
    if receipt.terminal_cursor != "Beginning"
        || receipt.terminal_prepared_flags != "None"
        || receipt.terminal_top_definition != "None"
        || receipt.terminal_bottom_definition != "None"
        || !receipt.source_observed_low_backing_preserved.0
        || receipt.adapter_clears_builder.0
    {
        return Err(PreparationFault::Unsupported);
    }
    if receipt.dependency_builder_content.as_slice() != current.backing {
        return Err(PreparationFault::Dependency);
    }
    let backing: [u8; 544] = terminal_content
        .try_into()
        .map_err(|_| PreparationFault::Length)?;
    let mut digest = String::with_capacity(64);
    for byte in data::rom::sha256(terminal_content) {
        write!(&mut digest, "{byte:02x}").map_err(|_| PreparationFault::Content)?;
    }
    if digest != receipt.terminal_builder_content_sha256 {
        return Err(PreparationFault::Content);
    }
    if !backing
        .iter()
        .take(512)
        .eq(current.backing.iter().take(512))
    {
        return Err(PreparationFault::Content);
    }
    Ok(BuilderCompletionReceipt {
        dependency: current.clone(),
        terminal: SpriteBuilderOrigin {
            backing,
            cursor: 0,
            prepared_flags: 0,
            prepared_top: 0,
            prepared_bottom: 0,
        },
    })
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
#[derive(Deserialize)]
struct Product {
    role: String,
    required_actual_result_encoding: u16,
    result_consumed_only_after_actual_device_completion: ContractFlag,
}
#[derive(Deserialize)]
struct RandomProjection {
    #[serde(flatten)]
    contract: Contract,
    required_current_seed: u16,
    terminal_seed: u16,
    requires_actual_first_completed_product_identity: ContractFlag,
    requires_actual_second_completed_product_identity: ContractFlag,
    first_product_preserved_across_second_request: ContractFlag,
    source_hardware_result_values_used_as_device_inputs: ContractFlag,
    actual_product_dependency: Vec<Product>,
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

/// Import an external random offer without supplying any actual hardware result.
/// Required product quantities remain predicates in distinct types, never device inputs.
/// Retain unread until full actual physical completion; the game consumes this offer
/// only after revalidating current seed and actual completed samples.
/// # Errors
/// Rejects malformed projections, changed current seed and unsupported dependency contracts.
pub fn random_offer(
    current: RandomSeed,
    projection: &str,
) -> Result<DeferredRandomReceipt, PreparationFault> {
    let receipt: RandomProjection =
        toml::from_str(projection).map_err(|_| PreparationFault::Format)?;
    receipt.contract.validate("Random")?;
    if !receipt.requires_actual_first_completed_product_identity.0
        || !receipt.requires_actual_second_completed_product_identity.0
        || !receipt.first_product_preserved_across_second_request.0
        || receipt
            .source_hardware_result_values_used_as_device_inputs
            .0
        || receipt.actual_product_dependency.len() != 2
    {
        return Err(PreparationFault::Unsupported);
    }
    if receipt.required_current_seed != current.0 {
        return Err(PreparationFault::Dependency);
    }
    for (dependency, role) in receipt
        .actual_product_dependency
        .iter()
        .zip(["FirstCompletedProduct", "SecondCompletedProductLowByte"])
    {
        if dependency.role != role
            || !dependency
                .result_consumed_only_after_actual_device_completion
                .0
        {
            return Err(PreparationFault::Unsupported);
        }
    }
    let mut dependencies = receipt.actual_product_dependency.into_iter();
    let first = dependencies.next().ok_or(PreparationFault::Unsupported)?;
    let second = dependencies.next().ok_or(PreparationFault::Unsupported)?;
    Ok(DeferredRandomReceipt {
        dependency: current,
        required_first: RequiredFirstSeedProduct(first.required_actual_result_encoding),
        required_second_low: RequiredSecondSeedProductLow(
            u8::try_from(second.required_actual_result_encoding)
                .map_err(|_| PreparationFault::Unsupported)?,
        ),
        terminal: RandomSeed(receipt.terminal_seed),
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
