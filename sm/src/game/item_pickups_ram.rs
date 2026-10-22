//! Transitional inventory serialization. No inventory snapshot survives effects.
#![forbid(unsafe_code)]
use crate::game::item_pickups::{Inventory, Loadout, PickupEffects, PickupKind, Supply};
use crate::game_state::Game;

fn read(game: &Game, address: usize) -> u16 {
    u16::from_le_bytes([game.g_ram[address], game.g_ram[address + 1]])
}
fn write(game: &mut Game, address: usize, value: u16) {
    game.g_ram[address..address + 2].copy_from_slice(&value.to_le_bytes());
}

pub(crate) fn collect(game: &mut Game, kind: PickupKind, amount: u16) -> PickupEffects {
    let supply = |address| Supply {
        current: read(game, address),
        maximum: read(game, address + 2),
    };
    let mut inventory = Inventory {
        equipment: Loadout {
            active: read(game, 0x9a2),
            owned: read(game, 0x9a4),
        },
        beams: Loadout {
            active: read(game, 0x9a6),
            owned: read(game, 0x9a8),
        },
        health: supply(0x9c2),
        reserve_capacity: read(game, 0x9d4),
        reserve_mode: read(game, 0x9c0),
        ammunition: [supply(0x9c6), supply(0x9ca), supply(0x9ce)],
    };
    let effects = inventory.collect(kind, amount);
    use PickupKind::*;
    match kind {
        Beam => {
            write(game, 0x9a8, inventory.beams.owned);
            write(game, 0x9a6, inventory.beams.active);
        }
        Equipment | Grapple | Xray => {
            write(game, 0x9a2, inventory.equipment.active);
            write(game, 0x9a4, inventory.equipment.owned);
        }
        Energy => {
            write(game, 0x9c4, inventory.health.maximum);
            write(game, 0x9c2, inventory.health.current);
        }
        Reserve => {
            write(game, 0x9d4, inventory.reserve_capacity);
            write(game, 0x9c0, inventory.reserve_mode);
        }
        Missiles | SuperMissiles | PowerBombs => {
            let index = match kind {
                Missiles => 0,
                SuperMissiles => 1,
                _ => 2,
            };
            let address = 0x9c6 + 4 * index;
            write(game, address + 2, inventory.ammunition[index].maximum);
            write(game, address, inventory.ammunition[index].current);
        }
    }
    effects
}
