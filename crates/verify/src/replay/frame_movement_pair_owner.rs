//! Current room provenance and complete `OrdinaryRight` physical admission at the edge.

use super::frame_origin::profiles::{Effect, Profile, ProfileName, Templates};
use game::production::frame::{
    movement_caller::{PairOwner, Room},
    movement_pair_contract::{PhysicalContract, SourceAdmission},
};

/// Invalid current source provenance or incomplete physical owner bindings.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fault {
    /// Source text, current room/classification or unchanged alignment guard differs.
    Source,
    /// Complete owner portions or arithmetic quantities/effects differ.
    Profile,
}

#[derive(serde::Deserialize)]
struct MovementContract {
    format: String,
    native_outputs_are_model_inputs: bool,
    source_indices_are_runtime_selectors: bool,
    #[serde(flatten)]
    dependencies: PairDependencies,
    physical_templates_once_consumed: bool,
    admission: Admission,
    room: RoomSource,
    support_alignment_source: Alignment,
    physical_owner: Vec<MovementEnvelope>,
    constituent_portion: Vec<Part>,
    arithmetic_binding: Vec<Arithmetic>,
}

#[derive(serde::Deserialize)]
struct PairDependencies {
    actual_completed_actor_owner_required: bool,
    actual_room_identity_required: bool,
    actual_pair_receipt_and_before_required: bool,
}
#[derive(serde::Deserialize)]
struct Admission {
    support_alignment: bool,
    raw_actor_identity: String,
    raw_actor_identity_is_validation_only: bool,
    plain_air_is_validation_only: bool,
}
#[derive(serde::Deserialize)]
struct RoomSource {
    width: u16,
    height: u16,
    room_pointer: u16,
    level_words: Vec<u16>,
    bts: Vec<u8>,
    plain_air: Vec<bool>,
    decoder: String,
    boundary: String,
    permission_decoder: String,
    origin_equal: bool,
    runtime_room_authority: String,
}
#[derive(serde::Deserialize)]
struct Alignment {
    private_wram_offset: usize,
    quantity_width: usize,
    source_current_sha256: String,
    origin_equal: bool,
    semantic_disposition: String,
}
#[derive(serde::Deserialize)]
struct MovementEnvelope {
    owner: String,
    profile: ProfileName,
    first_exposure: u64,
    end_exposure: u64,
    source_matches_frozen_profile: bool,
}
#[derive(serde::Deserialize)]
struct Part {
    owner: String,
    profile: ProfileName,
    first_exposure: u64,
    end_exposure: u64,
    semantic_owner: String,
    receipt_contract: String,
    completed_producer_receipts: Vec<String>,
}
#[derive(serde::Deserialize)]
struct Arithmetic {
    owner: String,
    kind: String,
    profile: ProfileName,
    exposure: u64,
    device: String,
    producer: Option<String>,
    task_ordinal: u32,
    actual_request_quantity_or_actual_hardware_sample_required: bool,
}

/// Decode actual current room inputs and validate the independent source projection.
/// Source room copies are equality predicates; the existing decoder consumes actual origin.
/// The root requires the packet's independent delta gate before admitting returned work.
/// # Errors
/// Different source text, current room/classification, arithmetic aliases or whole envelope.
pub fn ordinary_right(
    origin: &[u8],
    native: &super::movement::Content,
    templates: &Templates,
    text: &str,
) -> Result<(Room, PhysicalContract), Fault> {
    use crate::lock::Sha256;
    if Sha256::of(text.as_bytes())
        != Sha256::try_from("95f5550d884a19f507328c5c27cd7d211f8672903ece0db7a27d234f4558ac2c")
            .map_err(|_| Fault::Source)?
    {
        return Err(Fault::Source);
    }
    let source: MovementContract = toml::from_str(text).map_err(|_| Fault::Source)?;
    if source.format != "movement-source-admission-v1"
        || source.native_outputs_are_model_inputs
        || source.source_indices_are_runtime_selectors
        || !source.dependencies.actual_completed_actor_owner_required
        || !source.dependencies.actual_room_identity_required
        || !source.dependencies.actual_pair_receipt_and_before_required
        || !source.physical_templates_once_consumed
        || !source.admission.support_alignment
        || source.admission.raw_actor_identity != "Second"
        || !source.admission.raw_actor_identity_is_validation_only
        || !source.admission.plain_air_is_validation_only
        || !source.room.origin_equal
        || source.room.decoder != "ExistingCheckedPlayerRomMovementWorldDecoder"
        || source.room.runtime_room_authority != "ActualCurrentRoomOwner"
        || source.room.permission_decoder != "SpecialCommon45Only"
        || source.room.boundary != "Air"
        || !source.support_alignment_source.origin_equal
        || source.support_alignment_source.quantity_width != 2
        || source.support_alignment_source.semantic_disposition != "Enabled"
    {
        return Err(Fault::Source);
    }
    let alignment = &source.support_alignment_source;
    let end = alignment
        .private_wram_offset
        .checked_add(alignment.quantity_width)
        .ok_or(Fault::Source)?;
    let bytes = origin
        .get(alignment.private_wram_offset..end)
        .ok_or(Fault::Source)?;
    if Sha256::of(bytes)
        != Sha256::try_from(alignment.source_current_sha256.as_str()).map_err(|_| Fault::Source)?
    {
        return Err(Fault::Source);
    }
    let plain_air = room_cells(origin, &source.room)?;
    let (width, cells, permissions) =
        super::movement::level_blocks(origin, native).map_err(|_| Fault::Source)?;
    let room = Room {
        identity: Some(
            import::world::id(data::room::RoomKey::from_room_pointer(
                crate::source::ROOM_PTR
                    .read(origin)
                    .map_err(|_| Fault::Source)?,
            ))
            .ok_or(Fault::Source)?,
        ),
        width,
        cells,
        permissions,
        boundary: game::production::world::Boundary::Air,
    };
    let profile = whole(templates, &source, "OrdinaryRight")?;
    arithmetic(&profile, &source)?;
    let contract = PhysicalContract::new(
        PairOwner::OrdinaryRight,
        SourceAdmission {
            room: room.clone(),
            plain_air,
            raw_actor: game::production::frame::borrowed_preparation::ActorSlotIdentity(1),
            support_alignment: source.admission.support_alignment,
        },
        profile.template().map_err(|_| Fault::Profile)?,
    );
    Ok((room, contract))
}

fn room_cells(origin: &[u8], room: &RoomSource) -> Result<Vec<bool>, Fault> {
    if crate::source::ROOM_WIDTH_IN_BLOCKS
        .read(origin)
        .map_err(|_| Fault::Source)?
        != room.width
        || crate::source::ROOM_HEIGHT_IN_BLOCKS
            .read(origin)
            .map_err(|_| Fault::Source)?
            != room.height
        || crate::source::ROOM_PTR
            .read(origin)
            .map_err(|_| Fault::Source)?
            != room.room_pointer
    {
        return Err(Fault::Source);
    }
    let count = usize::from(room.width)
        .checked_mul(usize::from(room.height))
        .ok_or(Fault::Source)?;
    let raw = crate::source::LEVEL_DATA
        .read(origin)
        .map_err(|_| Fault::Source)?;
    let words = raw
        .get(..count.checked_mul(2).ok_or(Fault::Source)?)
        .ok_or(Fault::Source)?
        .as_chunks::<2>()
        .0
        .iter()
        .map(|bytes| u16::from_le_bytes(*bytes))
        .collect::<Vec<_>>();
    let bts = crate::source::BTS
        .read(origin)
        .map_err(|_| Fault::Source)?
        .get(..count)
        .ok_or(Fault::Source)?;
    let plain_air = words.iter().map(|word| word >> 12 == 0).collect::<Vec<_>>();
    if words != room.level_words || bts != room.bts || plain_air != room.plain_air {
        return Err(Fault::Source);
    }
    Ok(plain_air)
}

/// Share the same independently checked current room and admission predicates.
/// This reuses the existing complete source/ROM decoder, never a second room format.
/// # Errors
/// Any current source or immutable physical binding differs from the admitted packet.
pub fn source_admission(
    origin: &[u8],
    native: &super::movement::Content,
    templates: &Templates,
    text: &str,
) -> Result<SourceAdmission, Fault> {
    let (room, _) = ordinary_right(origin, native, templates, text)?;
    let source: MovementContract = toml::from_str(text).map_err(|_| Fault::Source)?;
    Ok(SourceAdmission {
        room,
        plain_air: source.room.plain_air,
        raw_actor: game::production::frame::borrowed_preparation::ActorSlotIdentity(1),
        support_alignment: source.admission.support_alignment,
    })
}

fn whole(templates: &Templates, source: &MovementContract, owner: &str) -> Result<Profile, Fault> {
    let mut owners = source
        .physical_owner
        .iter()
        .filter(|candidate| candidate.owner == owner);
    let required = owners.next().ok_or(Fault::Profile)?;
    if owners.next().is_some()
        || required.profile != ProfileName::PreWait
        || !required.source_matches_frozen_profile
    {
        return Err(Fault::Profile);
    }
    let portions = source
        .constituent_portion
        .iter()
        .filter(|part| part.owner == owner)
        .collect::<Vec<_>>();
    if portions.is_empty() {
        return Err(Fault::Profile);
    }
    let mut end = required.first_exposure;
    for part in portions {
        let mut matching = templates.portions.iter().filter(|actual| {
            actual.profile == part.profile && actual.first_exposure == part.first_exposure
        });
        let actual = matching.next().ok_or(Fault::Profile)?;
        if matching.next().is_some()
            || actual.end_exposure != part.end_exposure
            || actual.semantic_owner != part.semantic_owner
            || actual.receipt_contract != part.receipt_contract
            || actual.completed_producer_receipts != part.completed_producer_receipts
            || part.first_exposure != end
            || part.end_exposure <= end
        {
            return Err(Fault::Profile);
        }
        end = part.end_exposure;
    }
    if end != required.end_exposure {
        return Err(Fault::Profile);
    }
    let mut profiles = templates
        .profiles
        .iter()
        .filter(|profile| profile.name == required.profile);
    let profile = profiles.next().ok_or(Fault::Profile)?;
    if profiles.next().is_some() {
        return Err(Fault::Profile);
    }
    profile
        .slice(required.first_exposure, end)
        .map_err(|_| Fault::Profile)
}

fn arithmetic(profile: &Profile, source: &MovementContract) -> Result<(), Fault> {
    let bindings = source
        .arithmetic_binding
        .iter()
        .filter(|binding| binding.owner == "OrdinaryRight")
        .collect::<Vec<_>>();
    if bindings.len() != 4 {
        return Err(Fault::Profile);
    }
    let mut bindings = bindings.into_iter();
    let first = source
        .physical_owner
        .iter()
        .find(|owner| owner.owner == "OrdinaryRight")
        .ok_or(Fault::Profile)?
        .first_exposure;
    let mut exposure = first;
    for run in &profile.runs {
        for _ in 0..run.count {
            if !matches!(run.effect, Effect::None) {
                let binding = bindings.next().ok_or(Fault::Profile)?;
                if binding.profile != profile.name
                    || binding.exposure != exposure
                    || binding.task_ordinal != 5
                    || !binding.actual_request_quantity_or_actual_hardware_sample_required
                {
                    return Err(Fault::Profile);
                }
                let (device, task) = match &run.effect {
                    Effect::Publication(publication)
                        if binding.kind == "publication"
                            && binding.producer.as_ref() == Some(&publication.producer) =>
                    {
                        (publication.device.as_str(), publication.arithmetic.as_ref())
                    }
                    Effect::Sample(data::timing::Sample::ProductLow, task)
                        if binding.kind == "sample" =>
                    {
                        ("ProductLow", task.as_ref())
                    }
                    Effect::Sample(data::timing::Sample::ProductHigh, task)
                        if binding.kind == "sample" =>
                    {
                        ("ProductHigh", task.as_ref())
                    }
                    Effect::None | Effect::Publication(_) | Effect::Sample(_, _) => {
                        return Err(Fault::Profile);
                    }
                };
                let task = task.ok_or(Fault::Profile)?;
                if device != binding.device
                    || task.name != "OrdinaryRightFirstTerrainCell"
                    || task.quantities != ["CellRow", "RoomWidth"]
                {
                    return Err(Fault::Profile);
                }
            }
            exposure = exposure.checked_add(1).ok_or(Fault::Profile)?;
        }
    }
    if bindings.next().is_some() {
        return Err(Fault::Profile);
    }
    Ok(())
}
