//! Evir's body, attachment and projectile share positioning and damage state.
//! Extended words and widened peer aliases remain RAM-authoritative.
#![forbid(unsafe_code)]
use super::access::{AccessMode::Native, ActorAccess, ActorKey as Key};
use super::{api, combat};
use crate::game::enemy_speeds::SpeedTable;
use crate::game::session_runtime::{Runtime, actor_cosine_full, actor_sine_full};
use crate::game_state::Game;
use crate::ram_map::{word_arrays as A, words as W};
use sm_game::enemy_projectiles::host::Effects;
use sm_game::enemy_script::EnemyScript;

fn selected(runtime: &Runtime<'_>) -> Key {
    Key::from_legacy(runtime.word(W::cur_enemy_index))
}
fn extra(key: Key) -> usize {
    0x7800 + usize::from(key.byte_index())
}
fn drawing(key: Key, word: usize) -> usize {
    A::enemy_drawing_queue + usize::from(key.byte_index() & !1) + 2 * word
}
fn animate(runtime: &mut Runtime<'_>) {
    let key = selected(runtime);
    let at = extra(key);
    let instruction = runtime.native_word(at + 4);
    if instruction != runtime.native_word(at + 2) {
        runtime.edit_actor(key, Native, |actor| actor.instruction = instruction);
        runtime.write_slice_native(at + 2, instruction);
        runtime.edit_actor(key, Native, |actor| {
            actor.instruction_delay = 1;
            actor.timer = 0;
        });
    }
}
fn face_player(runtime: &mut Runtime<'_>) {
    let key = selected(runtime);
    let right = api::player_delta(runtime, key, api::Axis::X) & 0x8000 == 0;
    runtime.edit_actor(key, Native, |actor| actor.variables[1] = u16::from(right));
    let key = selected(runtime);
    runtime.write_slice_native(extra(key) + 4, 0x86a7);
    if runtime.resolve_actor(key, Native).variables[1] != 0 {
        runtime.write_slice_native(extra(key) + 4, 0x870b);
    }
    animate(runtime);
}
fn attach(runtime: &mut Runtime<'_>, projectile: bool) {
    let key = selected(runtime);
    let (facing, x_word, y_offset) = if projectile {
        (drawing(key, 83), 59, 18)
    } else {
        (
            A::enemy_drawing_queue_sizes + usize::from(key.byte_index() & !1) + 2,
            91,
            10,
        )
    };
    let facing = runtime.word(facing);
    let key = selected(runtime);
    runtime.edit_actor(key, Native, |actor| actor.variables[1] = facing);
    let x = runtime.word(drawing(key, x_word));
    runtime.edit_actor(key, Native, |actor| {
        actor.x.whole = if facing != 0 {
            x.wrapping_add(4)
        } else {
            x.wrapping_sub(4)
        };
    });
    let y = runtime
        .word(drawing(key, x_word + 2))
        .wrapping_add(y_offset);
    runtime.edit_actor(key, Native, |actor| actor.y.whole = y);
    if !projectile {
        runtime.write_slice_native(extra(key) + 4, if facing != 0 { 0x8727 } else { 0x86c3 });
        animate(runtime);
    }
}
pub(crate) fn initialize(game: &mut Game) {
    let mut runtime = Runtime::new(game);
    let key = selected(&runtime);
    if runtime.resolve_actor(key, Native).parameters[0] != 0 {
        attach(&mut runtime, false);
        runtime.edit_actor(key, Native, |actor| actor.layer = 4);
    } else {
        face_player(&mut runtime);
        let index =
            u16::from(runtime.resolve_actor(key, Native).parameters[1].to_ne_bytes()[0]) * 8;
        for (word, field) in [(0, 8), (1, 6), (2, 12), (3, 10)] {
            let value = runtime.enemy_speed(SpeedTable::Linear, (index >> 1) as isize + word);
            runtime.write_slice_native(extra(key) + field, value);
        }
        // The source's reload parameter belongs to the speed-derived actor index.
        let delay = runtime.byte(0xf78 + usize::from(index) + 0x3f) >> 1;
        runtime.edit_actor(key, Native, |actor| actor.variables[4] = u16::from(delay));
    }
    runtime.write_slice_native(extra(key), 0);
    runtime.write_slice_native(extra(key) + 2, 0);
    runtime.edit_actor(key, Native, |actor| actor.variables[2] = 0x8922);
}
pub(crate) fn initialize_projectile(game: &mut Game) {
    let mut runtime = Runtime::new(game);
    let key = selected(&runtime);
    runtime.write_slice_native(extra(key) + 4, 0x876f);
    animate(&mut runtime);
    let live = selected(&runtime);
    let palette = runtime.word(drawing(live, 73));
    runtime.edit_actor(key, Native, |actor| actor.palette = palette);
    let tile = runtime.word(drawing(live, 74));
    runtime.edit_actor(key, Native, |actor| actor.tile = tile);
    attach(&mut runtime, true);
    for field in [2, 24, 22] {
        runtime.write_slice_native(extra(key) + field, 0);
    }
    runtime.edit_actor(key, Native, |actor| {
        actor.pre_instruction = 0;
        actor.variables[2] = 0x8a34;
    });
}
fn bob(runtime: &mut Runtime<'_>) {
    let key = selected(runtime);
    if runtime.native_word(extra(key) + 128 + 24) == 0 {
        face_player(runtime);
    }
    let key = selected(runtime);
    let at = extra(key);
    let high = if runtime.native_word(at) != 0 { 8 } else { 12 };
    let speed = (u32::from(runtime.native_word(at + high)) << 16)
        | u32::from(runtime.native_word(at + high - 2));
    runtime.edit_actor(key, Native, |actor| actor.y.advance(speed as i32));
    let reverse = runtime.edit_actor(key, Native, |actor| {
        actor.variables[4] = actor.variables[4].wrapping_sub(1);
        if actor.variables[4] & 0x8000 != 0 {
            actor.variables[4] = u16::from(actor.parameters[1].to_ne_bytes()[1]);
            true
        } else {
            false
        }
    });
    if reverse {
        runtime.write_slice_native(at, runtime.native_word(at) ^ 1);
    }
}
fn launch(runtime: &mut Runtime<'_>) {
    let parent = Key::from_legacy(runtime.word(W::cur_enemy_index).wrapping_sub(128));
    if !api::player_within(runtime, parent, api::Axis::X, 0x80) {
        return;
    }
    let key = selected(runtime);
    let angle =
        u16::from((api::angle_to_player(runtime, key) as u8).wrapping_sub(64)).wrapping_neg();
    let key = selected(runtime);
    for (field, vertical) in [(14, false), (18, true)] {
        let value = runtime.foreign(|game| {
            if vertical {
                actor_sine_full(angle, 4, game)
            } else {
                actor_cosine_full(angle, 4, game)
            }
        });
        runtime.write_slice_native(extra(key) + field + 2, value as u16);
        runtime.write_slice_native(extra(key) + field, (value >> 16) as u16);
    }
    runtime.write_slice_native(extra(key) + 4, 0x876f);
    animate(runtime);
    runtime.write_slice_native(extra(key) + 22, 1);
    runtime.edit_actor(key, Native, |actor| actor.variables[2] = 0x8a3b);
}
fn retire_flight(runtime: &mut Runtime<'_>) {
    if api::current_outside_padded_screen(runtime, 0x100)
        && runtime.native_word(drawing(selected(runtime), 77)) == 0
    {
        let key = selected(runtime);
        runtime.write_slice_native(extra(key) + 22, 0);
        runtime.write_slice_native(extra(key) + 24, 1);
        runtime.edit_actor(key, Native, |actor| actor.variables[2] = 0x8a78);
        runtime.write_slice_native(extra(key) + 24, 1);
        runtime.write_slice_native(extra(key) + 4, 0x8775);
        animate(runtime);
    }
}
fn dispatch(runtime: &mut Runtime<'_>, state: u16) {
    match 0xa80000 | u32::from(state) {
        0xa88922 => {
            if runtime.resolve_actor(selected(runtime), Native).parameters[0] != 0 {
                attach(runtime, false);
            } else {
                bob(runtime);
            }
        }
        0xa88a34 => attach(runtime, true),
        0xa88a3b => {
            retire_flight(runtime);
            let key = selected(runtime);
            for (field, vertical) in [(14, false), (18, true)] {
                let speed = (u32::from(runtime.native_word(extra(key) + field)) << 16)
                    | u32::from(runtime.native_word(extra(key) + field + 2));
                runtime.edit_actor(key, Native, |actor| {
                    if vertical {
                        actor.y.advance(speed as i32);
                    } else {
                        actor.x.advance(speed as i32);
                    }
                });
            }
        }
        0xa88a78 => {
            if runtime.native_word(drawing(selected(runtime), 77)) == 0 {
                let key = selected(runtime);
                if runtime.native_word(extra(key) + 24) != 0 {
                    attach(runtime, true);
                    runtime.edit_actor(key, Native, |actor| {
                        actor.x.whole = actor.x.whole.wrapping_add(actor.pre_instruction)
                    });
                } else {
                    runtime.write_slice_native(extra(key) + 4, 0x876f);
                    animate(runtime);
                    runtime.write_slice_native(extra(key) + 24, 0);
                    runtime.write_slice_native(extra(key) + 22, 0);
                    runtime.edit_actor(key, Native, |actor| actor.variables[2] = 0x8a34);
                }
            }
        }
        _ => runtime.unreachable(),
    }
}
pub(crate) fn advance(game: &mut Game) {
    let mut runtime = Runtime::new(game);
    let state = runtime.resolve_actor(selected(&runtime), Native).variables[2];
    dispatch(&mut runtime, state);
}
pub(crate) fn advance_projectile(game: &mut Game) {
    let mut runtime = Runtime::new(game);
    let key = selected(&runtime);
    if runtime.resolve_actor(key, Native).frozen == 0 {
        if runtime.native_word(extra(key) + 22) != 0 {
            runtime.write_slice_native(extra(key) + 4, 0x876f);
            animate(&mut runtime);
        } else if runtime.native_word(extra(key) + 24) != 0 {
            runtime.write_slice_native(extra(key) + 4, 0x8775);
            animate(&mut runtime);
        } else {
            launch(&mut runtime);
        }
    }
    let state = runtime.resolve_actor(key, Native).variables[2];
    dispatch(&mut runtime, state);
}
pub(crate) fn damage(address: u32, game: &mut Game) {
    let mut runtime = Runtime::new(game);
    match address {
        11045638 => combat::contact_reaction(&mut runtime, api::DeathHandling::Finalize),
        11045644 => combat::power_bomb_reaction(&mut runtime, api::DeathHandling::Finalize),
        11045650 => combat::shot_reaction(&mut runtime, api::ShotHandling::Finalize),
        _ => {
            runtime.unreachable();
            return;
        }
    }
    let key = selected(&runtime);
    let base = 0xf78 + usize::from(key.byte_index());
    // Peer indices widen before addition; high identities must not wrap to actor zero.
    if runtime.resolve_actor(key, Native).health == 0 {
        for offset in [64, 128] {
            let at = base + offset + 0x0e;
            runtime.write_slice_native(at, runtime.native_word(at) | 0x200);
        }
    }
    let frozen = runtime.resolve_actor(key, Native).frozen;
    if frozen != 0 {
        runtime.write_slice_native(base + 64 + 0x26, frozen);
        runtime.write_slice_native(base + 64 + 0x12, runtime.native_word(base + 64 + 0x12) | 4);
        if runtime.native_word(base + 128 + 0x34) != 0x8a3b {
            runtime.write_slice_native(
                base + 128 + 0x12,
                runtime.native_word(base + 128 + 0x12) | 4,
            );
            runtime
                .write_slice_native(base + 128 + 0x26, runtime.resolve_actor(key, Native).frozen);
        }
    }
}
pub(crate) fn instruction(address: u32, script: EnemyScript, game: &mut Game) -> EnemyScript {
    let mut runtime = Runtime::new(game);
    let key = selected(&runtime);
    match address {
        11044751 => runtime.queue_sound(2, 6, 0x5e),
        11044763 | 11044790 => {
            let left = runtime.native_word(0x7782 + usize::from(runtime.word(W::cur_enemy_index)))
                == 0x870b;
            let key = if address == 11044763 {
                selected(&runtime)
            } else {
                key
            };
            runtime.edit_actor(key, Native, |actor| {
                actor.pre_instruction = if address == 11044763 {
                    if left { 0xfff8 } else { 8 }
                } else if left {
                    actor.pre_instruction.wrapping_add(1)
                } else {
                    actor.pre_instruction.wrapping_sub(1)
                };
            });
        }
        11044811 => {
            runtime.write_slice_native(extra(key) + 24, 0);
            runtime.write_slice_native(extra(key) + 22, 0);
            runtime.edit_actor(key, Native, |actor| actor.variables[2] = 0x8a34);
        }
        _ => {
            runtime.unreachable();
            return EnemyScript::Stop;
        }
    }
    script
}
