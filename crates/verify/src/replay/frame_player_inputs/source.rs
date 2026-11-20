use super::{PlayerInputFault, source};
use crate::lock::Sha256;
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(transparent)]
struct Flag(bool);

#[derive(Deserialize)]
struct Proof {
    format: String,
    opportunity_source: usize,
    required_previous_owner: String,
    appearance_figure_playhead_source_substitution_allowed: Flag,
    actual_playing_sprite_required: Flag,
    actual_completed_spin_and_clip_end_results_required: Flag,
    current_actor_geometry_authority: String,
    origin_roster_is_current_geometry_authority: Flag,
    revalidate_actual_world_and_control_dependencies: Flag,
    current_player_input: Vec<Descriptor>,
    world: World,
    remaining_actor_join: Actors,
}

#[derive(Deserialize)]
struct Descriptor {
    name: String,
    private_wram_offset: usize,
    quantity_width: usize,
    private_current_sha256: String,
    origin_equal_at_preparation: bool,
    preceding_writer_indices: Vec<usize>,
    movement_window_writer_indices: Vec<usize>,
    authority: String,
    physical_read_at_preparation_claimed: bool,
}

#[derive(Deserialize)]
struct World {
    room_content: String,
    cell_and_permission_codec: String,
    reaction_permission: String,
    immutable_slope_authority: String,
    current_world_is_future_output: bool,
}

#[derive(Deserialize)]
struct Actors {
    source_current_geometry_not_frontloaded: Flag,
    require_actual_completed_actor_owner_geometry: Flag,
    required_properties: String,
    source_only_scalar_roster_is_insufficient: Flag,
    actual_actor_geometry_model_or_typed_external_completion_receipt_required: Flag,
    new_actor_geometry_proof_in_this_packet: Flag,
}

const SCALARS: [(&str, source::WramWord); 9] = [
    ("JOYPAD1_LASTKEYS", source::JOYPAD1_LASTKEYS),
    ("JOYPAD1_NEWKEYS", source::JOYPAD1_NEWKEYS),
    ("EQUIPPED_ITEMS", source::EQUIPPED_ITEMS),
    ("CINEMATIC_ACTIVE", source::CINEMATIC_ACTIVE),
    ("ELEVATOR_STATUS", source::ELEVATOR_STATUS),
    ("NMI_FRAME_COUNTER_WORD", source::NMI_FRAME_COUNTER_WORD),
    ("ROOM_PTR", source::ROOM_PTR),
    ("ROOM_WIDTH_IN_BLOCKS", source::ROOM_WIDTH_IN_BLOCKS),
    ("ROOM_HEIGHT_IN_BLOCKS", source::ROOM_HEIGHT_IN_BLOCKS),
];

fn header(proof: &Proof) -> bool {
    proof.format == "bounded-current-player-preparation-v1"
        && proof.opportunity_source == 2189
        && proof.required_previous_owner == "ActualCompletedS2ActorAndSpriteObjectWork"
        && !proof
            .appearance_figure_playhead_source_substitution_allowed
            .0
        && proof.actual_playing_sprite_required.0
        && proof.actual_completed_spin_and_clip_end_results_required.0
        && proof.current_actor_geometry_authority == "ActualCompletedS2CurrentActorOwner"
        && !proof.origin_roster_is_current_geometry_authority.0
        && proof.revalidate_actual_world_and_control_dependencies.0
        && proof.world.room_content == "CheckedPlayerRomRoomDecodeForActualCurrentRoomPointer"
        && proof.world.cell_and_permission_codec
            == "ExistingMovementLevelWorldDecodedBlockKindAndCurrentBts"
        && proof.world.reaction_permission == "CurrentDecodedSpecialCommon45Only"
        && proof.world.immutable_slope_authority == "ExistingCheckedSlopes"
        && !proof.world.current_world_is_future_output
        && proof
            .remaining_actor_join
            .source_current_geometry_not_frontloaded
            .0
        && proof
            .remaining_actor_join
            .require_actual_completed_actor_owner_geometry
            .0
        && proof.remaining_actor_join.required_properties
            == "InteractiveCurrentRosterOccupiedSolidOrFrozenCurrentCentreAndRadius"
        && proof
            .remaining_actor_join
            .source_only_scalar_roster_is_insufficient
            .0
        && proof
            .remaining_actor_join
            .actual_actor_geometry_model_or_typed_external_completion_receipt_required
            .0
        && !proof
            .remaining_actor_join
            .new_actor_geometry_proof_in_this_packet
            .0
}

fn descriptor(
    bytes: &[u8],
    fields: &[Descriptor],
    name: &str,
    at: usize,
    width: usize,
) -> Result<(), PlayerInputFault> {
    let mut matches = fields.iter().filter(|field| field.name == name);
    let field = matches.next().ok_or(PlayerInputFault::Provenance)?;
    let current = bytes
        .get(at..at.saturating_add(width))
        .ok_or(PlayerInputFault::Field)?;
    let pin = Sha256::try_from(field.private_current_sha256.as_str())
        .map_err(|_| PlayerInputFault::Provenance)?;
    if matches.next().is_some()
        || field.private_wram_offset != at
        || field.quantity_width != width
        || !field.origin_equal_at_preparation
        || !field.preceding_writer_indices.is_empty()
        || !field.movement_window_writer_indices.is_empty()
        || field.authority != "ActualCurrentNeighborOwner"
        || field.physical_read_at_preparation_claimed
        || pin != Sha256::of(current)
    {
        return Err(PlayerInputFault::Provenance);
    }
    Ok(())
}

pub(super) fn validate(bytes: &[u8], proof: &str) -> Result<(), PlayerInputFault> {
    if bytes.len() != 131_072 {
        return Err(PlayerInputFault::Length);
    }
    let proof: Proof = toml::from_str(proof).map_err(|_| PlayerInputFault::Provenance)?;
    if !header(&proof) || proof.current_player_input.len() != 11 {
        return Err(PlayerInputFault::Provenance);
    }
    for (name, field) in SCALARS {
        descriptor(
            bytes,
            &proof.current_player_input,
            name,
            usize::from(field.offset),
            2,
        )?;
    }
    let width = usize::from(super::word(bytes, source::ROOM_WIDTH_IN_BLOCKS)?);
    let height = usize::from(super::word(bytes, source::ROOM_HEIGHT_IN_BLOCKS)?);
    let cells = width
        .checked_mul(height)
        .filter(|cells| *cells != 0)
        .ok_or(PlayerInputFault::Provenance)?;
    let words = cells.checked_mul(2).ok_or(PlayerInputFault::Provenance)?;
    descriptor(
        bytes,
        &proof.current_player_input,
        "CurrentRoomLevelWords",
        0x10002,
        words,
    )?;
    descriptor(
        bytes,
        &proof.current_player_input,
        "CurrentRoomBtsBytes",
        0x16402,
        cells,
    )
}

#[cfg(test)]
#[path = "source_tests.rs"]
mod tests;
