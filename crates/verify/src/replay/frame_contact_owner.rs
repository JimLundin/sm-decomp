//! Accepted Contact envelope and independently supplied current owner seeds.

use super::frame_origin::profiles::{ProfileName, Templates};
use game::production::frame::{
    contact_contract::{Context, Current, PhysicalContract, Surface},
    sender_contract::BusRegime,
};

/// Incomplete current provenance or a different physical Contact envelope.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fault {
    /// Current source image or accepted projection differs.
    Source,
    /// Complete source envelope differs or is ambiguous.
    Envelope,
}

/// Import the exact independently qualified whole Contact portion.
/// Source coordinates terminate here; actual current inputs select runtime admission.
/// # Errors
/// Missing, repeated, split or different qualified portion.
pub fn physical_contract(templates: &Templates) -> Result<PhysicalContract, Fault> {
    let mut portions = templates
        .portions
        .iter()
        .filter(|part| part.semantic_owner == "S4.Contact");
    let part = portions.next().ok_or(Fault::Envelope)?;
    if portions.next().is_some()
        || part.profile != ProfileName::PreWait
        || part.first_exposure != 11070
        || part.end_exposure != 11200
        || part.receipt_contract != "S4.Contact"
        || !part.completed_producer_receipts.is_empty()
    {
        return Err(Fault::Envelope);
    }
    let mut profiles = templates
        .profiles
        .iter()
        .filter(|profile| profile.name == ProfileName::PreWait);
    let profile = profiles.next().ok_or(Fault::Envelope)?;
    if profiles.next().is_some() {
        return Err(Fault::Envelope);
    }
    let template = profile
        .slice(part.first_exposure, part.end_exposure)
        .map_err(|_| Fault::Envelope)?
        .template()
        .map_err(|_| Fault::Envelope)?;
    Ok(PhysicalContract::new(template))
}

#[derive(serde::Deserialize)]
struct Source {
    format: String,
    #[serde(flatten)]
    graphics: GraphicContact,
    sound_suppressors_follow_graphics: bool,
    empty_atmospherics_is_predicate_authority: bool,
    current_contact_predicate: Vec<Descriptor>,
}

#[derive(serde::Deserialize)]
struct GraphicContact {
    actual_graphical_contact_reached: bool,
    actual_graphical_contact_identity_must_join: bool,
    actual_appearance_boost_stage_must_join: bool,
}

#[derive(serde::Deserialize)]
struct Descriptor {
    name: String,
    private_wram_offset: usize,
    quantity_width: usize,
    private_current_encoding: u16,
    origin_equal_at_opportunity: bool,
    preceding_writer_indices: Vec<usize>,
    authority: String,
}

fn word(bytes: &[u8], offset: usize) -> Result<u16, Fault> {
    let end = offset.checked_add(2).ok_or(Fault::Source)?;
    let raw: [u8; 2] = bytes
        .get(offset..end)
        .ok_or(Fault::Source)?
        .try_into()
        .map_err(|_| Fault::Source)?;
    Ok(u16::from_le_bytes(raw))
}

/// Seed explicit current room/liquid owners from the accepted independent origin.
/// `room` is the checked content import's binding, never a guessed native pointer ID.
/// External suppressors must come from the accepted appearance-neighbour importer.
/// Callers retain these actual owners and revalidate them at Contact and Part2;
/// this source-origin importer must never read a later expected cut as an input.
/// # Errors
/// Changed accepted projection, incomplete origin or missing current-owner provenance.
pub fn current_origin(
    bytes: &[u8],
    text: &str,
    suppressors: types::Set<samus_appearance::ContactSuppressor>,
    room: data::id::RoomId,
    bus: BusRegime,
) -> Result<Current, Fault> {
    use crate::lock::Sha256;
    if bytes.len() != 131_072
        || Sha256::of(text.as_bytes())
            != Sha256::try_from("4e94ab4122a34a24d88e903ac0bbbf4286ac5876ff7cd0b33d6f4329bf44d48b")
                .map_err(|_| Fault::Source)?
    {
        return Err(Fault::Source);
    }
    let source: Source = toml::from_str(text).map_err(|_| Fault::Source)?;
    if source.format != "actual-current-contact-camera-tiles-source-v2"
        || !source.graphics.actual_graphical_contact_reached
        || !source.sound_suppressors_follow_graphics
        || !source.graphics.actual_graphical_contact_identity_must_join
        || !source.graphics.actual_appearance_boost_stage_must_join
        || source.empty_atmospherics_is_predicate_authority
        || source.current_contact_predicate.len() != 6
    {
        return Err(Fault::Source);
    }
    for name in [
        "CurrentAreaCeres",
        "CurrentAppearanceBoostStageJoinOnly",
        "CurrentRoomIdentity",
        "CurrentWaterSurface",
        "CurrentLavaAcidSurface",
        "CurrentLiquidOptions",
    ] {
        let mut fields = source
            .current_contact_predicate
            .iter()
            .filter(|field| field.name == name);
        let field = fields.next().ok_or(Fault::Source)?;
        if fields.next().is_some()
            || field.quantity_width != 2
            || !field.origin_equal_at_opportunity
            || !field.preceding_writer_indices.is_empty()
            || field.authority != "ActualCurrentNeighborOwnerRevalidatedAtGraphicalPart2"
        {
            return Err(Fault::Source);
        }
        // Appearance's boost remains its actual clean owner; native boost never seeds it.
        if name != "CurrentAppearanceBoostStageJoinOnly"
            && word(bytes, field.private_wram_offset)? != field.private_current_encoding
        {
            return Err(Fault::Source);
        }
    }
    let room_field = source
        .current_contact_predicate
        .iter()
        .find(|field| field.name == "CurrentRoomIdentity")
        .ok_or(Fault::Source)?;
    let actual_room = import::world::id(data::room::RoomKey::from_room_pointer(word(
        bytes,
        room_field.private_wram_offset,
    )?))
    .ok_or(Fault::Source)?;
    if actual_room != room {
        return Err(Fault::Source);
    }
    let surface = |name: &str| -> Result<Surface, Fault> {
        let field = source
            .current_contact_predicate
            .iter()
            .find(|field| field.name == name)
            .ok_or(Fault::Source)?;
        Ok(if word(bytes, field.private_wram_offset)? & 0x8000 != 0 {
            Surface::Absent
        } else {
            Surface::Present
        })
    };
    let area = source
        .current_contact_predicate
        .iter()
        .find(|field| field.name == "CurrentAreaCeres")
        .ok_or(Fault::Source)?;
    let area = u8::try_from(word(bytes, area.private_wram_offset)?).map_err(|_| Fault::Source)?;
    if data::room::Area::try_from(area).map_err(|_| Fault::Source)? != data::room::Area::Ceres {
        return Err(Fault::Source);
    }
    Ok(Current {
        suppressors,
        context: Context {
            room,
            area: types::Area::Ceres,
            water: surface("CurrentWaterSurface")?,
            lava: surface("CurrentLavaAcidSurface")?,
        },
        bus,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_profile_name_without_its_whole_contact_portion_is_not_admission() {
        let templates = Templates {
            profiles: Vec::new(),
            portions: Vec::new(),
            materializations: Vec::new(),
        };
        assert_eq!(physical_contract(&templates), Err(Fault::Envelope));
    }

    #[test]
    fn an_unpinned_current_projection_cannot_seed_room_owners() {
        assert_eq!(
            current_origin(
                &vec![0; 131_072],
                "",
                types::Set::default(),
                data::id::RoomId(0),
                BusRegime::Fast
            ),
            Err(Fault::Source)
        );
    }
}
