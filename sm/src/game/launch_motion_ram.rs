//! Transitional serialization for the launch-motion model. No decoded state or
//! RAM borrow survives a call back into gameplay. This adapter is removable when
//! Samus owns LaunchMotion directly; it is not the final engine state storage.
#![forbid(unsafe_code)]
use crate::game::launch_motion::{LaunchMotion, MovementDirection};
use crate::game_state::Game;

fn read(game: &Game, address: usize) -> u16 {
    u16::from_le_bytes([game.g_ram[address], game.g_ram[address + 1]])
}

fn write(game: &mut Game, address: usize, value: u16) {
    game.g_ram[address..address + 2].copy_from_slice(&value.to_le_bytes());
}

fn history(game: &Game) -> LaunchMotion {
    LaunchMotion {
        displacements: core::array::from_fn(|i| {
            ((u32::from(read(game, 0xdaa + 4 * i)) << 16) | u32::from(read(game, 0xdac + 4 * i)))
                as i32
        }),
    }
}

pub(crate) fn record_movement(game: &mut Game, direction: MovementDirection, displacement: i32) {
    let mut motion = history(game);
    motion.record_movement(direction, displacement);
    let index = direction as usize;
    let value = motion.displacements[index] as u32;
    write(game, 0xdac + 4 * index, value as u16);
    write(game, 0xdaa + 4 * index, (value >> 16) as u16);
}
