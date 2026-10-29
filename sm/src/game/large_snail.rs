//! Large Snail movement and its ordered carrying/resistance interaction with Samus.
#![forbid(unsafe_code)]
use crate::game::actors::access::{AccessMode::Native, ActorAccess, ActorKey as Key};
use crate::game::actors::{animation, api, combat};
use crate::game::enemy_projectiles::host::Effects;
use crate::game::enemy_speeds::SpeedTable;
use crate::game::session_runtime::Runtime;
use crate::game_state::Game;
use crate::ram_map::words as fields;

fn selected(runtime: &Runtime<'_>) -> u16 {
    runtime.word(fields::cur_enemy_index)
}
fn location(raw: u16) -> (Key, usize) {
    (Key::from_legacy(raw), 0x7800 + usize::from(raw))
}
fn animate(runtime: &mut Runtime<'_>) {
    let raw = usize::from(selected(runtime));
    animation::restart_from_native_cache(runtime, raw, 0xa2cb77);
}
fn near(runtime: &mut Runtime<'_>, axis: api::Axis, distance: u16) -> bool {
    let key = Key::from_legacy(selected(runtime));
    api::player_within(runtime, key, axis, distance)
}
fn player_left(runtime: &Runtime<'_>) -> bool {
    api::player_delta(runtime, Key::from_legacy(selected(runtime)), api::Axis::X) & 0x8000 != 0
}
fn speed_word(runtime: &Runtime<'_>, index: isize) -> u16 {
    runtime.enemy_speed(SpeedTable::Linear, index)
}
pub(crate) fn initialize(game: &mut Game) {
    let mut runtime = Runtime::new(game);
    let (key, at) = location(selected(&runtime));
    runtime.edit_actor(key, Native, |actor| {
        actor.variables[1] = 0;
        actor.variables[3] = 0;
    });
    for offset in [0, 2, 4, 6, 20] {
        runtime.write_slice_native(at + offset, 0);
    }
    runtime.edit_actor(key, Native, |actor| {
        actor.variables[4] = 3;
        actor.variables[2] = 128;
        actor.instruction = 0xca4b;
        actor.variables[0] = 0xcde6;
        actor.pre_instruction = 0xcf66;
    });
}
pub(crate) fn prepare(game: &mut Game) -> u16 {
    let mut runtime = Runtime::new(game);
    let (key, at) = location(selected(&runtime));
    runtime.write_slice_native(at + 8, 0);
    runtime.write_slice_native(at + 10, 0);
    if near(&mut runtime, api::Axis::Y, 0x20) && near(&mut runtime, api::Axis::X, 0x18) {
        runtime.write_slice_native(at + 8, 1);
        if player_left(&runtime) {
            runtime.write_slice_native(at + 10, 1);
        }
    }
    let x = runtime.resolve_actor(key, Native).x.whole;
    runtime.write_slice_native(at + 12, x);
    let y = runtime.resolve_actor(key, Native).y.whole;
    runtime.write_slice_native(at + 16, y);
    let key = Key::from_legacy(selected(&runtime));
    runtime.resolve_actor(key, Native).variables[0]
}
/// The caller runs an arbitrary legacy preinstruction before the walking continuation.
pub(crate) fn begin_action(walking: bool, game: &mut Game) -> (u16, Option<u16>) {
    let mut runtime = Runtime::new(game);
    let raw = selected(&runtime);
    let (key, at) = location(raw);
    if runtime.resolve_actor(key, Native).variables[4] != 0 {
        return (
            raw,
            Some(runtime.resolve_actor(key, Native).pre_instruction),
        );
    }
    if walking {
        let current = Key::from_legacy(selected(&runtime));
        if api::move_vertical(&mut runtime, current, 1 << 16) != api::MovementResult::Blocked {
            runtime.edit_actor(key, Native, |actor| {
                actor.variables[1] = 0;
                actor.pre_instruction = 0xcf66;
                actor.variables[4] = 3;
            });
        }
    } else {
        runtime.write_slice_native(at, 0);
        if player_left(&runtime) {
            runtime.edit_actor(key, Native, |actor| actor.variables[3] = 1);
            runtime.write_slice_native(at, 1);
        }
        animate(&mut runtime);
        if near(&mut runtime, api::Axis::X, 0x18) {
            runtime.write_slice_native(at, runtime.native_word(at) | 2);
            animate(&mut runtime);
            runtime.edit_actor(key, Native, |actor| actor.variables[0] = 0xce2b);
        }
    }
    (raw, None)
}
fn pushed(runtime: &Runtime<'_>, key: Key, at: usize) -> bool {
    runtime.native_word(at + 8) != 0
        && runtime.resolve_actor(key, Native).variables[3] == runtime.native_word(at + 10)
}
pub(crate) fn walk(raw: u16, game: &mut Game) {
    let mut runtime = Runtime::new(game);
    let (key, at) = location(raw);
    if runtime.native_word(at + 22) != 0 {
        return;
    }
    let expired = runtime.edit_actor(key, Native, |actor| {
        actor.variables[2] = actor.variables[2].wrapping_sub(1);
        actor.variables[2] & 0x8000 != 0
    });
    let close = if expired {
        runtime.edit_actor(key, Native, |actor| actor.variables[2] = 0);
        near(&mut runtime, api::Axis::X, 0x20)
    } else {
        false
    };
    if close
        && runtime.native_word(at + 6) != 0
        && runtime.resolve_actor(key, Native).variables[4] == 0
    {
        runtime.write_slice_native(at, 0);
        runtime.edit_actor(key, Native, |actor| actor.variables[3] = 0);
        if player_left(&runtime) {
            runtime.write_slice_native(at, 1);
            runtime.edit_actor(key, Native, |actor| actor.variables[3] = 1);
        }
        runtime.edit_actor(key, Native, |actor| actor.variables[2] = 128);
        runtime.write_slice_native(at, runtime.native_word(at) & 1 | 4);
        animate(&mut runtime);
        runtime.edit_actor(key, Native, |actor| actor.variables[0] = 0xcf40);
    } else {
        let index = if runtime.resolve_actor(key, Native).variables[3] != 0 {
            66
        } else {
            64
        };
        let fraction = speed_word(&runtime, index + 1);
        let mut whole = speed_word(&runtime, index);
        if pushed(&runtime, key, at) {
            let adjustment = if runtime.resolve_actor(key, Native).variables[3] != 0 {
                -16i16
            } else {
                16
            };
            whole = whole.wrapping_add(adjustment as u16);
        }
        let current = Key::from_legacy(selected(&runtime));
        let speed = ((u32::from(whole) << 16) | u32::from(fraction)) as i32;
        let blocked = api::move_horizontal(&mut runtime, current, speed, api::SlopePolicy::Solid)
            == api::MovementResult::Blocked;
        if pushed(&runtime, key, at) {
            runtime.edit_actor(key, Native, |actor| {
                let adjustment = if actor.variables[3] != 0 { 16i16 } else { -16 };
                actor.x.whole = actor.x.whole.wrapping_add(adjustment as u16);
            });
        }
        if blocked {
            runtime.edit_actor(key, Native, |actor| actor.variables[3] ^= 1);
            runtime.write_slice_native(at, runtime.native_word(at) ^ 4);
            animate(&mut runtime);
        }
    }
}
pub(crate) fn end_action(game: &mut Game) {
    let mut runtime = Runtime::new(game);
    let (key, at) = location(selected(&runtime));
    if runtime.native_word(at + 4) != 0 {
        runtime.write_slice_native(at + 4, 0);
        runtime.write_slice_native(at, runtime.native_word(at).wrapping_sub(2));
        animate(&mut runtime);
        runtime.edit_actor(key, Native, |actor| actor.variables[0] = 0xce2b);
    }
}
/// Carry first, then resist. These disjoint writes cannot alter the selected actor or delta.
pub(crate) fn finish(game: &mut Game) {
    let mut runtime = Runtime::new(game);
    let (key, at) = location(selected(&runtime));
    runtime.write_slice_native(at + 22, 0);
    if api::touches_player_from_above(&runtime, key) {
        let y = runtime.resolve_actor(key, Native).y.whole;
        let moved = y.wrapping_sub(runtime.native_word(at + 16));
        runtime.write_word_le(moved, fields::extra_samus_y_displacement);
    }
    if runtime.native_word(at + 8) == 0 {
        return;
    }
    let x = runtime.resolve_actor(key, Native).x.whole;
    let moved = x.wrapping_sub(runtime.native_word(at + 12)) as i16;
    let side = runtime.native_word(at + 10) != 0;
    if (moved < 0) == side {
        let carried = runtime
            .word(fields::extra_samus_x_displacement)
            .wrapping_add(moved as u16);
        runtime.write_word_le(carried, fields::extra_samus_x_displacement);
    }
    if moved == 0 && (runtime.resolve_actor(key, Native).variables[3] != 0) != side {
        return;
    }
    let negative = if moved == 0 { side } else { moved < 0 };
    let mask = if negative { 0x100 } else { 0x200 };
    if runtime.word(fields::joypad1_lastkeys) & mask != 0 {
        runtime.write_slice_native(at + 22, 1);
        let x = runtime.native_word(at + 12);
        runtime.edit_actor(key, Native, |actor| actor.x.whole = x);
    }
}
pub(crate) fn bounce(up: bool, raw: u16, game: &mut Game) {
    let mut runtime = Runtime::new(game);
    let key = Key::from_legacy(raw);
    let phase = runtime.edit_actor(key, Native, |actor| {
        let mut phase = actor.variables[1].wrapping_add(if up { -384i16 as u16 } else { 384 });
        if !up && phase.wrapping_sub(0x4000) & 0x8000 == 0 {
            phase = 0x4000;
        }
        actor.variables[1] = phase;
        phase
    });
    if up && (phase as i16) < 0 {
        runtime.edit_actor(key, Native, |actor| {
            actor.variables[1] = 0;
            actor.pre_instruction = 0xcf66;
        });
        return;
    }
    let image = runtime.native_image();
    let offset = crate::rom::lorom_offset(0xa0cbc7) as usize
        + 8 * usize::from(phase >> 8)
        + if up { 4 } else { 0 };
    let speed = i32::from_ne_bytes(core::array::from_fn(|i| image.read(offset + i)));
    let blocked = api::move_vertical(&mut runtime, key, speed) == api::MovementResult::Blocked;
    if !up && blocked {
        runtime.edit_actor(key, Native, |actor| {
            actor.variables[4] = actor.variables[4].wrapping_sub(1);
            let next = (actor.variables[1] as i16).wrapping_sub(4096);
            if next < 0 {
                actor.variables[4] = 0;
            }
            actor.variables[1] = next as u16;
            actor.pre_instruction = 0xcfa9;
        });
    }
}
pub(crate) fn touch(damage: bool, game: &mut Game) {
    let mut runtime = Runtime::new(game);
    if damage {
        combat::contact_reaction(&mut runtime, api::DeathHandling::Finalize);
    }
    let key = Key::from_legacy(selected(&runtime));
    if !api::touches_player_from_above(&runtime, key) {
        let key = Key::from_legacy(selected(&runtime));
        let x = runtime.resolve_actor(key, Native).x.whole;
        let left = (runtime.word(fields::samus_x_pos).wrapping_sub(x) as i16) < 0;
        let displacement = runtime
            .word(fields::extra_samus_x_displacement)
            .wrapping_add(if left { -4i16 as u16 } else { 4 });
        runtime.write_word_le(displacement, fields::extra_samus_x_displacement);
    }
}
pub(crate) fn shot(game: &mut Game) {
    let mut runtime = Runtime::new(game);
    combat::shot_reaction(&mut runtime, api::ShotHandling::Finalize);
    runtime.queue_sound(2, 6, 0x57);
}
pub(crate) fn script_flag(offset: usize, value: u16, game: &mut Game) {
    let mut runtime = Runtime::new(game);
    let at = 0x7800 + usize::from(selected(&runtime));
    runtime.write_slice_native(at + offset, value);
}
