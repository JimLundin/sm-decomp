//! The full percentage (the decode notes' "No record: the full percentage"): every item the game
//! places, taken once from a new file, counts 100. It needs only the ROM.
//!
//! No importer lists the rooms, so the walk follows the doors from the Landing Site and from
//! Ceres's first room, reading each room's door list and its states' PLM populations through the
//! importer. A location holds one kind across its room's states, so the items are kept once per
//! location.

use std::collections::{BTreeMap, BTreeSet};

use clean_model_progress::{Count, Pickup, Progress, Report, Rules};
use clean_types::LocationId;
use data::plm::PlmKind;
use data::rom::{Lorom, RomAddress};
use data::room::{RoomKey, populations};

use super::{ITEM_PLMS, ProgressDomain};

/// The rooms the walk starts from: the Landing Site and Ceres's first room (bank $8F).
const STARTS: [u16; 2] = [0x91F8, 0xDF45];
/// Where the room headers lie in bank $8F.
const ROOM_HEADERS: core::ops::Range<u16> = 0x91F8..0xE82C;
/// Where the door headers lie in bank $83; a room's door list ends at the first word outside.
const DOOR_HEADERS: core::ops::Range<u16> = 0x88FC..0xAC00;
/// The longest door list the walk reads.
const DOOR_LIST_REACH: usize = 64;
/// The door headers' bank, and the rooms'.
const DOOR_BANK: u8 = 0x83;
const ROOM_BANK: u8 = 0x8F;

/// Every item the game places, by its location.
pub struct Population {
    /// The rooms the walk reached, in the order it reached them.
    pub rooms: Vec<RoomKey>,
    /// Each location's item.
    pub items: BTreeMap<LocationId, Pickup>,
}

/// A little-endian word of the ROM at `bank:offset`, if the image holds it.
fn rom_word(rom: Lorom<'_>, bank: u8, offset: u16) -> Option<u16> {
    let bytes = rom.read(RomAddress::new(bank, offset), 2).ok()?;
    Some(u16::from_le_bytes([*bytes.first()?, *bytes.get(1)?]))
}

impl ProgressDomain {
    /// The rooms the doors reach from [`STARTS`] and the items their states place.
    ///
    /// # Errors
    ///
    /// A room the importer can't read, an item PLM whose pickup the spec doesn't name, or a
    /// location two kinds share.
    pub fn population(&self) -> Result<Population, String> {
        let image = Lorom::new(&self.image);
        let mut seen = BTreeSet::new();
        let mut rooms = Vec::new();
        let mut items = BTreeMap::new();
        let mut todo = Vec::from(STARTS);
        while let Some(pointer) = todo.pop() {
            if seen.contains(&pointer) || !ROOM_HEADERS.contains(&pointer) {
                continue;
            }
            let room = populations(image, RoomKey::from_room_pointer(pointer))
                .map_err(|error| format!("room {pointer:#06X}: {error:?}"))?;
            seen.insert(pointer);
            rooms.push(RoomKey::from_room_pointer(pointer));
            for plms in &room.plms {
                for entry in &plms.entries {
                    let plm = entry.kind.id();
                    let (PlmKind::Item(_, argument) | PlmKind::Unmodelled { argument, .. }) =
                        entry.kind
                    else {
                        continue;
                    };
                    if !ITEM_PLMS.contains(&plm) {
                        continue;
                    }
                    let (_, pickup) = self.pickup(plm).map_err(|skip| skip.to_string())?;
                    let location = LocationId(argument);
                    if items
                        .insert(location, pickup)
                        .is_some_and(|was| was != pickup)
                    {
                        return Err(format!("location {argument} holds two kinds"));
                    }
                }
            }
            let doors = (0_u16..)
                .take(DOOR_LIST_REACH)
                .map_while(|place| {
                    let at = room.doors.offset.wrapping_add(place.wrapping_mul(2));
                    rom_word(image, ROOM_BANK, at).filter(|door| DOOR_HEADERS.contains(door))
                })
                .filter_map(|door| rom_word(image, DOOR_BANK, door))
                .filter(|&into| into != 0);
            todo.extend(doors);
        }
        Ok(Population { rooms, items })
    }
}

/// The percentage the ending shows after a new file takes every item of `population` once.
#[must_use]
pub fn of_everything(population: &Population) -> Count {
    let rules = Rules::reference();
    let mut progress = Progress::default();
    for (&location, &pickup) in &population.items {
        progress.apply(
            &rules,
            Report::Taken {
                pickup,
                location: Some(location),
            },
        );
    }
    progress.inventory.item_percentage(&rules)
}
