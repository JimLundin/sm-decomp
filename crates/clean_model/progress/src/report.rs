//! What other domains report (spec §1.3), and how each report changes the progress.

use core::num::Wrapping;

use clean_types::{Area, BossKind, DoorId, Event, LocationId, StationId};

use crate::inventory::{Count, Pickup};
use crate::places::MapStatus;
use crate::progress::Progress;
use crate::rules::Rules;

/// Something another domain tells progress has happened.
///
/// When a report comes is the reporting domain's to decide; progress only records it. A report
/// that concerns an area names it: the area of the room she is in, which is still the area she
/// leaves when she leaves it.
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
    /// She enters the room that gives the map of its area, the area named. Spec: MapOnArrival.
    MapOnArrival(Area),
    /// She leaves the area named, by a door into another area. Spec: MapOnArrival.
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
            Report::Happened(event) => self.events[event] = true,
            Report::ZebetiteDestroyed => self.zebetites += Wrapping(1),
            Report::Defeated { kind, area } => self.defeated[area][kind] = true,
            Report::Taken { pickup, location } => {
                self.inventory.take(pickup, rules.beam_switches_off);
                self.emptied.remember(location);
            }
            Report::DoorOpened(door) => self.opened.remember(door),
            Report::MapStation(area) => self.maps[area] = MapStatus::Downloaded,
            // Spec: MapOnArrival. The room she enters leaves its area's map `OnArrival`, unless
            // she has it downloaded; leaving the area turns `OnArrival` into `Downloaded`.
            Report::MapOnArrival(entered) if self.maps[entered] != MapStatus::Downloaded => {
                self.maps[entered] = MapStatus::OnArrival;
            }
            Report::LeftArea(left) if self.maps[left] == MapStatus::OnArrival => {
                self.maps[left] = MapStatus::Downloaded;
            }
            Report::MapOnArrival(_) | Report::LeftArea(_) => {}
            Report::Saved { area, station } => {
                self.stations[area].insert(station);
            }
            Report::ItemsPlaced(count) => self.items_placed += count,
        }
    }
}
