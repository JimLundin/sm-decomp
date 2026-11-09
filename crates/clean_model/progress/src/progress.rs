//! The whole of what the game keeps of a run besides where she is (spec §1), and what has
//! happened in the world (§2).

use std::collections::BTreeSet;

use clean_types::{Area, BossKind, DoorId, Event, LocationId, Set, StationId, Table};

use crate::inventory::{Count, Inventory};
use crate::places::{MapStatus, Remembered};

/// Events that have happened. Once an event happens it has happened for the rest of the run; one
/// that happens again changes nothing. Spec: EventsHappen.
pub type Events = Set<Event, { Event::COUNT }>;

/// Kinds of boss: those defeated in an area, or those a room's version asks about. A boss is
/// remembered by its kind in its area; one area's kind says nothing of another's.
/// Spec: BossesDefeated.
pub type BossKinds = Set<BossKind, { BossKind::COUNT }>;

/// One value for each area: the bosses defeated there, its map, its stations saved at.
pub type PerArea<V> = Table<Area, V, { Area::ALL.len() }>;

/// What the game remembers of her run: what she has, what has happened, what places remember.
///
/// It carries from room to room and into the saved file. Other domains read its fields as they
/// are now; `Progress::apply` is what changes them.
///
/// Its default is a new file: the default of each part, so what `Inventory`'s default gives her
/// and nothing else. A new file takes the events, the bosses defeated and the items placed from
/// the slot, which the file select has already emptied, so they too start at none and 0.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Progress {
    /// What she has.
    pub inventory: Inventory,
    /// The events that have happened. Spec: EventsHappen.
    pub events: Events,
    /// How many of Tourian's zebetites she has destroyed, 0 to 4: one more for each destroyed.
    /// The zebetites read it back, so the ones she broke stay broken. Spec: ZebetitesCounted.
    pub zebetites: Count,
    /// The kinds of boss defeated in each area. Spec: BossesDefeated.
    pub defeated: PerArea<BossKinds>,
    /// The item locations she has taken an item from. Spec: EmptiedLocations.
    pub emptied: Remembered<LocationId>,
    /// The doors that stay open. Spec: DoorsStayOpen.
    pub opened: Remembered<DoorId>,
    /// How she has each area's map. Spec: MapsByArea, Spec: MapOnArrival.
    pub maps: PerArea<MapStatus>,
    /// The save stations she has saved at, by area. A station's id counts within its area, so
    /// the same id in two areas is two stations. Spec: StationsSavedAt.
    pub stations: PerArea<BTreeSet<StationId>>,
    /// The items placed: how many times a room's loading has placed an item, over the whole run.
    /// It counts placements, not items taken: every item counts each time its room loads, taken
    /// or not. The quirk of the items placed: nothing in play reads it, but the saved file keeps
    /// it. Spec: ItemsPlacedCount.
    pub items_placed: Count,
}
