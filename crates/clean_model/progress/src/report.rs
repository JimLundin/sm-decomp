//! What other domains report, and how each report changes the progress.

use clean_types::{Area, BossKind, DoorId, Event, LocationId, StationId};

use crate::count::Count;
use crate::inventory::Pickup;
use crate::progress::Progress;
use crate::rules::Rules;

/// Something another domain tells progress has happened. When a report comes is the reporting
/// domain's to decide; progress only records it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Report {
    /// An event happens. Spec: EventsHappen.
    Happened(Event),
    /// A zebetite is destroyed. Spec: ZebetitesCounted.
    ZebetiteDestroyed,
    /// A boss of this kind is defeated, in the frame's area. Spec: BossesDefeated.
    Defeated(BossKind),
    /// She takes an item, a beam or a tank, from a place with an id or without one.
    /// Spec: TakingItems, Spec: TakingBeams, Spec: Tanks, Spec: EmptiedLocations.
    Taken {
        /// What she takes.
        pickup: Pickup,
        /// The id of the place it lay in, if its room's population gave it one.
        location: Option<LocationId>,
    },
    /// A door that stays open is opened, with its id if its room's population gave it one.
    /// Spec: DoorsStayOpen.
    DoorOpened(Option<DoorId>),
    /// The map station of the frame's area downloads. Spec: MapsByArea.
    MapStation,
    /// She enters the room that gives the map of the frame's area. Spec: MapOnArrival.
    MapOnArrival,
    /// She leaves the frame's area by a door into another area. Spec: MapOnArrival.
    LeftArea,
    /// She saves at this station of the frame's area, or the gunship first lands there.
    /// Spec: StationsSavedAt.
    Saved(StationId),
    /// A room's loading places this many items. Spec: ItemsPlacedCount.
    ItemsPlaced(Count),
}

impl Progress {
    /// Records one report, made in a frame whose area is `area`: the area of the room she is
    /// in, which is still the old area when she leaves it.
    pub fn apply(&mut self, rules: &Rules, area: Area, report: Report) {
        let world = &mut self.world;
        match report {
            Report::Happened(event) => world.happen(event),
            Report::ZebetiteDestroyed => world.destroy_zebetite(),
            Report::Defeated(kind) => world.defeat(area, kind),
            Report::Taken { pickup, location } => {
                self.inventory.take(rules, pickup);
                world.empty(location);
            }
            Report::DoorOpened(door) => world.open(door),
            Report::MapStation => world.download_map(area),
            Report::MapOnArrival => world.map_on_arrival(area),
            Report::LeftArea => world.leave_area(area),
            Report::Saved(station) => world.save(area, station),
            Report::ItemsPlaced(count) => world.place_items(count),
        }
    }

    /// Records a frame's reports in the order they come.
    pub fn apply_all(&mut self, rules: &Rules, area: Area, reports: &[Report]) {
        for &report in reports {
            self.apply(rules, area, report);
        }
    }
}
