//! Rising/falling actor: wait near Samus, rise, play native motion rows, then reset.
#![forbid(unsafe_code)]
use crate::game::actors::access::{AccessMode::Native, ActorAccess, ActorKey as Key};
use crate::game::actors::api::{self, Axis};
use crate::game::enemy_projectiles::host::Effects;
use crate::game::enemy_speeds::SpeedTable;
use crate::game::session_runtime::Runtime;
use crate::game_state::Game;
use crate::ram_map::words as W;

fn animate(runtime: &mut Runtime<'_>, instruction: u16) {
    let key = Key::from_legacy(runtime.word(W::cur_enemy_index));
    runtime.edit_actor(key, Native, |actor| actor.restart_animation(instruction));
}
fn table(runtime: &Runtime<'_>, address: u32, index: usize) -> u16 {
    runtime
        .loaded_image()
        .expect("ROM must be loaded")
        .table_word(crate::rom::lorom_offset(address) as usize, index as isize)
}
fn motion(runtime: &Runtime<'_>, index: usize) -> u32 {
    let high = runtime.enemy_speed(SpeedTable::Quadratic, (index + 1) as isize);
    let low = runtime.enemy_speed(SpeedTable::Quadratic, index as isize);
    (u32::from(high) << 16) | u32::from(low)
}
pub(crate) fn update(initialize: bool, game: &mut Game) {
    let mut runtime = Runtime::new(game);
    if initialize {
        animate(&mut runtime, 0xd82c);
    }
    let raw = runtime.word(W::cur_enemy_index);
    let key = Key::from_legacy(raw);
    let extra = 0x7800 + usize::from(raw);
    let base = 0xf78 + usize::from(raw);
    if initialize {
        runtime.edit_actor(key, Native, |actor| actor.variables[0] = 0xe035);
        for axis in 0..2 {
            let position = runtime.native_word(base + 2 + 4 * axis);
            runtime.write_slice_native(extra + 2 * axis, position);
        }
        let index = usize::from(runtime.resolve_actor(key, Native).parameters[0].to_ne_bytes()[1]);
        let script = table(&runtime, 0xa2df5e, index);
        runtime.edit_actor(key, Native, |actor| actor.variables[4] = script);
        let address = table(&runtime, 0xa2df6a, index);
        let stride = runtime.legacy_native_record(0xa2, address).word(0);
        runtime.edit_actor(key, Native, |actor| actor.pre_instruction = stride);
        let live = Key::from_legacy(runtime.word(W::cur_enemy_index));
        let height = {
            let actor = runtime.resolve_actor(live, Native);
            u16::from(actor.parameters[0] as u8) * u16::from(actor.pre_instruction as u8)
        };
        let live_extra = 0x7800 + usize::from(runtime.word(W::cur_enemy_index));
        let mut distance = 0u32;
        let mut velocity = 0u16;
        loop {
            velocity = velocity.wrapping_add(512);
            distance = distance.wrapping_add(motion(&runtime, 4 * usize::from(velocity >> 8)));
            if (distance >> 16).wrapping_sub(u32::from(height)) & 0x8000 == 0 {
                break;
            }
        }
        runtime.write_slice_native(live_extra + 12, velocity);
        runtime.write_slice_native(
            extra + 6,
            runtime.native_word(extra + 2).wrapping_sub(height),
        );
        runtime.write_slice_native(extra + 4, runtime.native_word(extra));
        let velocity = runtime.native_word(extra + 12);
        runtime.edit_actor(key, Native, |actor| actor.variables[1] = velocity);
        return;
    }
    let phase = runtime.resolve_actor(key, Native).variables[0];
    match phase {
        0xe035 => {
            if api::player_within(&mut runtime, key, Axis::X, 0x50) {
                let live = runtime.word(W::cur_enemy_index);
                let key = Key::from_legacy(live);
                let delay = runtime.resolve_actor(key, Native).parameters[1];
                runtime.write_slice_native(0x780e + usize::from(live), delay);
                runtime.edit_actor(key, Native, |actor| actor.variables[0] = 0xe04f);
            }
        }
        0xe04f => {
            let delay = runtime.native_word(extra + 14).wrapping_sub(1);
            runtime.write_slice_native(extra + 14, delay);
            if (delay as i16) < 0 {
                animate(&mut runtime, 0xd834);
                runtime.edit_actor(key, Native, |actor| actor.variables[0] = 0xe06a);
            }
        }
        0xe06a => {
            let index =
                usize::from(runtime.resolve_actor(key, Native).variables[1].to_ne_bytes()[1]);
            let amount = motion(&runtime, 4 * index + 2);
            // AddToHiLo reads and writes the fractional word before the whole word.
            let low = runtime.native_word(base + 8);
            let high = runtime.native_word(base + 6);
            let next = ((u32::from(high) << 16) | u32::from(low)).wrapping_add(amount);
            runtime.write_slice_native(base + 8, next as u16);
            runtime.write_slice_native(base + 6, (next >> 16) as u16);
            let velocity = runtime.edit_actor(key, Native, |actor| {
                actor.variables[1] = actor.variables[1].wrapping_sub(512);
                actor.variables[1]
            });
            if (velocity as i16) < 0 {
                for axis in 0..2 {
                    let position = runtime.native_word(extra + 4 + 2 * axis);
                    runtime.write_slice_native(extra + 8 + 2 * axis, position);
                    runtime.write_slice_native(base + 2 + 4 * axis, position);
                }
                runtime.edit_actor(key, Native, |actor| {
                    actor.variables[2] = 0;
                    actor.variables[3] =
                        u16::from(actor.parameters[0].to_ne_bytes()[0]).wrapping_sub(1);
                });
                animate(&mut runtime, 0xd840);
                runtime.edit_actor(key, Native, |actor| actor.variables[0] = 0xe0cd);
            }
        }
        0xe0cd => {
            let address = {
                let actor = runtime.resolve_actor(key, Native);
                actor.variables[4].wrapping_add(4 * u16::from(actor.variables[2].to_ne_bytes()[1]))
            };
            let row = runtime.legacy_native_record(0xa2, address);
            if row.word(0) == 0x8000 {
                let stride = runtime.resolve_actor(key, Native).pre_instruction;
                runtime.write_slice_native(
                    extra + 10,
                    runtime.native_word(extra + 10).wrapping_add(stride),
                );
                let finished = runtime.edit_actor(key, Native, |actor| {
                    actor.variables[2] = 0;
                    actor.variables[3] = actor.variables[3].wrapping_sub(1);
                    actor.variables[3] & 0x8000 != 0
                });
                if finished {
                    let velocity = runtime.native_word(extra + 12);
                    runtime.edit_actor(key, Native, |actor| actor.variables[1] = velocity);
                    for axis in 0..2 {
                        let position = runtime.native_word(extra + 2 * axis);
                        runtime.write_slice_native(base + 2 + 4 * axis, position);
                        runtime.write_slice_native(base + 4 + 4 * axis, 0);
                    }
                    animate(&mut runtime, 0xd82c);
                    runtime.edit_actor(key, Native, |actor| actor.variables[0] = 0xe035);
                }
            } else {
                for axis in 0..2 {
                    let delta = row.word(2 * axis);
                    let position = delta.wrapping_add(runtime.native_word(extra + 8 + 2 * axis));
                    runtime.write_slice_native(base + 2 + 4 * axis, position);
                }
                runtime.edit_actor(key, Native, |actor| {
                    actor.variables[2] = actor.variables[2].wrapping_add(256)
                });
            }
        }
        _ => runtime.unreachable(),
    }
}
