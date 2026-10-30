//! Paired lava seahorses share emergence, firing, retreat and hit synchronization.
#![forbid(unsafe_code)]
use crate::game::actors::api;
use crate::game::session_runtime::Runtime;
use crate::game_state::Game;
use crate::ram_map::words as W;
use sm_game::actors::access::{AccessMode::Native, ActorAccess, ActorKey as Key};
use sm_game::enemy_script::EnemyScript;
use {
    crate::game::enemy_projectiles::SpawnEprojWithGfx, sm_game::enemy_projectiles::host::Effects,
};

fn animate(runtime: &mut Runtime<'_>, child: bool) {
    let raw = usize::from(runtime.word(W::cur_enemy_index)) + if child { 64 } else { 0 };
    crate::game::actors::restart_from_native_cache(runtime, raw, 0xa2e5ef);
}
fn tick(runtime: &mut Runtime<'_>, key: Key) -> u16 {
    runtime.edit_actor(key, Native, |actor| {
        let previous = actor.variables[3];
        actor.variables[3] = previous.wrapping_sub(1);
        previous
    })
}
pub(crate) fn initialize(game: &mut Game) {
    let mut runtime = Runtime::new(game);
    let raw = runtime.word(W::cur_enemy_index);
    let key = Key::from_legacy(raw);
    let extra = 0x7800 + usize::from(raw);
    runtime.write_slice_native(extra + 4, 0);
    let child = runtime.resolve_actor(key, Native).parameters[0] != 0;
    for at in [extra, extra + 2] {
        runtime.write_slice_native(at, if child { 2 } else { 0 });
    }
    runtime.edit_actor(key, Native, |actor| {
        actor.instruction = if child { 0xe5a1 } else { 0xe59b };
        if child {
            actor.properties |= 0x400;
        }
        actor.pre_instruction = if child { 0xe781 } else { 0xe654 };
    });
}
pub(crate) fn pre_instruction(game: &mut Game) -> u32 {
    let runtime = Runtime::new(game);
    let key = Key::from_legacy(runtime.word(W::cur_enemy_index));
    0xa20000 | u32::from(runtime.resolve_actor(key, Native).pre_instruction)
}
pub(crate) fn fire_ready(cursor: EnemyScript, game: &mut Game) -> EnemyScript {
    let mut runtime = Runtime::new(game);
    let at = 0x7804 + usize::from(runtime.word(W::cur_enemy_index));
    runtime.write_slice_native(at, 1);
    cursor
}
pub(crate) fn synchronize(game: &mut Game) {
    let mut runtime = Runtime::new(game);
    let base = 0xf78 + usize::from(runtime.word(W::cur_enemy_index));
    // Child offsets stay wide; copy shake, invincibility, flash, frozen, then handlers.
    if runtime.native_word(base + 0x14) != 0 {
        for field in [0x2a, 0x28, 0x24, 0x26, 0x12] {
            let value = runtime.native_word(base + field);
            runtime.write_slice_native(base + 64 + field, value);
        }
    } else {
        let properties = runtime.native_word(base + 64 + 0xe) | 0x200;
        runtime.write_slice_native(base + 64 + 0xe, properties);
    }
}
pub(crate) fn phase(k: u16, phase: u16, game: &mut Game) {
    let mut runtime = Runtime::new(game);
    let key = Key::from_legacy(k);
    let extra = 0x7800 + usize::from(k);
    match phase {
        0xe654 | 0xe6ad | 0xe749 => {
            if tick(&mut runtime, key).wrapping_sub(1) & 0x8000 != 0 {
                let (delay, next) = match phase {
                    0xe654 => (48, 0xe6ad),
                    0xe6ad => (3, 0xe6f1),
                    _ => (128, 0xe654),
                };
                if phase == 0xe6ad {
                    let variant = runtime.native_word(extra).wrapping_add(4);
                    runtime.write_slice_native(extra, variant);
                }
                runtime.edit_actor(key, Native, |actor| {
                    actor.variables[3] = delay;
                    actor.pre_instruction = next;
                });
                if phase == 0xe654 {
                    let delta = api::player_delta(&runtime, key, api::Axis::X);
                    let left = runtime.edit_actor(key, Native, |actor| {
                        actor.variables[0] = actor.variables[0] & 0x7fff | delta & 0x8000;
                        actor.variables[0] & 0x8000 != 0
                    });
                    for at in [extra, extra + 64] {
                        let flags = runtime.byte(at);
                        runtime.set_byte(at, if left { flags & !1 } else { flags | 1 });
                    }
                    animate(&mut runtime, false);
                    animate(&mut runtime, true);
                }
            }
            if phase != 0xe654 {
                for raw in [usize::from(k), usize::from(k) + 64] {
                    let at = 0xf7e + raw;
                    let delta = if phase == 0xe6ad { u16::MAX } else { 1 };
                    runtime.write_slice_native(at, runtime.native_word(at).wrapping_add(delta));
                }
            }
        }
        0xe6f1 => {
            animate(&mut runtime, false);
            if runtime.native_word(extra + 4) != 0 {
                runtime.write_slice_native(extra + 4, 0);
                runtime.write_slice_native(extra + 2, u16::MAX);
                runtime.foreign(|game| SpawnEprojWithGfx(0xffff, k, 0xb5cb, game));
                runtime.queue_sound(2, 6, 0x61);
                if tick(&mut runtime, key) == 1 {
                    let variant = runtime.native_word(extra).wrapping_sub(4);
                    runtime.write_slice_native(extra, variant);
                    runtime.edit_actor(key, Native, |actor| {
                        actor.variables[3] = 96;
                        actor.pre_instruction = 0xe734;
                    });
                }
            }
        }
        0xe734 => {
            if tick(&mut runtime, key) == 1 {
                runtime.edit_actor(key, Native, |actor| {
                    actor.variables[3] = 48;
                    actor.pre_instruction = 0xe749;
                });
                animate(&mut runtime, false);
            }
        }
        _ => unreachable!("unknown paired seahorse phase"),
    }
}
