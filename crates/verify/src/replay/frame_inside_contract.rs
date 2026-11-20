//! Checked whole Inside template at the edge; current source predicates require their own gate.

use super::frame_origin::profiles::{Effect, Profile, ProfileName, Templates};
use data::timing::{Sample, Template};
use game::production::frame::inside_contract::PhysicalContract;

#[cfg(test)]
mod tests;

/// Missing actual-current source qualification cannot be replaced by semantic navigation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fault {
    /// Accepted current source text or its field/body/room linkage is invalid.
    CurrentSource,
    /// Incomplete whole envelope or incompatible arithmetic namespaces.
    Profile,
}

#[derive(serde::Deserialize)]
struct Source {
    format: String,
    room_type_mask: String,
    current_guard: Vec<Scalar>,
    body_join: Vec<Scalar>,
    room_guard: Vec<RoomGuard>,
    query_binding: Vec<Query>,
    physical_portion: Vec<Part>,
}
#[derive(serde::Deserialize)]
struct Scalar {
    name: String,
    private_wram_offset: usize,
    quantity_width: usize,
    required_current_encoding: u16,
    source_writer_index: Option<u64>,
}
#[derive(serde::Deserialize)]
struct RoomGuard {
    name: String,
    private_wram_offset: usize,
    quantity_width: usize,
    required_current_sha256: String,
}
#[derive(serde::Deserialize)]
struct Query {
    producer: String,
    required_room_width_guard_encoding: u16,
    required_row_guard_encoding: u16,
    required_type_mask_encoding: u16,
    source_type_read_private_wram_offset: usize,
    source_type_read_byte_width: usize,
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

/// Import independently gated current geometry and a complete whole-owner physical contract.
/// Only unchanged current room content becomes an actual input; selected geometry is predicate-only.
/// # Errors
/// Wrong accepted source text, current bytes/body linkage, raw type nibble or whole owner.
pub fn contract(
    origin: &[u8],
    native: &super::movement::Content,
    templates: &Templates,
    text: &str,
) -> Result<
    (
        game::production::frame::inside_world::InsideWorld,
        PhysicalContract,
    ),
    Fault,
> {
    use crate::lock::Sha256;
    use game::production::frame::{
        inside_contract::RequiredDependencies, inside_world::InsideWorld,
    };
    if origin.len() != 131_072
        || Sha256::of(text.as_bytes())
            != Sha256::try_from("127725538b356cf300a488b173ec5ccf4d5925f8673227e566e5cff716929087")
                .map_err(|_| Fault::CurrentSource)?
    {
        return Err(Fault::CurrentSource);
    }
    let source: Source = toml::from_str(text).map_err(|_| Fault::CurrentSource)?;
    if source.format != "bounded-inside-current-physical-source-v1"
        || source.current_guard.len() != 4
        || source.body_join.len() != 2
        || source.room_guard.len() != 5
        || source.query_binding.len() != 3
    {
        return Err(Fault::CurrentSource);
    }
    for field in &source.current_guard {
        if field.quantity_width != 2
            || (field.name != "CurrentBodyHalfHeight"
                && word(origin, field.private_wram_offset)? != field.required_current_encoding)
        {
            return Err(Fault::CurrentSource);
        }
    }
    // These source facts were produced by the preceding completed pose-input owner.
    // They are predicates for its actual body, never inputs read from the initial origin.
    if source
        .body_join
        .iter()
        .any(|field| field.quantity_width != 2 || field.source_writer_index.is_none())
    {
        return Err(Fault::CurrentSource);
    }
    for field in &source.room_guard {
        let end = field
            .private_wram_offset
            .checked_add(field.quantity_width)
            .ok_or(Fault::CurrentSource)?;
        let bytes = origin
            .get(field.private_wram_offset..end)
            .ok_or(Fault::CurrentSource)?;
        if Sha256::of(bytes)
            != Sha256::try_from(field.required_current_sha256.as_str())
                .map_err(|_| Fault::CurrentSource)?
        {
            return Err(Fault::CurrentSource);
        }
    }
    let scalar = |name: &str| {
        let mut fields = source
            .current_guard
            .iter()
            .chain(&source.body_join)
            .filter(|field| field.name == name);
        let first = fields
            .next()
            .ok_or(Fault::CurrentSource)?
            .required_current_encoding;
        if fields.next().is_some() {
            return Err(Fault::CurrentSource);
        }
        Ok(first)
    };
    let pose = data::assets::samus::map::key_of(data::pose::PoseId(scalar("CurrentPose")?))
        .ok_or(Fault::CurrentSource)?;
    let radius = [scalar("SAMUS_X_RADIUS")?, scalar("SAMUS_Y_RADIUS")?];
    let body = native.movement.radius(pose).ok_or(Fault::CurrentSource)?;
    if [body.x.0, body.y.0] != radius || scalar("CurrentBodyHalfHeight")? != radius[1] {
        return Err(Fault::CurrentSource);
    }
    let (width, cells, _) =
        super::movement::level_blocks(origin, native).map_err(|_| Fault::CurrentSource)?;
    let key = data::room::RoomKey::from_room_pointer(
        crate::source::ROOM_PTR
            .read(origin)
            .map_err(|_| Fault::CurrentSource)?,
    );
    let identity = import::world::id(key).ok_or(Fault::CurrentSource)?;
    let room = InsideWorld::for_room(
        identity,
        width,
        cells,
        game::production::world::Boundary::Air,
    )
    .map_err(|_| Fault::CurrentSource)?;
    let pixels = [scalar("CurrentCentreX")?, scalar("CurrentCentreY")?];
    raw_queries(origin, &source, pixels, radius[1], width)?;
    parts(templates, &source.physical_portion)?;
    let physical = PhysicalContract::new(
        RequiredDependencies {
            pixels,
            pose,
            radius,
            room: Box::new(room.clone()),
        },
        whole_template(templates)?,
    );
    Ok((room, physical))
}

fn word(bytes: &[u8], at: usize) -> Result<u16, Fault> {
    let end = at.checked_add(2).ok_or(Fault::CurrentSource)?;
    let bytes = bytes.get(at..end).ok_or(Fault::CurrentSource)?;
    Ok(u16::from_le_bytes(
        bytes.try_into().map_err(|_| Fault::CurrentSource)?,
    ))
}

fn raw_queries(
    origin: &[u8],
    source: &Source,
    [x, y]: [u16; 2],
    radius: u16,
    width: u16,
) -> Result<(), Fault> {
    let bottom = y
        .checked_add(radius)
        .and_then(|edge| edge.checked_sub(1))
        .ok_or(Fault::CurrentSource)?;
    let top = y.checked_sub(radius).ok_or(Fault::CurrentSource)?;
    if x >= 4096 || bottom >= 4096 || top > y || y > bottom || width == 0 || width > 255 {
        return Err(Fault::CurrentSource);
    }
    let rows = [bottom >> 4, y >> 4, top >> 4];
    let [bottom, centre, top] = rows;
    if bottom == centre || bottom == top || centre == top || top == 0 || bottom ^ centre ^ top == 0
    {
        return Err(Fault::CurrentSource);
    }
    let mut levels = source
        .room_guard
        .iter()
        .filter(|field| field.name == "CurrentRoomLevelWords");
    let level = levels.next().ok_or(Fault::CurrentSource)?;
    if levels.next().is_some() || source.room_type_mask != "QueriedCellLowByteHighNibbleOnly" {
        return Err(Fault::CurrentSource);
    }
    for ((query, row), name) in source.query_binding.iter().zip(rows).zip([
        "InsideBlockQueryOne",
        "InsideBlockQueryTwo",
        "InsideBlockQueryThree",
    ]) {
        let cell = usize::from(row)
            .checked_mul(usize::from(width))
            .and_then(|row| row.checked_add(usize::from(x >> 4)))
            .ok_or(Fault::CurrentSource)?;
        let offset = cell
            .checked_mul(2)
            .and_then(|cell| cell.checked_add(1))
            .and_then(|cell| level.private_wram_offset.checked_add(cell))
            .ok_or(Fault::CurrentSource)?;
        let value = u16::from(*origin.get(offset).ok_or(Fault::CurrentSource)?);
        if query.producer != name
            || query.required_room_width_guard_encoding != width
            || query.required_row_guard_encoding != row
            || query.source_type_read_private_wram_offset != offset
            || query.source_type_read_byte_width != 2
            || query.required_type_mask_encoding != 0xf0
            || value & query.required_type_mask_encoding != 0
        {
            return Err(Fault::CurrentSource);
        }
    }
    Ok(())
}

fn parts(templates: &Templates, required: &[Part]) -> Result<(), Fault> {
    let actual = templates
        .portions
        .iter()
        .filter(|part| part.semantic_owner == "S2.InsideQueries")
        .collect::<Vec<_>>();
    if actual.len() != 14 || required.len() != 14 {
        return Err(Fault::Profile);
    }
    for (actual, required) in actual.iter().zip(required) {
        if actual.profile != required.profile
            || actual.first_exposure != required.first_exposure
            || actual.end_exposure != required.end_exposure
            || actual.semantic_owner != required.semantic_owner
            || actual.receipt_contract != required.receipt_contract
            || actual.completed_producer_receipts != required.completed_producer_receipts
        {
            return Err(Fault::Profile);
        }
    }
    Ok(())
}

/// Admit the complete fourteen-part physical Inside template once.
/// # Errors
/// Missing/overlapping owner parts, wrong whole boundaries or arithmetic namespaces.
pub fn whole_template(templates: &Templates) -> Result<Template, Fault> {
    let portions = templates
        .portions
        .iter()
        .filter(|part| part.semantic_owner == "S2.InsideQueries")
        .collect::<Vec<_>>();
    if portions.len() != 14 {
        return Err(Fault::Profile);
    }
    let mut end = 3846;
    for part in portions {
        if part.profile != ProfileName::PreWait
            || part.first_exposure != end
            || part.end_exposure <= end
        {
            return Err(Fault::Profile);
        }
        end = part.end_exposure;
    }
    if end != 4615 {
        return Err(Fault::Profile);
    }
    let mut profiles = templates
        .profiles
        .iter()
        .filter(|profile| profile.name == ProfileName::PreWait);
    let profile = profiles.next().ok_or(Fault::Profile)?;
    if profiles.next().is_some() {
        return Err(Fault::Profile);
    }
    let whole = profile.slice(3846, 4615).map_err(|_| Fault::Profile)?;
    arithmetic(&whole)?;
    whole.template().map_err(|_| Fault::Profile)
}

fn arithmetic(profile: &Profile) -> Result<(), Fault> {
    let mut publications = Vec::new();
    let mut samples = Vec::new();
    for run in &profile.runs {
        for _ in 0..run.count {
            match &run.effect {
                Effect::None => {}
                Effect::Publication(publication) => {
                    publications.push((
                        publication.device.as_str(),
                        publication.arithmetic.as_ref().ok_or(Fault::Profile)?,
                    ));
                }
                Effect::Sample(device, task) => {
                    samples.push((*device, task.as_ref().ok_or(Fault::Profile)?));
                }
            }
        }
    }
    if publications.len() != 9 || samples.len() != 6 {
        return Err(Fault::Profile);
    }
    let tasks = [
        "InsideBlockQueryOne",
        "InsideBlockQueryTwo",
        "InsideBlockQueryThree",
    ];
    for ((device, task), (expected_device, expected_task)) in
        publications.iter().zip(tasks.into_iter().flat_map(|name| {
            [
                ("Multiplicand", name),
                ("Multiplier", name),
                ("DividendLow", name),
            ]
        }))
    {
        if *device != expected_device
            || task.name != expected_task
            || task.quantities != ["RoomWidth", "CellRow", "Zero"]
        {
            return Err(Fault::Profile);
        }
    }
    for ((device, task), (expected_device, expected_task)) in samples.iter().zip(
        tasks
            .into_iter()
            .flat_map(|name| [(Sample::ProductLow, name), (Sample::ProductHigh, name)]),
    ) {
        if *device != expected_device
            || task.name != expected_task
            || task.quantities != ["RoomWidth", "CellRow", "Zero"]
        {
            return Err(Fault::Profile);
        }
    }
    Ok(())
}
