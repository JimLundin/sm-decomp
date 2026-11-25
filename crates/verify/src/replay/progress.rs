//! Fixed event visits are causal reports; event RAM is a separate expected observation.

use super::{compare::Observation, data_failure, result::Outcome};
use core::num::Wrapping;
use oracle_record::FrameRecord;
use run_progress::{MapStatus, Progress, Report, ReserveMode, Supply};
use types::{Area, Beam, BossKind, DoorId, Event, Item, LocationId, Set, StationId};

const PLANET_AWAKE_SETTER: u32 = 0x84_BE0B;
const EVENTS: [(u16, Event); 18] = [
    (0, Event::PlanetAwake),
    (2, Event::MotherBrainTankBroken),
    (6, Event::PhantoonStatueGrey),
    (7, Event::RidleyStatueGrey),
    (8, Event::DraygonStatueGrey),
    (9, Event::KraidStatueGrey),
    (10, Event::TourianOpen),
    (11, Event::GlassTunnelBroken),
    (12, Event::AcidReceded),
    (13, Event::ShaktoolPath),
    (14, Event::TimeBombSet),
    (15, Event::AnimalsFreed),
    (16, Event::MetroidRoom1Empty),
    (17, Event::MetroidRoom2Empty),
    (18, Event::MetroidRoom3Empty),
    (19, Event::MetroidRoom4Empty),
    (20, Event::TourianRoomsUnset),
    (21, Event::LavaQuakeOutrun),
];
const ITEMS: [(u16, Item); 11] = [
    (1, Item::VariaSuit),
    (2, Item::SpringBall),
    (4, Item::MorphBall),
    (8, Item::ScrewAttack),
    (0x20, Item::GravitySuit),
    (0x100, Item::HiJumpBoots),
    (0x200, Item::SpaceJump),
    (0x1000, Item::Bombs),
    (0x2000, Item::SpeedBooster),
    (0x4000, Item::GrappleBeam),
    (0x8000, Item::XRayScope),
];
const BEAMS: [(u16, Beam); 5] = [
    (1, Beam::Wave),
    (2, Beam::Ice),
    (4, Beam::Spazer),
    (8, Beam::Plasma),
    (0x1000, Beam::Charge),
];
const AREAS: [Area; 7] = [
    Area::Crateria,
    Area::Brinstar,
    Area::Norfair,
    Area::WreckedShip,
    Area::Maridia,
    Area::Tourian,
    Area::Ceres,
];
const SUPPLIES: [(Supply, usize, usize); 5] = [
    (Supply::Energy, 0x09C4, 0x09C2),
    (Supply::Reserve, 0x09D4, 0x09D6),
    (Supply::Missiles, 0x09C8, 0x09C6),
    (Supply::SuperMissiles, 0x09CC, 0x09CA),
    (Supply::PowerBombs, 0x09D0, 0x09CE),
];

fn byte(wram: &[u8], at: usize) -> Result<u8, Outcome> {
    wram.get(at)
        .copied()
        .ok_or_else(|| data_failure(None, "progress RAM observation"))
}
fn word(wram: &[u8], at: usize) -> Result<u16, Outcome> {
    Ok(u16::from_le_bytes([
        byte(wram, at)?,
        byte(wram, at.saturating_add(1))?,
    ]))
}
fn bit(wram: &[u8], base: usize, index: u16) -> Result<bool, Outcome> {
    Ok((byte(wram, base.saturating_add(usize::from(index / 8)))? & (1 << (index % 8))) != 0)
}
fn flags<E: enum_map::Enum + Copy>(value: u16, table: &[(u16, E)]) -> Result<Set<E>, Outcome> {
    let known = table.iter().fold(0, |bits, (mask, _)| bits | mask);
    if (value & !known) != 0 {
        return Err(data_failure(None, "progress unnamed flag observation"));
    }
    let mut result = Set::default();
    for &(mask, member) in table {
        result[member] = (value & mask) != 0;
    }
    Ok(result)
}

/// Decode the record's retained Progress.
pub(super) fn decode(wram: &[u8]) -> Result<Progress, Outcome> {
    let mut progress = Progress::default();
    progress.inventory.items = flags(word(wram, 0x09A4)?, &ITEMS)?;
    progress.inventory.items_switched_on = flags(word(wram, 0x09A2)?, &ITEMS)?;
    progress.inventory.beams = flags(word(wram, 0x09A8)?, &BEAMS)?;
    progress.inventory.beams_switched_on = flags(word(wram, 0x09A6)?, &BEAMS)?;
    progress.inventory.reserve_mode = match word(wram, 0x09C0)? {
        0 => ReserveMode::NoTank,
        1 => ReserveMode::Automatic,
        2 => ReserveMode::Manual,
        _ => return Err(data_failure(None, "progress reserve mode")),
    };
    for (supply, capacity, held) in SUPPLIES {
        progress.inventory.capacity.0[supply] = Wrapping(word(wram, capacity)?);
        progress.inventory.held.0[supply] = Wrapping(word(wram, held)?);
    }
    for (index, event) in EVENTS {
        progress.events[event] = bit(wram, 0xD820, index)?;
    }
    progress.zebetites = Wrapping(u16::from((byte(wram, 0xD820)? >> 3) & 7));
    for (index, area) in AREAS.into_iter().enumerate() {
        progress.defeated[area] = flags(
            u16::from(byte(wram, 0xD828_usize.saturating_add(index))?),
            &[
                (1, BossKind::MainBoss),
                (2, BossKind::MiniBoss),
                (4, BossKind::Torizo),
            ],
        )?;
        progress.maps[area] = match byte(wram, 0xD908_usize.saturating_add(index))? {
            0 => MapStatus::Unmapped,
            0xFF => MapStatus::Downloaded,
            _ => MapStatus::OnArrival,
        };
        for station in 0..8 {
            if bit(
                wram,
                0xD8F8_usize.saturating_add(index.saturating_mul(2)),
                station,
            )? {
                progress.stations[area].insert(StationId(station));
            }
        }
    }
    for index in 0..512 {
        if bit(wram, 0xD870, index)? {
            progress.emptied.0.insert(LocationId(index));
        }
        if bit(wram, 0xD8B0, index)? {
            progress.opened.0.insert(DoorId(index));
        }
    }
    progress.items_placed = Wrapping(word(wram, 0xD91A)?);
    Ok(progress)
}

/// This source supplies the fixed named setter, without dynamic arguments or inferred changes.
pub(super) fn validate_watches(watched: &[u32]) -> Result<(), Outcome> {
    if watched.contains(&PLANET_AWAKE_SETTER) {
        Ok(())
    } else {
        Err(data_failure(None, "progress fixed-event watch absent"))
    }
}

/// Preserve observed visit order and multiplicity. Unselected producer effects are not modeled.
pub(super) fn input(record: &FrameRecord) -> Vec<Report> {
    record
        .hits
        .iter()
        .filter(|hit| hit.pc == PLANET_AWAKE_SETTER)
        .map(|_| Report::Happened(Event::PlanetAwake))
        .collect()
}

pub(super) fn expected(wram: &[u8]) -> Result<Vec<Observation>, Outcome> {
    let mut observations = Vec::new();
    for (index, event) in EVENTS {
        observations.push(Observation::quantity(
            format!("progress.events.{event:?}"),
            i64::from(bit(wram, 0xD820, index)?),
        ));
    }
    observations.push(Observation::quantity(
        "progress.zebetites",
        i64::from((byte(wram, 0xD820)? >> 3) & 7),
    ));
    Ok(observations)
}

pub(super) fn actual(progress: &Progress) -> Vec<Observation> {
    let mut observations: Vec<_> = EVENTS
        .iter()
        .map(|&(_, event)| {
            Observation::quantity(
                format!("progress.events.{event:?}"),
                i64::from(progress.events[event]),
            )
        })
        .collect();
    observations.push(Observation::quantity(
        "progress.zebetites",
        i64::from(progress.zebetites.0),
    ));
    observations
}
