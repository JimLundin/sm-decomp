//! Floor and ceiling Sidehoppers share launch selection and ordered hop movement.
#![forbid(unsafe_code)]
use super::access::{AccessMode::Native, ActorAccess, ActorKey as Key};
use super::{api, combat};
use crate::game::enemy_speeds::SpeedTable;
use crate::game::session_runtime::Runtime;
use crate::game_state::Game;
use sm_game::enemy_projectiles::host::Effects;
use sm_game::enemy_script::EnemyScript;
use sm_game::player_projectiles::content::table;
use sm_game::player_projectiles::host::Host;
use sm_game::session_access::SessionAccess;

fn extra(key: Key) -> usize {
    0x7800 + usize::from(key.byte_index())
}
fn animate(runtime: &mut Runtime<'_>) {
    let key = combat::current(runtime);
    let instruction = runtime.native_word(extra(key));
    runtime.edit_actor(key, Native, |actor| actor.restart_animation(instruction));
}
fn definition_kind(runtime: &mut Runtime<'_>, key: Key) -> u16 {
    let definition = runtime.resolve_actor(key, Native).definition;
    runtime.record(0xa0, definition).word(0x2a)
}
pub(crate) fn initialize(game: &mut Game) {
    let mut runtime = Runtime::new(game);
    runtime.write_slice_le(crate::ram_map::words::random_number, 37);
    runtime.next_random();
    let key = combat::current(&runtime);
    let at = extra(key);
    for offset in [6, 8, 10] {
        runtime.write_slice_native(at + offset, 0);
    }
    let kind = definition_kind(&mut runtime, key).wrapping_mul(2);
    runtime.write_slice_native(at + 12, kind);
    let address = if runtime.resolve_actor(key, Native).parameters[0] != 0 {
        0xa3aaca
    } else {
        0xa3aac2
    };
    let instruction = table(
        &runtime,
        address,
        (runtime.native_word(at + 12) >> 1) as isize,
    );
    runtime.write_slice_native(at, instruction);
    animate(&mut runtime);
    if definition_kind(&mut runtime, key) != 0 {
        runtime.write_slice_native(at + 10, 2);
    }
    for (offset, height, step) in [(2, 0xa3aaee, 0xa3aae6), (4, 0xa3aafa, 0xa3aaf2)] {
        let index = (runtime.native_word(at + 10) >> 1) as isize;
        let height = table(&runtime, height, index);
        let step = table(&runtime, step, index);
        let phase =
            crate::game::hopping_blobs::initial_hop_phase(height, step, runtime.native_image());
        runtime.write_slice_native(at + offset, phase);
    }
    runtime.edit_actor(key, Native, |actor| actor.variables[1] = 0xabd6);
}
fn jump(runtime: &mut Runtime<'_>, falling: bool, ceiling: bool) {
    runtime.probe(
        [
            ["hop.Sidehopper_Func_19", "hop.Sidehopper_Func_20"],
            ["hop.Sidehopper_Func_21", "hop.Sidehopper_Func_22"],
        ][usize::from(ceiling)][usize::from(falling)],
    );
    let key = combat::current(runtime);
    let at = extra(key);
    let index = 4 * runtime.resolve_actor(key, Native).variables[2] as isize
        + if falling == ceiling { 2 } else { 0 };
    let current = combat::current(runtime);
    let high = runtime.enemy_speed(SpeedTable::Quadratic, index + 1);
    let low = runtime.enemy_speed(SpeedTable::Quadratic, index);
    let amount = ((u32::from(high) << 16) | u32::from(low)) as i32;
    if api::move_vertical(runtime, current, amount) == api::MovementResult::Blocked {
        if falling {
            runtime.write_slice_native(at + 6, 0);
            runtime.edit_actor(key, Native, |actor| actor.variables[1] = 0xace4);
        } else {
            runtime.edit_actor(key, Native, |actor| {
                actor.variables[3] = actor.variables[3].wrapping_neg()
            });
            runtime.write_slice_native(at + 6, 1);
        }
        return;
    }
    let current = combat::current(runtime);
    let amount = i32::from(runtime.resolve_actor(key, Native).variables[3] as i16) << 16;
    let blocked = api::move_horizontal(runtime, current, amount, api::SlopePolicy::Ignore)
        == api::MovementResult::Blocked;
    if blocked {
        runtime.edit_actor(key, Native, |actor| {
            actor.variables[3] = actor.variables[3].wrapping_neg()
        });
    }
    if falling {
        runtime.edit_actor(key, Native, |actor| {
            let next = actor.variables[4].wrapping_add(actor.variables[2]);
            actor.variables[2] = if next.wrapping_sub(64) & 0x8000 == 0 {
                64
            } else {
                next
            };
        });
    } else if blocked {
        runtime.write_slice_native(at + 6, 1);
    } else {
        let next = runtime.edit_actor(key, Native, |actor| {
            actor.variables[2] = actor.variables[2].wrapping_sub(actor.variables[4]);
            actor.variables[2]
        });
        if next & 0x8000 != 0 {
            runtime.write_slice_native(at + 6, 1);
            runtime.edit_actor(key, Native, |actor| actor.variables[2] = 0);
        }
    }
}
pub(crate) fn advance(game: &mut Game) {
    let mut runtime = Runtime::new(game);
    let key = combat::current(&runtime);
    let at = extra(key);
    let phase = runtime.resolve_actor(key, Native).variables[1];
    match phase {
        0xabd6 => {
            let image = runtime.loaded_image().expect("ROM must be loaded");
            let index = (runtime.next_random() & 1) as isize;
            let next = image.table_word(crate::rom::lorom_offset(0xa3aae2) as usize, index);
            runtime.edit_actor(key, Native, |actor| actor.variables[1] = next);
        }
        0xabe6 | 0xac13 => {
            let second = phase == 0xac13;
            let index = (runtime.native_word(at + 10) >> 1) as isize;
            let base = if second { 0xa3aaf2 } else { 0xa3aae6 };
            let step = table(&runtime, base, index);
            runtime.edit_actor(key, Native, |actor| actor.variables[4] = step);
            let speed = table(&runtime, base + 4, index);
            runtime.edit_actor(key, Native, |actor| actor.variables[3] = speed);
            let initial = runtime.native_word(at + if second { 4 } else { 2 });
            runtime.edit_actor(key, Native, |actor| {
                actor.variables[2] = initial;
                actor.variables[1] = if actor.parameters[0] != 0 {
                    0xac56
                } else {
                    0xac40
                };
            });
        }
        0xac40 | 0xac56 => {
            let next = if phase == 0xac40 { 0xac6c } else { 0xaca8 };
            runtime.edit_actor(key, Native, |actor| actor.variables[1] = next);
            if api::player_delta(&runtime, combat::current(&runtime), api::Axis::X) & 0x8000 == 0 {
                runtime.edit_actor(key, Native, |actor| actor.variables[1] = next + 0x23);
            }
        }
        0xac6c | 0xac8f | 0xaca8 | 0xaccb => {
            let ceiling = phase >= 0xaca8;
            let reverse = matches!(phase, 0xac6c | 0xaca8);
            if reverse {
                runtime.edit_actor(key, Native, |actor| {
                    actor.variables[3] = actor.variables[3].wrapping_neg()
                });
            }
            let base = if ceiling { 0xa3aada } else { 0xa3aad2 };
            let instruction = table(&runtime, base, (runtime.native_word(at + 12) >> 1) as isize);
            runtime.write_slice_native(at, instruction);
            animate(&mut runtime);
            runtime.edit_actor(key, Native, |actor| {
                actor.variables[1] =
                    if ceiling { 0xad32 } else { 0xad0e } + if reverse { 0 } else { 0x12 };
            });
        }
        0xace4 => {
            let instruction = table(
                &runtime,
                0xa3aac2,
                (runtime.native_word(at + 12) >> 1) as isize,
            );
            runtime.write_slice_native(at, instruction);
            if runtime.resolve_actor(key, Native).parameters[0] != 0 {
                let instruction = table(
                    &runtime,
                    0xa3aaca,
                    (runtime.native_word(at + 12) >> 1) as isize,
                );
                runtime.write_slice_native(at, instruction);
            }
            animate(&mut runtime);
            runtime.edit_actor(key, Native, |actor| actor.variables[1] = 0xad56);
        }
        0xad0e | 0xad20 | 0xad32 | 0xad44 => {
            let falling = runtime.native_word(at + 6) != 0;
            jump(&mut runtime, falling, phase >= 0xad32);
        }
        0xad56 => {
            if runtime.native_word(at + 8) != 0 {
                runtime.write_slice_native(at + 8, 0);
                runtime.edit_actor(key, Native, |actor| actor.variables[1] = 0xabd6);
            }
        }
        _ => runtime.unreachable(),
    }
}
pub(crate) fn instruction(sound: bool, script: EnemyScript, game: &mut Game) -> EnemyScript {
    let mut runtime = Runtime::new(game);
    if sound {
        runtime.queue_sound(2, 3, script.word(0));
        script.offset(1)
    } else {
        let at = extra(combat::current(&runtime));
        runtime.write_slice_native(at + 8, 1);
        script
    }
}
