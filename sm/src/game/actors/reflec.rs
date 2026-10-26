//! Reflec palette cycling and projectile reflection with checked actor access.
#![forbid(unsafe_code)]
use super::access::{AccessMode::Native, ActorAccess, ActorKey as Key};
use super::{api, combat};
use crate::game::enemy_projectiles::{host::ShotMember, set_shot_direction, shot_word};
use crate::game::enemy_script::EnemyScript;
use crate::game::player_projectiles::{
    access::{shot, shot_lane},
    content::table,
    host::Host,
    motion,
};
use crate::game::session_runtime::Runtime;
use crate::game_state::Game;
use crate::ram_map::{word_arrays, words};

pub(crate) fn palette(game: &mut Game) {
    let mut runtime = Runtime::new(game);
    let hook = word_arrays::variables_for_enemy_graphics_drawn_hook;
    if runtime.word(words::door_transition_flag_enemies) != 0 {
        return;
    }
    runtime.write_slice_le(hook + 4, runtime.word(hook + 4).wrapping_sub(1));
    if runtime.word(hook + 4) != 0 {
        return;
    }
    runtime.write_slice_le(hook + 4, 16);
    let mut destination = runtime.word(hook);
    let mut source = runtime.word(hook + 2).wrapping_mul(8);
    for _ in 0..4 {
        let color = table(&runtime, 0xa3dabc, (source >> 1) as isize);
        runtime.write_slice_native(
            word_arrays::palette_buffer + 2 * (usize::from(destination >> 1) + 137),
            color,
        );
        source = source.wrapping_add(2);
        destination = destination.wrapping_add(2);
    }
    runtime.write_slice_le(hook + 2, (u16::from(runtime.byte(hook + 2)) + 1) & 7);
}
pub(crate) fn initialize(game: &mut Game) {
    let mut runtime = Runtime::new(game);
    let key = combat::current(&runtime);
    runtime.edit_actor(key, Native, |actor| actor.properties |= 0x1000);
    let parameter = runtime.resolve_actor(key, Native).parameters[0];
    let instruction = table(&runtime, 0xa3dc0b, parameter as isize);
    runtime.edit_actor(key, Native, |actor| actor.instruction = instruction);
    runtime.write_slice_native(0x178c, 0xdb0c);
    runtime.write_slice_native(0x178e, 0x00a3);
    let palette =
        ((16 * u32::from(runtime.resolve_actor(key, Native).palette) & 0xff00) >> 8) as u16;
    runtime.write_slice_le(
        word_arrays::variables_for_enemy_graphics_drawn_hook,
        palette,
    );
    runtime.write_slice_le(word_arrays::variables_for_enemy_graphics_drawn_hook + 4, 16);
}
/// A redirected target requires the retained native diagnostic before absorption.
pub(crate) fn prepare_shot(game: &mut Game) -> Option<u16> {
    let mut runtime = Runtime::new(game);
    let raw = runtime
        .word(words::collision_detection_index)
        .wrapping_mul(2);
    let key = combat::current(&runtime);
    runtime.edit_actor(key, Native, |actor| actor.invincibility = 10);
    let parameter = runtime.resolve_actor(key, Native).parameters[1];
    let shot_index = (raw >> 1) as isize;
    let target = parameter
        .wrapping_mul(32)
        .wrapping_add((shot_word(&runtime, ShotMember::Direction, shot_index) & 15) * 2);
    let index = (target >> 1) as isize;
    if table(&runtime, 0xa3dcae, index) == 0x8000 {
        combat::mark_shot(&mut runtime, shot_index);
        Some(target)
    } else {
        let mut direction = table(&runtime, 0xa3dcae, index);
        if direction & 0x8000 != 0 {
            direction = table(&runtime, 0xa3dcae, (target >> 1) as isize).wrapping_neg();
        }
        set_shot_direction(&mut runtime, shot_index, direction);
        shot!(runtime, raw, kind = shot!(runtime, raw, kind) & !0x8000; SliceLe);
        runtime.owned_call(|runtime| motion::reflect(runtime, raw));
        runtime.sound(2, 6, 0x57);
        None
    }
}
pub(crate) fn absorb(raw: u16, game: &mut Game) {
    let mut runtime = Runtime::new(game);
    let target = Key::from_legacy(raw);
    if runtime.resolve_actor(target, Native).health != 0 {
        combat::shot_reaction(&mut runtime, api::ShotHandling::ReactionOnly);
        if runtime.resolve_actor(target, Native).health == 0 {
            let parameter = runtime.resolve_actor(target, Native).parameters[1];
            let instruction = table(&runtime, 0xa3dca6, parameter as isize);
            runtime.edit_actor(target, Native, |actor| actor.restart_animation(instruction));
        }
    }
}
pub(crate) fn set_parameter(raw: u16, script: EnemyScript, game: &mut Game) -> EnemyScript {
    let parameter = script.word(0);
    Runtime::new(game).edit_actor(Key::from_legacy(raw), Native, |actor| {
        actor.parameters[1] = parameter
    });
    script.offset(1)
}
