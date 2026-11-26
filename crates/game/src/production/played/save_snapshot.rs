//! Capture the running Ceres file from its current gameplay owners.

use data::map::MapCells;
use interface_hud::{Ammo, HudState, SelectableItem};
use run_progress::Supply as Stock;
use types::save_files::{Ammunition, ExploredCells, Gear, RunSnapshot, SelectedWeapon, Supply};
use types::{Area, Pair, StationId};

/// The current run and the active map that the staged save receives.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Capture {
    /// Live save payload before the writer sets its restart route.
    pub run: RunSnapshot,
    /// Exploration in the active area.
    pub live_map: ExploredCells,
    /// Area and station reached by gameplay.
    pub location: (Area, StationId),
}

/// Combine retained file values with the live owners' save fields.
///
/// The saved map history and active map stay separate until the cartridge writer captures them.
#[must_use]
pub fn capture(
    selected: &RunSnapshot,
    room: &super::State,
    location: (Area, StationId),
) -> Option<Capture> {
    let mut run = selected.clone();
    let progress = &room.progress;
    let inventory = &progress.inventory;
    let stock = |kind| Supply {
        current: inventory.held.0[kind].0,
        maximum: inventory.capacity.0[kind].0,
    };
    run.profile.owned = Gear {
        items: inventory.items,
        beams: inventory.beams,
    };
    run.profile.in_use = Gear {
        items: inventory.items_switched_on,
        beams: inventory.beams_switched_on,
    };
    run.profile.energy = Supply {
        current: room.health.0.energy.whole.get(),
        ..stock(Stock::Energy)
    };
    run.profile.reserves = stock(Stock::Reserve);
    for (ammo, kind) in [
        (Ammunition::Missiles, Stock::Missiles),
        (Ammunition::SuperMissiles, Stock::SuperMissiles),
        (Ammunition::PowerBombs, Stock::PowerBombs),
    ] {
        run.profile.ammunition[ammo] = stock(kind);
    }
    run.profile.weapon = match room.hud.0.selection.item() {
        None => SelectedWeapon::Nothing,
        Some(SelectableItem::Ammo(Ammo::Missiles)) => SelectedWeapon::Missiles,
        Some(SelectableItem::Ammo(Ammo::SuperMissiles)) => SelectedWeapon::SuperMissiles,
        Some(SelectableItem::Ammo(Ammo::PowerBombs)) => SelectedWeapon::PowerBombs,
        Some(SelectableItem::GrappleBeam) => SelectedWeapon::GrappleBeam,
        Some(SelectableItem::XRayScope) => SelectedWeapon::XRayScope,
    };
    run.profile.play_time = room.play_time;
    run.progress.events = progress.events;
    run.progress.bosses = progress.defeated;
    run.progress.item_count = progress.items_placed.0;
    run.progress.items.0 = progress.emptied.0.iter().map(|place| place.0).collect();
    run.progress.doors.0 = progress.opened.0.iter().map(|door| door.0).collect();
    run.progress.maps_collected = enum_map::EnumMap::from_fn(|area| progress.maps[area].is_known());

    let live_map = explored(&room.hud.0).to_bitmap().to_vec();
    Some(Capture {
        run,
        live_map: ExploredCells::new(live_map)?,
        location,
    })
}

pub(super) fn explored(hud: &HudState) -> MapCells {
    MapCells(core::array::from_fn(|y| {
        let y = i32::try_from(y).unwrap_or_default();
        (0..64).rev().fold(0, |row, x| {
            (row << 1) | u64::from(hud.explored.contains(Pair { x, y }))
        })
    }))
}
