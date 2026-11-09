//! What other domains report, and how each report changes the progress.

use core::num::Wrapping;

use clean_types::{Area, BossKind, DoorId, Event, LocationId, StationId};

use crate::count::Count;
use crate::inventory::Pickup;
use crate::places::MapStatus;
use crate::progress::Progress;
use crate::rules::Rules;

/// Something another domain tells progress has happened.
///
/// When a report comes is the reporting domain's to decide; progress only records it. A report
/// that concerns an area names it: the area of the room she is in, which is still the old area
/// when she leaves it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Report {
    /// An event happens. Spec: EventsHappen.
    Happened(Event),
    /// A zebetite is destroyed. Spec: ZebetitesCounted.
    ZebetiteDestroyed,
    /// A boss of this kind is defeated in this area. Spec: BossesDefeated.
    Defeated {
        /// The kind the boss reports.
        kind: BossKind,
        /// The area it is defeated in.
        area: Area,
    },
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
    /// The map station of this area downloads. Spec: MapsByArea.
    MapStation(Area),
    /// She enters the room that gives the map of this area. Spec: MapOnArrival.
    MapOnArrival(Area),
    /// She leaves this area by a door into another area. Spec: MapOnArrival.
    LeftArea(Area),
    /// She saves at a station of this area, or the gunship first lands at its area's first.
    /// Spec: StationsSavedAt.
    Saved {
        /// The station's area.
        area: Area,
        /// The station's id within its area.
        station: StationId,
    },
    /// A room's loading places this many items. Spec: ItemsPlacedCount.
    ItemsPlaced(Count),
}

impl Progress {
    /// Records one report. A frame's reports apply in the order they come; none reads another's
    /// result, but she leaves the old area before the new room's loading reports.
    pub fn apply(&mut self, rules: &Rules, report: Report) {
        match report {
            Report::Happened(event) => self.events = self.events.with(event),
            Report::ZebetiteDestroyed => self.zebetites += Wrapping(1),
            Report::Defeated { kind, area } => self.defeated[area] = self.defeated[area].with(kind),
            Report::Taken { pickup, location } => {
                self.inventory.take(rules, pickup);
                self.emptied.remember(location);
            }
            Report::DoorOpened(door) => self.opened.remember(door),
            Report::MapStation(area) => self.maps[area] = MapStatus::Downloaded,
            // Spec: MapOnArrival. The room that gives the map leaves it `OnArrival`, unless she
            // has it downloaded; leaving the area turns `OnArrival` into `Downloaded`.
            Report::MapOnArrival(area) if self.maps[area] != MapStatus::Downloaded => {
                self.maps[area] = MapStatus::OnArrival;
            }
            Report::LeftArea(area) if self.maps[area] == MapStatus::OnArrival => {
                self.maps[area] = MapStatus::Downloaded;
            }
            Report::MapOnArrival(_) | Report::LeftArea(_) => {}
            Report::Saved { area, station } => {
                self.stations[area].insert(station);
            }
            Report::ItemsPlaced(count) => self.placements += count,
        }
    }
}
