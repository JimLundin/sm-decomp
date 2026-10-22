//! Transitional serialization for the launch-motion model. No decoded state or
//! RAM borrow survives a call back into gameplay. This adapter is removable when
//! Samus owns LaunchMotion directly; it is not the final engine state storage.
#![forbid(unsafe_code)]
use crate::game::launch_motion::{Aim, LaunchMotion, MovementDirection};
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

pub(crate) fn finish_projectile_phase(game: &mut Game) {
    let mut motion = history(game);
    motion.finish_projectile_phase();
    for (index, value) in motion.displacements.into_iter().enumerate() {
        write(game, 0xdaa + 4 * index, (value >> 16) as u16);
        write(game, 0xdac + 4 * index, value as u16);
    }
}

/// False preserves the legacy invalid-direction outcome after clearing subpixels.
/// Keep its diagnostic/unsafe callback in the existing flight caller.
pub(crate) fn initialize(k: u16, speed: u16, game: &mut Game) -> bool {
    let offset = usize::from(k >> 1) * 2;
    write(game, 0xb8c + offset, 0);
    write(game, 0xba0 + offset, 0);
    let aim = match read(game, 0xc04 + offset) & 0xf {
        0 | 9 => Aim::Up,
        1 => Aim::UpRight,
        2 => Aim::Right,
        3 => Aim::DownRight,
        4 | 5 => Aim::Down,
        6 => Aim::DownLeft,
        7 => Aim::Left,
        8 => Aim::UpLeft,
        _ => return false,
    };
    let velocity = history(game).launch(aim, speed, read(game, 0xda8));
    write(game, 0xbf0 + offset, velocity.y as u16);
    // Legacy indices can address beyond projectile slots, including history.
    // Preserve the original vertical-write-before-horizontal-read ordering.
    let velocity = history(game).launch(aim, speed, read(game, 0xda8));
    write(game, 0xbdc + offset, velocity.x as u16);
    true
}
