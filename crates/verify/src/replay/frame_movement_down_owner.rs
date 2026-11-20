//! Independently accepted current Down source and whole physical bindings.

use super::frame_origin::profiles::{Effect, Profile, ProfileName, Templates};
use game::production::frame::movement_down_contract::{PhysicalContract, SourceAdmission};

/// Invalid current Room source or incomplete whole physical task.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fault {
    /// Frozen source or actual current Room linkage differs.
    Source,
    /// Whole profile portions or active arithmetic bindings differ.
    Profile,
}
#[derive(serde::Deserialize)]
struct Source {
    room: RoomSource,
    physical_owner: MovementEnvelope,
    constituent_portion: Vec<Part>,
    arithmetic_binding: Vec<Arithmetic>,
}
#[derive(serde::Deserialize)]
struct RoomSource {
    width: u16,
    height: u16,
    room_pointer: u16,
    level_words: Vec<u16>,
    bts: Vec<u8>,
}
#[derive(serde::Deserialize)]
struct MovementEnvelope {
    owner: String,
    profile: ProfileName,
    first_exposure: u64,
    end_exposure: u64,
    intrinsic_ticks: u64,
    arithmetic_task: u32,
    source_matches_frozen_profile: bool,
}
#[derive(serde::Deserialize)]
struct Part {
    profile: ProfileName,
    first_exposure: u64,
    end_exposure: u64,
    semantic_owner: String,
    receipt_contract: String,
    completed_producer_receipts: Vec<String>,
}
#[derive(serde::Deserialize)]
struct Arithmetic {
    kind: String,
    profile: ProfileName,
    exposure: u64,
    device: String,
    producer: Option<String>,
    task_ordinal: u32,
    actual_request_or_hardware_sample_required: bool,
}

/// Decode the complete existing Room and qualify the independent Down source delta.
/// Source geometry/result witnesses never become runtime seeds or answers.
/// # Errors
/// Different source pin, actual current Room or complete physical bindings.
pub fn contract(
    origin: &[u8],
    native: &super::movement::Content,
    templates: &Templates,
    text: &str,
    room_source: &str,
) -> Result<PhysicalContract, Fault> {
    use crate::lock::Sha256;
    if Sha256::of(text.as_bytes())
        != Sha256::try_from("1932bbd082066b0559362f722a31ec8e4fcc2ddfd8c16df3fada523a36f55798")
            .map_err(|_| Fault::Source)?
    {
        return Err(Fault::Source);
    }
    let source: Source = toml::from_str(text).map_err(|_| Fault::Source)?;
    let admission =
        super::frame_movement_pair_owner::source_admission(origin, native, templates, room_source)
            .map_err(|_| Fault::Source)?;
    if crate::source::ROOM_WIDTH_IN_BLOCKS
        .read(origin)
        .map_err(|_| Fault::Source)?
        != source.room.width
        || crate::source::ROOM_HEIGHT_IN_BLOCKS
            .read(origin)
            .map_err(|_| Fault::Source)?
            != source.room.height
        || crate::source::ROOM_PTR
            .read(origin)
            .map_err(|_| Fault::Source)?
            != source.room.room_pointer
    {
        return Err(Fault::Source);
    }
    let count = usize::from(source.room.width)
        .checked_mul(usize::from(source.room.height))
        .ok_or(Fault::Source)?;
    let level = crate::source::LEVEL_DATA
        .read(origin)
        .map_err(|_| Fault::Source)?;
    let words = level
        .get(..count.checked_mul(2).ok_or(Fault::Source)?)
        .ok_or(Fault::Source)?
        .as_chunks::<2>()
        .0
        .iter()
        .map(|bytes| u16::from_le_bytes(*bytes))
        .collect::<Vec<_>>();
    let bts = crate::source::BTS.read(origin).map_err(|_| Fault::Source)?;
    if words != source.room.level_words
        || bts.get(..count) != Some(source.room.bts.as_slice())
        || admission.room.width != source.room.width
        || admission.room.cells.len() != count
    {
        return Err(Fault::Source);
    }
    let whole = whole(templates, &source)?;
    arithmetic(&whole, &source)?;
    Ok(PhysicalContract::new(
        SourceAdmission {
            room: admission.room,
            plain_solid: words.iter().map(|word| word >> 12 == 8).collect(),
            raw_actor: admission.raw_actor,
        },
        whole.template().map_err(|_| Fault::Profile)?,
    ))
}

fn whole(templates: &Templates, source: &Source) -> Result<Profile, Fault> {
    let owner = &source.physical_owner;
    if owner.owner != "GroundFollowingDown"
        || owner.profile != ProfileName::PreWait
        || !owner.source_matches_frozen_profile
        || owner.arithmetic_task != 8
        || owner.intrinsic_ticks != 5312
        || source.constituent_portion.len() != 5
    {
        return Err(Fault::Profile);
    }
    let mut end = owner.first_exposure;
    for part in &source.constituent_portion {
        let mut matches = templates.portions.iter().filter(|actual| {
            actual.profile == part.profile && actual.first_exposure == part.first_exposure
        });
        let actual = matches.next().ok_or(Fault::Profile)?;
        if matches.next().is_some()
            || part.profile != owner.profile
            || part.first_exposure != end
            || part.end_exposure <= end
            || actual.end_exposure != part.end_exposure
            || actual.semantic_owner != part.semantic_owner
            || actual.receipt_contract != part.receipt_contract
            || actual.completed_producer_receipts != part.completed_producer_receipts
        {
            return Err(Fault::Profile);
        }
        end = part.end_exposure;
    }
    if end != owner.end_exposure {
        return Err(Fault::Profile);
    }
    let mut profiles = templates
        .profiles
        .iter()
        .filter(|profile| profile.name == owner.profile);
    let profile = profiles.next().ok_or(Fault::Profile)?;
    if profiles.next().is_some() {
        return Err(Fault::Profile);
    }
    profile
        .slice(owner.first_exposure, end)
        .map_err(|_| Fault::Profile)
}

fn arithmetic(profile: &Profile, source: &Source) -> Result<(), Fault> {
    if source.arithmetic_binding.len() != 4 {
        return Err(Fault::Profile);
    }
    let mut bindings = source.arithmetic_binding.iter();
    let mut exposure = source.physical_owner.first_exposure;
    for run in &profile.runs {
        for _ in 0..run.count {
            if !matches!(run.effect, Effect::None) {
                let binding = bindings.next().ok_or(Fault::Profile)?;
                if binding.profile != profile.name
                    || binding.exposure != exposure
                    || binding.task_ordinal != 8
                    || !binding.actual_request_or_hardware_sample_required
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
                    || task.name != "GroundFollowingFirstTerrainCell"
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
