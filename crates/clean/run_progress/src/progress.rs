//! The whole of what the game keeps of a run besides where she is (spec §1), and what has
//! happened in the world (§2).

use std::collections::BTreeSet;

use enum_map::EnumMap;
use types::{Area, BossKind, DoorId, Event, LocationId, Set, StationId};

use crate::inventory::{Count, Inventory};
use crate::places::{MapStatus, Remembered};

/// What the game remembers of her run: what she has, what has happened, what places remember.
///
/// It carries from room to room and into the saved file. Other domains read its fields as they
/// are now; `Progress::apply` is what changes them.
///
/// Its default is what a new file holds: what `Inventory`'s default gives her (99 energy of a
/// capacity of 99, nothing else), and none of everything else: no events, no bosses defeated, no
/// locations emptied, no doors open, no maps, no stations, no zebetites and no items placed. A
/// new file takes its events, bosses defeated and items placed from the slot rather than
/// clearing them, but the file select has already emptied the slot, so they too are none and 0.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Progress {
    /// What she has.
    pub inventory: Inventory,
    /// The events that have happened. Once an event happens it has happened for the rest of the
    /// run; one that happens again changes nothing. Spec: EventsHappen.
    pub events: Set<Event>,
    /// How many of Tourian's zebetites she has destroyed: one more for each destroyed. The
    /// zebetites read it back, so the ones she broke stay broken. The spec says it only rises to
    /// 4, as there are four zebetites; that is a fact of the game's world, not a limit kept here:
    /// a fifth report would make it 5. Spec: ZebetitesCounted.
    pub zebetites: Count,
    /// The kinds of boss defeated in each area. A boss is remembered by its kind in its area; one
    /// area's kind says nothing of another's. Spec: BossesDefeated.
    pub defeated: EnumMap<Area, Set<BossKind>>,
    /// The item locations she has taken an item from. Spec: EmptiedLocations.
    pub emptied: Remembered<LocationId>,
    /// The doors that stay open. Spec: DoorsStayOpen.
    pub opened: Remembered<DoorId>,
    /// How she has each area's map. Spec: MapsByArea, Spec: MapOnArrival.
    pub maps: EnumMap<Area, MapStatus>,
    /// The save stations she has saved at, by area, for the maps to show. The gunship's first
    /// landing counts as a save at its area's first station (`Report::Saved`). A station's id
    /// counts within its area, so the same id in two areas is two stations.
    /// Spec: StationsSavedAt.
    pub stations: EnumMap<Area, BTreeSet<StationId>>,
    /// The items placed: how many times a room's loading has placed an item, over the whole run.
    /// It counts placements, not items taken: every item counts each time its room loads, taken
    /// or not. The quirk of the items placed: nothing in play reads it, but the saved file keeps
    /// it. Spec: ItemsPlacedCount.
    pub items_placed: Count,
}
