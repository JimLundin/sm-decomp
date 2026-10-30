//! Rio dives share collision responses and guarded animation changes.
#![forbid(unsafe_code)]
use crate::game::actors::api::{self, Axis, MovementResult, SlopePolicy};
use crate::game::session_runtime::Runtime;
use crate::game_state::Game;
use sm_game::actors::access::{AccessMode::Native, ActorAccess, ActorKey as Key};
use sm_game::projectile_geometry::SINE_COSINE;

fn current(runtime: &Runtime<'_>) -> Key {
    Key::from_legacy(runtime.word(0xe54))
}

fn animate(runtime: &mut Runtime<'_>, instruction: u16) {
    let key = current(runtime);
    runtime.edit_actor(key, Native, |actor| {
        if actor.pre_instruction != instruction {
            actor.pre_instruction = instruction;
            actor.restart_animation(instruction);
        }
    });
}

pub(super) fn aim_velocity(
    runtime: &mut Runtime<'_>,
    key: Key,
    vertical: usize,
    tables: [u32; 2],
    index: isize,
) {
    for (variable, address, entry) in [(vertical, tables[0], index), (vertical + 1, tables[1], 0)] {
        let value = runtime
            .loaded_image()
            .expect("ROM must be loaded")
            .table_word(crate::rom::lorom_offset(address) as usize, entry);
        runtime.edit_actor(key, Native, |actor| actor.variables[variable] = value);
    }
    let player_x = runtime.session().player.x;
    let actor_x = runtime.resolve_actor(key, Native).x.whole;
    if (player_x.wrapping_sub(actor_x) as i16) < 0 {
        runtime.edit_actor(key, Native, |actor| {
            actor.variables[vertical + 1] = actor.variables[vertical + 1].wrapping_neg();
        });
    }
}

pub(super) fn move_axis(runtime: &mut Runtime<'_>, key: Key, axis: Axis, variable: usize) -> bool {
    let amount = (runtime.resolve_actor(key, Native).variables[variable] as i16 as i32) << 8;
    (match axis {
        Axis::X => api::move_horizontal(runtime, key, amount, SlopePolicy::Ignore),
        Axis::Y => api::move_vertical(runtime, key, amount),
    }) == MovementResult::Blocked
}

pub(super) fn bounce_horizontal(runtime: &mut Runtime<'_>, key: Key, variable: usize) -> bool {
    let blocked = move_axis(runtime, key, Axis::X, variable);
    if blocked {
        runtime.edit_actor(key, Native, |actor| {
            actor.variables[variable] = actor.variables[variable].wrapping_neg();
        });
    }
    blocked
}

pub(crate) fn initialize(game: &mut Game) {
    let mut runtime = Runtime::new(game);
    let key = current(&runtime);
    runtime.edit_actor(key, Native, |actor| {
        actor.variables[4] = 0;
        actor.pre_instruction = 0;
        actor.instruction = 0xbb4b;
        actor.variables[1] = 0xbbed;
    });
}

pub(crate) fn instruction_finished(game: &mut Game) {
    let mut runtime = Runtime::new(game);
    let key = current(&runtime);
    runtime.edit_actor(key, Native, |actor| actor.variables[4] = 1);
}

/// False preserves the caller's existing invalid-state diagnostic.
pub(crate) fn advance(game: &mut Game) -> bool {
    let mut runtime = Runtime::new(game);
    let key = current(&runtime);
    let mut sound = false;
    let phase = runtime.resolve_actor(key, Native).variables[1];
    match phase {
        0xbbed => {
            let delta = api::player_delta(&runtime, key, Axis::X) as i16;
            if (delta.wrapping_abs() as u16) < 0xa0 {
                aim_velocity(&mut runtime, key, 2, [0xa2bbbb, 0xa2bbbf], 0);
                animate(&mut runtime, 0xbb7f);
                runtime.edit_actor(key, Native, |actor| actor.variables[1] = 0xbc48);
                sound = !api::current_outside_screen(&runtime);
            }
        }
        0xbc32 => {
            if runtime.resolve_actor(key, Native).variables[4] != 0 {
                runtime.edit_actor(key, Native, |actor| actor.variables[4] = 0);
                animate(&mut runtime, 0xbb53);
                runtime.edit_actor(key, Native, |actor| actor.variables[1] = 0xbbed);
            }
        }
        0xbc48 => {
            if !bounce_horizontal(&mut runtime, key, 3) && !move_axis(&mut runtime, key, Axis::Y, 2)
            {
                let falling = runtime.edit_actor(key, Native, |actor| {
                    actor.variables[2] = actor.variables[2].wrapping_sub(24);
                    if (actor.variables[2] as i16) < 0 {
                        actor.variables[0] = actor.variables[3];
                        actor.variables[3] = 0;
                        actor.variables[2] = 0;
                        actor.variables[1] = 0xbcff;
                        true
                    } else {
                        false
                    }
                });
                if !falling && runtime.resolve_actor(key, Native).variables[4] != 0 {
                    runtime.edit_actor(key, Native, |actor| actor.variables[4] = 0);
                    animate(&mut runtime, 0xbb97);
                }
            } else {
                runtime.edit_actor(key, Native, |actor| {
                    actor.variables[2] = actor.variables[2].wrapping_neg();
                    actor.variables[1] = 0xbcb7;
                });
            }
        }
        0xbcb7 => {
            bounce_horizontal(&mut runtime, key, 3);
            if move_axis(&mut runtime, key, Axis::Y, 2) {
                animate(&mut runtime, 0xbba3);
                runtime.edit_actor(key, Native, |actor| actor.variables[1] = 0xbc32);
            } else {
                runtime.edit_actor(key, Native, |actor| {
                    actor.variables[2] = actor.variables[2].wrapping_sub(24);
                });
            }
        }
        0xbcff => {
            let y = runtime.resolve_actor(key, Native).y.whole;
            if (y.wrapping_sub(runtime.session().player.y) as i16) >= 0 {
                runtime.edit_actor(key, Native, |actor| {
                    actor.variables[3] = actor.variables[0];
                    actor.variables[2] = u16::MAX;
                    actor.variables[1] = 0xbcb7;
                });
            } else {
                let angle = usize::from(api::angle_to_player(&runtime, key));
                runtime.edit_actor(key, Native, |actor| {
                    actor.variables[3] = SINE_COSINE[angle + 64] as u16;
                    actor.variables[2] = SINE_COSINE[angle] as u16;
                });
                move_axis(&mut runtime, key, Axis::X, 3);
                move_axis(&mut runtime, key, Axis::Y, 2);
            }
        }
        _ => return false,
    }
    drop(runtime);
    if sound {
        crate::game::music::queue_sfx2_max6(0x65, game);
    }
    true
}
