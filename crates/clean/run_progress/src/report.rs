//! What other domains report (spec §1.3), and how each report changes the progress.

use core::num::Wrapping;

use types::{Area, BossKind, DoorId, Event, LocationId, StationId};

use crate::inventory::{Count, Pickup};
use crate::places::MapStatus;
use crate::progress::Progress;
use crate::rules::Rules;

/// Something another domain tells progress has happened.
///
/// When a report comes is the reporting domain's to decide; progress only records it. A report
/// that concerns an area carries it. Each carries the frame's area, the area of the room she is
/// in, except `LeftArea`: in the frame a door takes her into another area, the frame's area is
/// the new room's, and `LeftArea` carries the area she leaves.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Report {
    /// An event happens. Spec: EventsHappen.
    Happened(Event),
    /// A zebetite is destroyed. Spec: ZebetitesCounted.
    ZebetiteDestroyed,
    /// A boss of this kind is defeated. Spec: BossesDefeated.
    Defeated {
        /// The kind the boss reports.
        boss: BossKind,
        /// The frame's area, which the boss is defeated in.
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
    /// The map station downloads; it carries the frame's area, its station's. Spec: MapsByArea.
    MapStation(Area),
    /// She enters the room that gives the map of its area; it carries the frame's area, the
    /// room's. Spec: MapOnArrival.
    MapOnArrival(Area),
    /// A door takes her into another area; it carries the area she leaves, not the frame's.
    /// Spec: MapOnArrival.
    LeftArea(Area),
    /// She saves at a save station; or the gunship lands for the first time, which counts as a
    /// save at the first station of the area it lands in, and comes with that station's id.
    /// Spec: StationsSavedAt.
    Saved {
        /// The frame's area, the station's.
        area: Area,
        /// The station's id within its area.
        station: StationId,
    },
    /// A room's loading places this many items. Spec: ItemsPlacedCount.
    ItemsPlaced(Count),
}

impl Progress {
    /// Records a frame's reports, in the order they come: the progress after them is what the
    /// same reports give one per frame, in that order.
    ///
    /// The order shows for two pairs. Two beams taken, one of which switches the other off: the
    /// later stays on (Spec: TakingBeams). A map given on arrival and her leaving its area:
    /// given then left is `Downloaded`, left then given `OnArrival` (Spec: MapOnArrival). Every
    /// other pair changes different quantities, adds to one count, or sets what is already set.
    /// The game never puts either pair in one frame; only an adjusted game's reports show the
    /// order.
    pub fn apply(&mut self, rules: &Rules, reports: &[Report]) {
        for &report in reports {
            self.record(rules, report);
        }
    }

    /// Records one report.
    fn record(&mut self, rules: &Rules, report: Report) {
        match report {
            Report::Happened(event) => self.events[event] = true,
            Report::ZebetiteDestroyed => self.zebetites += Wrapping(1),
            Report::Defeated { boss, area } => self.defeated[area][boss] = true,
            Report::Taken { pickup, location } => {
                self.inventory.take(pickup, &rules.switches_off);
                self.emptied.remember(location);
            }
            Report::DoorOpened(door) => self.opened.remember(door),
            Report::MapStation(area) => self.maps[area] = MapStatus::Downloaded,
            // Spec: MapOnArrival. The room she enters leaves its area's map `OnArrival`, unless
            // she has it downloaded; a door taking her into another area turns the map of the
            // area she leaves from `OnArrival` into `Downloaded`.
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
