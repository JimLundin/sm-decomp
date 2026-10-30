//! Walking lava seahorse jumps, walking and fire commands.
//! Jump acceleration and home coordinates remain in native compatibility words.
#![forbid(unsafe_code)]
use super::access::{AccessMode::Native, ActorAccess, ActorKey as Key};
use super::api::{self, MovementResult::Blocked, SlopePolicy};
use crate::game::enemy_projectiles::{SpawnEprojWithGfx, host::Effects};
use crate::game::session_runtime::Runtime;
use crate::game_state::Game;
use crate::ram_map::words as W;
use crate::rtl::abs16;
use sm_game::enemy_script::EnemyScript;

fn selected(runtime: &Runtime<'_>) -> Key {
    Key::from_legacy(runtime.word(W::cur_enemy_index))
}
fn extra(key: Key) -> usize {
    0x7800 + usize::from(key.byte_index())
}
fn animate(runtime: &mut Runtime<'_>, key: Key, left: u16, right: u16) {
    runtime.edit_actor(key, Native, |actor| {
        actor.instruction = if actor.variables[3] & 0x8000 != 0 {
            left
        } else {
            right
        };
        actor.instruction_delay = 1;
    });
}
fn reset_jump(runtime: &mut Runtime<'_>, key: Key) {
    runtime.edit_actor(key, Native, |actor| {
        actor.variables[1] = (-12i16) as u16;
        actor.variables[2] = 0;
        actor.variables[3] = 0;
        actor.variables[4] = 0;
    });
    runtime.write_slice_native(extra(key), 0);
    runtime.write_slice_native(extra(key) + 2, 0x8000);
}
fn pair(runtime: &Runtime<'_>, at: usize) -> u32 {
    (u32::from(runtime.native_word(at)) << 16) | u32::from(runtime.native_word(at + 2))
}
fn add_pair(runtime: &mut Runtime<'_>, at: usize, amount: u32) -> u16 {
    let low = runtime.native_word(at + 2);
    let high = runtime.native_word(at);
    let next = ((u32::from(high) << 16) | u32::from(low)).wrapping_add(amount);
    runtime.write_slice_native(at + 2, next as u16);
    runtime.write_slice_native(at, (next >> 16) as u16);
    (next >> 16) as u16
}
fn velocity(runtime: &Runtime<'_>, key: Key) -> u32 {
    pair(runtime, 0xfaa + usize::from(key.byte_index()))
}
fn accelerate(runtime: &mut Runtime<'_>, key: Key) -> u16 {
    let amount = pair(runtime, extra(key));
    add_pair(runtime, 0xfaa + usize::from(key.byte_index()), amount)
}
fn drift(runtime: &mut Runtime<'_>, key: Key) {
    let amount = velocity(runtime, key);
    add_pair(runtime, 0xf7e + usize::from(key.byte_index()), amount);
}
fn player_offset(runtime: &Runtime<'_>, key: Key) -> u16 {
    runtime
        .word(W::samus_x_pos)
        .wrapping_sub(runtime.resolve_actor(key, Native).x.whole)
}
pub(crate) fn initialize(game: &mut Game) {
    let mut runtime = Runtime::new(game);
    let key = selected(&runtime);
    runtime.write_slice_native(extra(key) + 8, 0);
    runtime.edit_actor(key, Native, |actor| actor.pre_instruction = actor.y.whole);
    let x = runtime.resolve_actor(key, Native).x.whole;
    runtime.write_slice_native(extra(key) + 4, x);
    runtime.edit_actor(key, Native, |actor| {
        actor.properties |= 0x2000;
        actor.instruction = 0xdbe7;
        actor.variables[0] = 0xdd71;
    });
    let live = selected(&runtime);
    reset_jump(&mut runtime, live);
    loop {
        drift(&mut runtime, key);
        let live = selected(&runtime);
        if accelerate(&mut runtime, live) & 0x8000 == 0 {
            break;
        }
    }
    loop {
        let live = selected(&runtime);
        let amount = velocity(&runtime, key) as i32;
        if api::move_vertical(&mut runtime, live, amount) == Blocked {
            break;
        }
        let live = selected(&runtime);
        accelerate(&mut runtime, live);
    }
    let y = runtime.resolve_actor(key, Native).y.whole;
    runtime.write_slice_native(extra(key) + 6, y);
    runtime.edit_actor(key, Native, |actor| actor.y.whole = actor.pre_instruction);
}
pub(crate) fn advance(game: &mut Game) {
    let mut runtime = Runtime::new(game);
    let key = selected(&runtime);
    let phase = runtime.resolve_actor(key, Native).variables[0];
    match phase {
        0xdd71 => {
            let dy = runtime
                .native_word(extra(key) + 6)
                .wrapping_sub(runtime.word(W::samus_y_pos));
            if abs16(dy).wrapping_sub(32) & 0x8000 == 0 {
                return;
            }
            let dx = player_offset(&runtime, key);
            if (abs16(dx) as i16).wrapping_sub(0x50) >= 0 {
                return;
            }
            reset_jump(&mut runtime, key);
            runtime.edit_actor(key, Native, |actor| {
                actor.variables[3] = if dx & 0x8000 != 0 { (-2i16) as u16 } else { 2 }
            });
            animate(&mut runtime, key, 0xdc4b, 0xdcbb);
            runtime.edit_actor(key, Native, |actor| actor.variables[0] = 0xddc6);
            runtime.queue_sound(2, 6, 0x5e);
        }
        phase @ (0xddc6 | 0xdecd | 0xdeec) => {
            drift(&mut runtime, key);
            let actor = runtime.resolve_actor(key, Native);
            if phase == 0xdeec
                && (actor.y.whole as i16).wrapping_sub(actor.pre_instruction as i16) >= 0
            {
                runtime.edit_actor(key, Native, |actor| actor.y.whole = actor.pre_instruction);
                let x = runtime.native_word(extra(key) + 4);
                runtime.edit_actor(key, Native, |actor| {
                    actor.x.whole = x;
                    actor.variables[0] = 0xdd71;
                });
            } else if accelerate(&mut runtime, key) & 0x8000 == 0 && phase != 0xdeec {
                runtime.edit_actor(key, Native, |actor| {
                    actor.variables[0] = if phase == 0xddc6 { 0xde05 } else { 0xdeec }
                });
                if phase == 0xddc6 {
                    animate(&mut runtime, key, 0xdc51, 0xdcc1);
                }
            }
        }
        0xde05 => {
            let amount = velocity(&runtime, key) as i32;
            if api::move_vertical(&mut runtime, key, amount) == Blocked {
                let left = player_offset(&runtime, key) & 0x8000 != 0;
                runtime.edit_actor(key, Native, |actor| {
                    actor.variables[3] = if left { (-2i16) as u16 } else { 2 }
                });
                animate(&mut runtime, key, 0xdbe7, 0xdc57);
                runtime.edit_actor(key, Native, |actor| actor.variables[0] = 0xde4b);
                runtime.write_slice_native(extra(key) + 8, 1);
            } else {
                accelerate(&mut runtime, key);
            }
        }
        0xde4b => {
            api::move_vertical(&mut runtime, key, 2 << 16);
            let dx = runtime
                .native_word(extra(key) + 4)
                .wrapping_sub(runtime.resolve_actor(key, Native).x.whole);
            if (abs16(dx) as i16).wrapping_sub(0x70) >= 0 {
                runtime.edit_actor(key, Native, |actor| {
                    actor.variables[0] = 0xdecd;
                    actor.variables[1] = (-4i16) as u16;
                    actor.variables[2] = 0;
                });
                animate(&mut runtime, key, 0xdc51, 0xdcc1);
            } else if runtime.native_word(extra(key) + 8) == 0 {
                let left = player_offset(&runtime, key) & 0x8000 != 0;
                if left == (runtime.resolve_actor(key, Native).variables[3] & 0x8000 != 0) {
                    animate(&mut runtime, key, 0xdc03, 0xdc73);
                    runtime.edit_actor(key, Native, |actor| actor.variables[0] = 0xdecc);
                }
            }
        }
        0xdecc => {}
        _ => runtime.unreachable(),
    }
}
pub(crate) fn instruction(address: u32, script: EnemyScript, game: &mut Game) -> EnemyScript {
    let mut runtime = Runtime::new(game);
    let key = selected(&runtime);
    match address {
        11067164 | 11067187 | 11067193 => {
            let variant = match address {
                11067164 => 0,
                11067187 => 2,
                _ => 4,
            };
            runtime.foreign(|game| SpawnEprojWithGfx(variant, key.byte_index(), 0x9e90, game));
            runtime.queue_sound(2, 6, 0x3f);
        }
        11067199 => {
            runtime.edit_actor(key, Native, |actor| actor.variables[0] = 0xde4b);
            let count = runtime.word(W::random_number) & 3;
            runtime.write_slice_native(extra(key) + 8, if count == 0 { 2 } else { count });
            return EnemyScript::jump(
                if runtime.resolve_actor(key, Native).variables[3] & 0x8000 != 0 {
                    0xdbe7
                } else {
                    0xdc57
                },
            );
        }
        11067235 | 11067249 => {
            if address == 11067235 {
                let count = runtime.native_word(extra(key) + 8);
                if count != 0 {
                    runtime.write_slice_native(extra(key) + 8, count.wrapping_sub(1));
                }
            }
            let key = selected(&runtime);
            let amount = (runtime.resolve_actor(key, Native).variables[3] as i16 as i32) << 16;
            if api::move_horizontal(&mut runtime, key, amount, SlopePolicy::Ignore) == Blocked {
                let speed = runtime.resolve_actor(key, Native).variables[3] as i16;
                let next = if speed < 0 { 0xdc59 } else { 0xdbe9 };
                runtime.edit_actor(key, Native, |actor| {
                    actor.variables[3] = speed.wrapping_neg() as u16
                });
                return EnemyScript::jump(next);
            }
            let live = selected(&runtime);
            api::snap_to_slopes(&mut runtime, live);
        }
        _ => {
            runtime.unreachable();
            return EnemyScript::Stop;
        }
    }
    script
}
