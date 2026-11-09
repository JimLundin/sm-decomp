//! The whole of what the game keeps of a run besides where she is.

use std::collections::BTreeSet;

use clean_types::{DoorId, LocationId, StationId};

use crate::count::Count;
use crate::inventory::Inventory;
use crate::per_area::PerArea;
use crate::places::{MapStatus, Remembered};
use crate::world::{BossKinds, Events};

/// What the game remembers of her run: what she has, what has happened, what places remember.
///
/// It carries from room to room and into the saved file. Other domains read its fields as they
/// are now; `Progress::apply` is what changes them.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Progress {
    /// What she has.
    pub inventory: Inventory,
    /// The events that have happened. Spec: EventsHappen.
    pub events: Events,
    /// How many of Tourian's zebetites she has destroyed, 0 to 4. Spec: ZebetitesCounted.
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
    /// How many times a room's loading has placed an item, over the whole run: every item each
    /// time its room loads, taken or not. The quirk of the items placed: nothing in play reads
    /// it, but the saved file keeps it. Spec: ItemsPlacedCount.
    pub placements: Count,
}

impl Progress {
    /// A new file: what `Inventory::new_file` gives her, and nothing else. The events, the bosses
    /// defeated and the placements come from the slot, which the file select has already
    /// emptied, so they too start at none and 0.
    #[must_use]
    pub fn new_file() -> Self {
        Self {
            inventory: Inventory::new_file(),
            events: Events::default(),
            zebetites: Count::default(),
            defeated: PerArea::default(),
            emptied: Remembered::of([]),
            opened: Remembered::of([]),
            maps: PerArea::default(),
            stations: PerArea::default(),
            placements: Count::default(),
        }
    }
}
