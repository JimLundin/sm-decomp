//! Canonical form of the run progress that crosses the two cinematics.

use canonical::Canonical;
use enum_map::Enum;
use run_progress::{MapStatus, Progress, ReserveMode, Supply};

#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
enum Map {
    Unmapped,
    Downloaded,
    OnArrival,
}

#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
enum Reserve {
    NoTank,
    Automatic,
    Manual,
}

/// The complete inventory, events, remembered places and map status of a run.
#[derive(Canonical, Clone, Debug, PartialEq, Eq)]
pub struct Remembered {
    items: [bool; 11],
    items_on: [bool; 11],
    beams: [bool; 5],
    beams_on: [bool; 5],
    capacity: [u16; 5],
    held: [u16; 5],
    reserve: Reserve,
    events: [bool; 18],
    zebetites: u16,
    defeated: [[bool; 3]; 7],
    emptied: Vec<u16>,
    opened: Vec<u16>,
    maps: [Map; 7],
    stations: [Vec<u16>; 7],
    items_placed: u16,
}

impl From<&Progress> for Remembered {
    fn from(run: &Progress) -> Self {
        let inventory = &run.inventory;
        Self {
            items: inventory.items.into_array(),
            items_on: inventory.items_switched_on.into_array(),
            beams: inventory.beams.into_array(),
            beams_on: inventory.beams_switched_on.into_array(),
            capacity: inventory.capacity.0.into_array().map(|value| value.0),
            held: inventory.held.0.into_array().map(|value| value.0),
            reserve: match inventory.reserve_mode {
                ReserveMode::NoTank => Reserve::NoTank,
                ReserveMode::Automatic => Reserve::Automatic,
                ReserveMode::Manual => Reserve::Manual,
            },
            events: run.events.into_array(),
            zebetites: run.zebetites.0,
            defeated: run.defeated.into_array().map(enum_map::EnumMap::into_array),
            emptied: run.emptied.0.iter().map(|place| place.0).collect(),
            opened: run.opened.0.iter().map(|door| door.0).collect(),
            maps: run.maps.into_array().map(|status| match status {
                MapStatus::Unmapped => Map::Unmapped,
                MapStatus::Downloaded => Map::Downloaded,
                MapStatus::OnArrival => Map::OnArrival,
            }),
            stations: run
                .stations
                .as_array()
                .each_ref()
                .map(|stations| stations.iter().map(|station| station.0).collect()),
            items_placed: run.items_placed.0,
        }
    }
}

impl Remembered {
    pub(super) fn restore_energy(&mut self, energy: u16) {
        if let Some(held) = self.held.get_mut(Supply::Energy.into_usize()) {
            *held = energy;
        }
    }
}
