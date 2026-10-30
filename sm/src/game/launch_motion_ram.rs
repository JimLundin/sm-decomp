//! Transitional serialization for the launch-motion model. No decoded state or
//! RAM borrow survives a call back into gameplay. This adapter is removable when
//! Samus owns LaunchMotion directly; it is not the final engine state storage.
#![forbid(unsafe_code)]
use crate::game_state::Game;
use sm_game::launch_motion::{LaunchMotion, MovementDirection};

fn read(game: &Game, address: usize) -> u16 {
    u16::from_le_bytes([
        game.ram().read_byte(address),
        game.ram().read_byte(address + 1),
    ])
}

fn write(game: &mut Game, address: usize, value: u16) {
    game.ram().copy_in(address, &value.to_le_bytes());
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
    record_observed(&mut motion, direction, displacement);
    let index = direction as usize;
    let value = motion.displacements[index] as u32;
    write(game, 0xdac + 4 * index, value as u16);
    write(game, 0xdaa + 4 * index, (value >> 16) as u16);
}

/// Record one launch-motion step, after the verifier observes it.
pub(crate) fn record_observed(
    motion: &mut LaunchMotion,
    direction: MovementDirection,
    amount: i32,
) {
    #[cfg(any(test, sm_verify_dll))]
    crate::game::player_collision_owned_checks::record_launch(direction, amount);
    motion.record_movement(direction, amount);
}
