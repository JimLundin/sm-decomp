//! Bang shares native flight and speed changes across its pursuit phases.
#![forbid(unsafe_code)]
use super::access::{AccessMode::Native, ActorAccess, ActorKey as Key};
use super::{api, combat};
use crate::game::enemy_projectiles::{host::ShotMember, shot_word};
use crate::game::session_runtime::Runtime;
use crate::game_state::Game;
use sm_game::enemy_script::EnemyScript;
use sm_game::player_projectiles::content::{self, table};

fn selected(runtime: &Runtime<'_>) -> (Key, usize) {
    let key = combat::current(runtime);
    (key, 0x7800 + usize::from(key.byte_index()))
}
fn animate(runtime: &mut Runtime<'_>) {
    let (key, at) = selected(runtime);
    let stage = runtime.native_word(at + 0x800);
    if stage != runtime.native_word(at + 0x802) {
        runtime.write_slice_native(at + 0x802, stage);
        let instruction = table(runtime, 0xa3b722, stage as isize);
        runtime.edit_actor(key, Native, |actor| actor.restart_animation(instruction));
    }
}
pub(crate) fn initialize(game: &mut Game) {
    let mut runtime = Runtime::new(game);
    let (key, at) = selected(&runtime);
    runtime.edit_actor(key, Native, |actor| {
        actor.variables[1] = actor.palette;
        actor.properties |= 0x1000;
        actor.pre_instruction = 0xbca5;
    });
    runtime.write_slice_native(at, 16);
    for offset in [2, 4, 0x800, 0x802, 0x804] {
        runtime.write_slice_native(at + offset, 0);
    }
    let parameter = runtime.resolve_actor(key, Native).parameters[1].to_ne_bytes();
    let limit = table(&runtime, 0xa3ba84, parameter[0] as isize);
    runtime.write_slice_native(at + 22, limit);
    let index = 2 * parameter[1] as isize;
    let delay = table(&runtime, 0xa3bc6a, index);
    runtime.write_slice_native(at + 24, delay);
    runtime.write_slice_native(at + 26, delay);
    let delay = table(&runtime, 0xa3bc6a, index + 1);
    runtime.write_slice_native(at + 28, delay);
    runtime.edit_actor(key, Native, |actor| {
        if actor.instruction == 0 {
            actor.pre_instruction = 0xbcc1;
        }
        actor.instruction = 0xb75e;
    });
}
pub(crate) fn main_address(game: &mut Game) -> u32 {
    let runtime = Runtime::new(game);
    u32::from(
        runtime
            .resolve_actor(combat::current(&runtime), Native)
            .parameters[0],
    ) | 0xa30000
}
pub(crate) fn wait(game: &mut Game) {
    let mut runtime = Runtime::new(game);
    let (key, at) = selected(&runtime);
    let remaining = runtime.native_word(at).wrapping_sub(1);
    runtime.write_slice_native(at, remaining);
    if remaining == 0 {
        runtime.write_slice_native(at, 16);
        runtime.edit_actor(key, Native, |actor| actor.pre_instruction = 0xbcc5);
    }
}
fn player_angle(runtime: &Runtime<'_>) -> u16 {
    u16::from((api::angle_to_player(runtime, combat::current(runtime)) as u8).wrapping_sub(64))
}
pub(crate) fn aim(game: &mut Game) {
    let mut runtime = Runtime::new(game);
    let angle = player_angle(&runtime);
    let (key, at) = selected(&runtime);
    runtime.write_slice_native(at + 2, angle);
    runtime.edit_actor(key, Native, |actor| actor.pre_instruction = 0xbd1c);
    for offset in [14, 16, 18, 20] {
        runtime.write_slice_native(at + offset, 0);
    }
}
pub(crate) fn fly(slowing: bool, game: &mut Game) {
    let mut runtime = Runtime::new(game);
    for (axis, adjustment, flag) in [(api::Axis::X, 64, 14), (api::Axis::Y, 0, 16)] {
        let (key, at) = selected(&runtime);
        let angle = (runtime.native_word(at + 2) as u8).wrapping_add(adjustment);
        let sine = table(&runtime, 0xa0b1c3, angle as isize);
        if sine & 0x8000 != 0 {
            runtime.write_slice_native(at + flag, 1);
        }
        let mut velocity =
            i32::from((sine as i16).unsigned_abs() >> 8) * i32::from(runtime.native_word(at + 20));
        if runtime.native_word(at + flag) != 0 {
            velocity = -velocity;
        }
        runtime.edit_actor(key, Native, |actor| match axis {
            api::Axis::X => actor.x.advance(velocity),
            api::Axis::Y => actor.y.advance(velocity),
        });
    }
    let (_, at) = selected(&runtime);
    let remaining = runtime.native_word(at + 24).wrapping_sub(1);
    runtime.write_slice_native(at + 24, remaining);
    if remaining & 0x8000 != 0 {
        let delay = runtime.native_word(at + if slowing { 28 } else { 26 });
        runtime.write_slice_native(at + 24, delay);
        let speed = runtime.native_word(at + 20);
        if slowing || speed.wrapping_sub(runtime.native_word(at + 22)) & 0x8000 != 0 {
            let acceleration =
                runtime
                    .native_word(at + 18)
                    .wrapping_add(if slowing { 0xffea } else { 22 });
            runtime.write_slice_native(at + 18, acceleration);
            let next = if slowing {
                speed.wrapping_sub(acceleration)
            } else {
                speed.wrapping_add(acceleration)
            };
            runtime.write_slice_native(at + 20, next);
        }
    }
    if slowing {
        let (key, at) = selected(&runtime);
        if runtime.native_word(at + 20) != 0 || runtime.native_word(at + 18) as i16 <= 0 {
            runtime.edit_actor(key, Native, |actor| actor.pre_instruction = 0xbca5);
        }
    } else {
        let angle = player_angle(&runtime);
        let (key, at) = selected(&runtime);
        runtime.write_slice_native(at + 4, angle);
        let difference = crate::rtl::sign_extend8(angle.wrapping_sub(runtime.native_word(at + 2)));
        if (difference as i16).unsigned_abs().wrapping_sub(48) & 0x8000 == 0 {
            runtime.edit_actor(key, Native, |actor| actor.pre_instruction = 0xbd2c);
        }
    }
}
pub(crate) fn shot(game: &mut Game) {
    let mut runtime = Runtime::new(game);
    let (key, at) = selected(&runtime);
    if runtime.resolve_actor(key, Native).pre_instruction != 0xbcc1 {
        let image = content::image(&runtime);
        let index = runtime.session().room.collision_detection_index as isize;
        let direction = shot_word(&runtime, ShotMember::Direction, index) & 15;
        let angle = image.table_word(content::offset(0xa3ba94), direction as isize);
        runtime.write_slice_native(at + 2, angle);
        runtime.edit_actor(key, Native, |actor| actor.pre_instruction = 0xbd2c);
        runtime.write_slice_native(at + 14, 0);
        runtime.write_slice_native(at + 16, 0);
        runtime.write_slice_native(at + 18, 256);
        runtime.write_slice_native(at + 20, 1536);
    }
    let stage = runtime.native_word(at + 0x800);
    if stage != 9 {
        runtime.write_slice_native(at + 0x800, stage.wrapping_add(1));
        animate(&mut runtime);
        let index = runtime.session().room.collision_detection_index as isize;
        let damage = shot_word(&runtime, ShotMember::Damage, index);
        runtime.edit_actor(key, Native, |actor| {
            actor.variables[4] = actor.variables[4].wrapping_add(damage)
        });
        combat::mark_shot(&mut runtime, index);
        if runtime.native_word(at + 0x800) == 9 {
            runtime.edit_actor(key, Native, |actor| actor.variables[3] = 1);
        }
    }
}
pub(crate) fn arm(cursor: EnemyScript, game: &mut Game) -> EnemyScript {
    let mut runtime = Runtime::new(game);
    let (_, at) = selected(&runtime);
    runtime.write_slice_native(at + 0x804, 1);
    cursor
}
