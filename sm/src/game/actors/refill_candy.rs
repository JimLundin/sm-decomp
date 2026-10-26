//! Maridia refill creatures rise toward Samus, dash, and return off screen.
#![forbid(unsafe_code)]
use super::access::{AccessMode::Native, ActorAccess};
use super::{api, combat};
use crate::game::enemy_projectiles::host::Effects;
use crate::game::enemy_script::EnemyScript;
use crate::game::player_projectiles::content::table;
use crate::game::session_runtime::Runtime;
use crate::game_state::Game;

fn animate(runtime: &mut Runtime<'_>) {
    let key = combat::current(runtime);
    let direction = runtime.resolve_actor(key, Native).variables[3];
    if direction != runtime.resolve_actor(key, Native).variables[4] {
        runtime.edit_actor(key, Native, |actor| actor.variables[4] = direction);
        let instruction = table(runtime, 0xa3b40d, direction as isize);
        runtime.edit_actor(key, Native, |actor| actor.restart_animation(instruction));
    }
}
pub(crate) fn initialize(game: &mut Game) {
    let mut runtime = Runtime::new(game);
    let key = combat::current(&runtime);
    runtime.edit_actor(key, Native, |actor| {
        actor.parameters[0] = 0xb482;
        actor.variables[3] = 0;
        actor.variables[4] = 0;
    });
    runtime.write_slice_native(0x7800 + usize::from(key.byte_index()), 0);
    runtime.edit_actor(key, Native, |actor| {
        actor.instruction = 0xb3c1;
        actor.properties |= 0x100;
        actor.variables[1] = actor.x.whole;
        actor.variables[2] = actor.y.whole;
    });
}
pub(crate) fn advance(game: &mut Game) {
    let mut runtime = Runtime::new(game);
    let key = combat::current(&runtime);
    let phase = runtime.resolve_actor(key, Native).parameters[0];
    match phase {
        0xb482 => {
            let current = combat::current(&runtime);
            if api::player_within(&mut runtime, current, api::Axis::X, 0x80) {
                let current = combat::current(&runtime);
                let direction = if api::player_delta(&runtime, current, api::Axis::X) & 0x8000 == 0
                {
                    3
                } else {
                    1
                };
                let key = combat::current(&runtime);
                runtime.edit_actor(key, Native, |actor| actor.variables[3] = direction);
                animate(&mut runtime);
                runtime.edit_actor(key, Native, |actor| actor.parameters[0] = 0xb4a8);
            }
        }
        0xb4a8 => {
            runtime.edit_actor(key, Native, |actor| actor.properties &= !0x100);
            if api::player_delta(&runtime, key, api::Axis::Y) & 0x8000 != 0 {
                runtime.edit_actor(key, Native, |actor| actor.y.advance(-0x8000));
            } else {
                runtime.edit_actor(key, Native, |actor| {
                    actor.variables[3] = actor.variables[3].wrapping_sub(1)
                });
                animate(&mut runtime);
                runtime.edit_actor(key, Native, |actor| {
                    actor.pre_instruction = 0;
                    actor.parameters[0] = 0xb4d6;
                });
            }
        }
        0xb4d6 => {
            let index = (runtime.native_word(0x7800 + usize::from(key.byte_index())) >> 1) as isize;
            let high = table(&runtime, 0xa3b415, index);
            let low = table(&runtime, 0xa3b415, index + 1);
            let speed = ((u32::from(high) << 16) | u32::from(low)) as i32;
            // Preserve ordinary signed negation, including its checked-build overflow.
            let speed = if runtime.resolve_actor(key, Native).variables[3] != 0 {
                speed
            } else {
                -speed
            };
            runtime.edit_actor(key, Native, |actor| actor.x.advance(speed));
            let outside = api::current_outside_screen(&runtime);
            if outside {
                runtime.edit_actor(key, Native, |actor| {
                    actor.properties |= 0x100;
                    actor.x.whole = actor.variables[1];
                    actor.y.whole = actor.variables[2];
                    actor.variables[3] = 0;
                });
            }
            animate(&mut runtime);
            if outside {
                runtime.edit_actor(key, Native, |actor| actor.parameters[0] = 0xb482);
            }
        }
        _ => runtime.unreachable(),
    }
}
pub(crate) fn instruction(value: u16, script: EnemyScript, game: &mut Game) -> EnemyScript {
    let mut runtime = Runtime::new(game);
    let key = combat::current(&runtime);
    runtime.write_slice_native(0x7800 + usize::from(key.byte_index()), value);
    script
}
