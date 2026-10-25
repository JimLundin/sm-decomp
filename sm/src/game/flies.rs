//! Flies share fractional movement while circling and diving toward the player.
#![forbid(unsafe_code)]
use crate::game::actors::access::{AccessMode::Native, ActorAccess, ActorKey as Key};
use crate::game::actors::api::{self, Axis};
use crate::game::projectile_geometry::SINE_COSINE;
use crate::game::session_runtime::Runtime;
use crate::game_state::Game;

fn move_fly(runtime: &mut Runtime<'_>, key: Key, circling: bool) -> u16 {
    let angle = if circling {
        usize::from(runtime.resolve_actor(key, Native).variables[4] >> 1)
    } else {
        0
    };
    for axis in 0..2 {
        let velocity = if circling {
            SINE_COSINE[angle + if axis == 0 { 64 } else { 0 }] as u16
        } else {
            runtime.resolve_actor(key, Native).variables[axis + 1]
        };
        runtime.edit_actor(key, Native, |actor| {
            let position = if axis == 0 {
                &mut actor.x
            } else {
                &mut actor.y
            };
            position.advance_8_8(i16::from_le_bytes(velocity.to_ne_bytes()));
        });
    }
    runtime.resolve_actor(key, Native).y.whole
}

fn aim(runtime: &mut Runtime<'_>, key: Key) {
    let angle = usize::from(api::angle_to_player(runtime, key));
    for (variable, index, scale) in [(1, angle + 64, 2u16), (2, angle, 4)] {
        let speed = scale.wrapping_mul(SINE_COSINE[index] as u16);
        runtime.edit_actor(key, Native, |actor| actor.variables[variable] = speed);
    }
    let target_y = runtime.session().player.y;
    runtime.edit_actor(key, Native, |actor| {
        actor.variables[3] = target_y;
        actor.pre_instruction = 0xb1aa;
    });
}

pub(crate) fn initialize(game: &mut Game) {
    let mut runtime = Runtime::new(game);
    let key = Key::from_legacy(runtime.word(0xe54));
    runtime.edit_actor(key, Native, |actor| {
        actor.variables[4] = 0;
        actor.pre_instruction = 0xb14e;
        actor.instruction = 0xb013;
        actor.spritemap = 0x804d;
        actor.instruction_delay = 1;
        actor.spritemap = 0x804d;
    });
}

pub(crate) fn circle(k: u16, reverse: bool, game: &mut Game) {
    let mut runtime = Runtime::new(game);
    let key = Key::from_legacy(k);
    let cooldown = runtime.resolve_actor(key, Native).variables[0];
    if cooldown != 0 {
        runtime.edit_actor(key, Native, |actor| {
            actor.variables[0] = cooldown.wrapping_sub(1)
        });
    } else if ((api::player_delta(&runtime, key, Axis::X) as i16).wrapping_abs() as u16) < 0x70 {
        aim(&mut runtime, key);
        return;
    }
    move_fly(&mut runtime, key, true);
    runtime.edit_actor(key, Native, |actor| {
        actor.variables[4] =
            actor.variables[4].wrapping_add(if reverse { 0xffe0 } else { 32 }) & 0x1ff;
        if actor.variables[4] == 0 {
            actor.pre_instruction = if reverse { 0xb14e } else { 0xb17c };
        }
    });
}

pub(crate) fn dive(k: u16, returning: bool, game: &mut Game) {
    let mut runtime = Runtime::new(game);
    let key = Key::from_legacy(k);
    let y = move_fly(&mut runtime, key, false);
    runtime.edit_actor(key, Native, |actor| {
        if returning {
            actor.variables[0] = actor.variables[0].wrapping_sub(1);
            if actor.variables[0] & 0x8000 != 0 {
                actor.variables[0] = 24;
                actor.pre_instruction = 0xb14e;
            }
        } else {
            actor.variables[0] = actor.variables[0].wrapping_add(1);
            let reverse = if actor.variables[2] & 0x8000 == 0 {
                y >= actor.variables[3]
            } else {
                y < actor.variables[3]
            };
            if reverse {
                actor.variables[2] = actor.variables[2].wrapping_neg();
                actor.pre_instruction = 0xb1d2;
            }
        }
    });
}
