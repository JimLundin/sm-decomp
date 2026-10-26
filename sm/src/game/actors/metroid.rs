//! Metroid pursuit, attachment, recoil and sprite partners share one lifecycle.
#![forbid(unsafe_code)]
use super::access::{AccessMode::Native, ActorAccess, ActorKey as Key};
use super::{api, combat, scheduling};
use crate::game::enemy_projectiles::host::Effects;
use crate::game::enemy_projectiles::{DropSource, scatter_drop};
use crate::game::enemy_script::EnemyScript;
use crate::game::player_projectiles::content::table;
use crate::game::session_access::SessionAccess;
use crate::game::session_runtime::{Runtime, player_command};
use crate::game::sprite_effect_runtime as sprites;
use crate::game_state::Game;
use crate::ram_map::{word_arrays, words};

fn extra(key: Key) -> usize {
    0x7800 + usize::from(key.byte_index())
}
fn animate(runtime: &mut Runtime<'_>, key: Key, attached: bool) {
    runtime.edit_actor(key, Native, |actor| {
        actor.instruction = if attached { 0xea25 } else { 0xe9cf };
        actor.instruction_delay = 1;
    });
}
fn pursue(target_y: u16, close: bool, runtime: &mut Runtime<'_>) {
    let key = combat::current(runtime);
    for axis in [1, 0] {
        let target = if axis == 1 {
            target_y
        } else {
            runtime.session().player.x
        };
        let velocity = runtime.edit_actor(key, Native, |actor| {
            let position = if axis == 1 {
                actor.y.whole
            } else {
                actor.x.whole
            };
            let pair = &mut actor.variables[axis * 2..axis * 2 + 2];
            let next = if close {
                let bytes = [
                    (target.wrapping_sub(position) >> 8) as u8,
                    (target as u8).wrapping_sub(position.to_ne_bytes()[0]),
                ];
                let mut high = (u16::from_ne_bytes(bytes) & 0xff00) >> 3;
                if high & 0x1000 != 0 {
                    high |= 0xe000;
                }
                u32::from(high) << 16
            } else {
                let acceleration = i32::from((position.wrapping_sub(target) as i16) >> 2) << 8;
                ((u32::from(pair[1]) << 16) | u32::from(pair[0])).wrapping_sub(acceleration as u32)
            };
            pair[0] = next as u16;
            pair[1] = (next >> 16) as u16;
            let high = pair[1] as i16;
            if high < -3 || high >= 3 {
                pair[1] = high.clamp(-3, 3) as u16;
                pair[0] = 0;
            }
            ((u32::from(pair[1]) << 16) | u32::from(pair[0])) as i32
        });
        let current = combat::current(runtime);
        let result = if axis == 1 {
            api::move_vertical(runtime, current, velocity)
        } else {
            api::move_horizontal(runtime, current, velocity, api::SlopePolicy::Ignore)
        };
        if result == api::MovementResult::Blocked {
            runtime.edit_actor(key, Native, |actor| {
                actor.variables[axis * 2] = 0;
                actor.variables[axis * 2 + 1] = 0;
            });
        }
    }
}
fn attach(target_y: u16, runtime: &mut Runtime<'_>) {
    let key = combat::current(runtime);
    let x = runtime.session().player.x;
    runtime.edit_actor(key, Native, |actor| {
        actor.x.whole = x;
        actor.y.whole = target_y;
        actor.variables[..4].fill(0);
    });
}
fn escape(_: u16, runtime: &mut Runtime<'_>) {
    let key = combat::current(runtime);
    let phase = runtime.resolve_actor(key, Native).variables[4] & 3;
    for axis in [0, 1] {
        let displacement = table(runtime, 0xa3ea3f, phase as isize + axis * 4);
        runtime.edit_actor(key, Native, |actor| {
            let position = if axis == 0 {
                &mut actor.x.whole
            } else {
                &mut actor.y.whole
            };
            *position = position.wrapping_add(displacement);
        });
    }
    let finished = runtime.edit_actor(key, Native, |actor| {
        actor.variables[..4].fill(0);
        let previous = actor.variables[4];
        actor.variables[4] = previous.wrapping_sub(1);
        if previous == 1 {
            actor.pre_instruction = 0;
        }
        previous == 1
    });
    if finished {
        animate(runtime, key, false);
    }
}
pub(crate) fn initialize(game: &mut Game) {
    let mut runtime = Runtime::new(game);
    let key = combat::current(&runtime);
    runtime.edit_actor(key, Native, |actor| actor.instruction = 0xe9cf);
    for (offset, selector) in [(0, 50), (2, 52)] {
        let (x, y, appearance) = {
            let actor = runtime.resolve_actor(key, Native);
            (actor.x.whole, actor.y.whole, actor.tile | actor.palette)
        };
        let handle =
            runtime.foreign(|game| sprites::create_sprite(x, y, selector, appearance, game));
        runtime.write_slice_native(extra(key) + offset, handle);
    }
    runtime.write_slice_native(extra(key) + 4, 0);
    runtime.probe("sprite.metroid.init");
}
pub(crate) fn frozen(game: &mut Game) {
    let mut runtime = Runtime::new(game);
    scheduling::update_frozen_enemy(&mut runtime);
    let key = combat::current(&runtime);
    runtime.edit_actor(key, Native, |actor| {
        if actor.variables[4] != 0 {
            actor.variables[4] = actor.variables[4].wrapping_sub(1);
            actor.flash = 2;
        }
    });
    for (offset, program) in [(0, 0xc3ba), (2, 0xc4b6)] {
        let handle = runtime.native_word(extra(key) + offset);
        runtime.foreign(|game| sprites::freeze_attachment(handle, 3072, program, game));
    }
    runtime.probe("sprite.metroid.frozen");
}
pub(crate) fn hurt(game: &mut Game) {
    let mut runtime = Runtime::new(game);
    let key = combat::current(&runtime);
    let appearance = {
        let actor = runtime.resolve_actor(key, Native);
        if actor.flash & 2 != 0 {
            actor.palette
        } else {
            0
        }
    };
    for offset in [0, 2] {
        let handle = runtime.native_word(extra(key) + offset);
        runtime.foreign(|game| sprites::flash_attachment(handle, appearance, game));
    }
    runtime.probe("sprite.metroid.hurt");
}
pub(crate) fn advance(game: &mut Game) {
    let mut runtime = Runtime::new(game);
    let key = combat::current(&runtime);
    let phase = runtime.resolve_actor(key, Native).pre_instruction;
    let behaviors: [fn(u16, &mut Runtime<'_>); 4] = [
        |y, runtime| pursue(y, false, runtime),
        |y, runtime| pursue(y, true, runtime),
        attach,
        escape,
    ];
    let behavior = behaviors[usize::from(phase)];
    let target = runtime.session().player.y.wrapping_sub(8);
    behavior(target, &mut runtime);
    let appearance = {
        let actor = runtime.resolve_actor(key, Native);
        actor.tile | actor.palette
    };
    for offset in [0, 2] {
        let handle = runtime.native_word(extra(key) + offset);
        runtime.foreign(|game| sprites::attach_actor(handle, key.byte_index(), appearance, game));
    }
    runtime.probe("sprite.metroid.main");
}
fn recoil(runtime: &mut Runtime<'_>, key: Key, contact: bool, target_y: u16) {
    runtime.edit_actor(key, Native, |actor| {
        actor.variables[0] = 0;
        actor.variables[2] = 0;
    });
    for axis in [0, 1] {
        let target = if contact {
            if axis == 0 {
                runtime.session().player.x
            } else {
                target_y
            }
        } else {
            runtime.word(if axis == 0 {
                word_arrays::projectile_x_pos
            } else {
                word_arrays::projectile_y_pos
            })
        };
        let difference = {
            let actor = runtime.resolve_actor(key, Native);
            (if axis == 0 {
                actor.x.whole
            } else {
                actor.y.whole
            })
            .wrapping_sub(target) as i16
        };
        runtime.edit_actor(key, Native, |actor| {
            actor.variables[axis * 2 + 1] = if difference < 0 { 0xff00 } else { 0 }
        });
        let middle = (difference as u16).wrapping_mul(if contact { 64 } else { 32 });
        runtime.write_slice_native(
            0xf78 + usize::from(key.byte_index()) + 0x31 + axis * 4,
            middle,
        );
    }
    runtime.edit_actor(key, Native, |actor| actor.pre_instruction = 0);
    animate(runtime, key, false);
}
fn drain(runtime: &mut Runtime<'_>, key: Key) {
    let amount = if runtime.word(words::equipped_items) & 0x20 != 0 {
        12288
    } else if runtime.word(words::equipped_items) & 1 != 0 {
        24576
    } else {
        49152
    };
    let at = extra(key) + 4;
    let previous = runtime.native_word(at);
    runtime.write_slice_native(at, previous.wrapping_sub(amount));
    if previous < amount {
        runtime.damage_player(1);
    }
}
pub(crate) fn touch(game: &mut Game) {
    let mut runtime = Runtime::new(game);
    let key = combat::current(&runtime);
    let target_y = runtime.session().player.y.wrapping_sub(8);
    if runtime.word(words::samus_contact_damage_index) != 0 {
        if runtime.resolve_actor(key, Native).pre_instruction != 2 {
            recoil(&mut runtime, key, true, target_y);
        }
    } else {
        let current = combat::current(&runtime);
        if runtime.resolve_actor(key, Native).pre_instruction != 3 {
            if runtime.word(words::random_enemy_counter) & 7 == 7
                && runtime.word(words::samus_health).wrapping_sub(30) & 0x8000 == 0
            {
                runtime.queue_sound(3, 6, 0x2d);
            }
            drain(&mut runtime, current);
        }
        if runtime.resolve_actor(key, Native).pre_instruction < 2 {
            let actor = runtime.resolve_actor(key, Native);
            let near =
                (actor.x.whole.wrapping_sub(runtime.session().player.x) as i16).unsigned_abs() < 8
                    && (actor.y.whole.wrapping_sub(target_y) as i16).unsigned_abs() < 8;
            let phase = if near { 2 } else { 1 };
            if near {
                runtime.foreign(|game| player_command(0x12, game));
            }
            runtime.edit_actor(key, Native, |actor| actor.pre_instruction = phase);
            if near {
                animate(&mut runtime, key, true);
            }
        }
    }
}
fn retire(runtime: &mut Runtime<'_>, captured: Option<Key>) -> u16 {
    let mut handle = 0;
    for offset in [0, 2] {
        let key = captured.unwrap_or_else(|| combat::current(runtime));
        handle = runtime.native_word(extra(key) + offset);
        runtime.foreign(|game| sprites::retire_native(handle, game));
    }
    handle
}
pub(crate) fn power_bomb(game: &mut Game) {
    let mut runtime = Runtime::new(game);
    let key = combat::current(&runtime);
    combat::power_bomb_reaction(&mut runtime, api::DeathHandling::Finalize);
    if runtime.resolve_actor(key, Native).health == 0 {
        runtime.foreign(|game| player_command(0x13, game));
        retire(&mut runtime, None);
    }
}
pub(crate) fn shot(game: &mut Game) {
    let mut runtime = Runtime::new(game);
    let raw = runtime
        .session()
        .room
        .collision_detection_index
        .wrapping_mul(2);
    let key = combat::current(&runtime);
    let projectile = word_arrays::projectile_type + usize::from(raw >> 1) * 2;
    if runtime.resolve_actor(key, Native).frozen != 0 {
        if matches!(runtime.word(projectile) & 0xf00, 0x100 | 0x200) {
            let x = runtime.resolve_actor(key, Native).x.whole;
            runtime.write_slice_le(words::special_death_item_drop_x_origin_pos, x);
            let y = runtime.resolve_actor(key, Native).y.whole;
            runtime.write_slice_le(words::special_death_item_drop_y_origin_pos, y);
            combat::shot_reaction(&mut runtime, api::ShotHandling::ReactionOnly);
            if runtime.resolve_actor(key, Native).health == 0 {
                runtime.edit_actor(key, Native, |actor| actor.variables[1] = 0);
                combat::die(&mut runtime, api::DeathKind::Normal { animation: 4 });
                runtime.foreign(|game| player_command(0x13, game));
                let handle = retire(&mut runtime, Some(key));
                runtime.foreign(|game| scatter_drop(DropSource::Metroid, handle, game));
            }
        }
    } else if runtime.resolve_actor(key, Native).pre_instruction == 2 {
        if runtime.word(projectile) & 0xf00 == 0x500 {
            runtime.edit_actor(key, Native, |actor| {
                actor.variables[4] = 4;
                actor.pre_instruction = 3;
            });
            animate(&mut runtime, key, false);
            runtime.foreign(|game| player_command(0x13, game));
        }
    } else {
        recoil(&mut runtime, key, false, 0);
        let index = usize::from(runtime.session().room.collision_detection_index);
        if runtime.word(word_arrays::projectile_type + 2 * index) & 2 != 0 {
            runtime.queue_sound(3, 6, 0xa);
            let damage = runtime.word(word_arrays::projectile_damage + 2 * index);
            let frozen = runtime.edit_actor(key, Native, |actor| {
                actor.flash = 4;
                let previous = actor.parameters[1];
                actor.parameters[1] = previous.saturating_sub(damage);
                if damage >= previous {
                    actor.frozen = 400;
                    actor.ai_handlers |= 4;
                    true
                } else {
                    false
                }
            });
            if frozen {
                return;
            }
        }
        runtime.queue_sound(2, 6, 0x5a);
    }
}
pub(crate) fn sound(random: bool, script: EnemyScript, game: &mut Game) -> EnemyScript {
    let mut runtime = Runtime::new(game);
    let sound = if random {
        let index = runtime.next_random() & 7;
        table(&runtime, 0xa3ead6, index as isize)
    } else {
        0x50
    };
    runtime.queue_sound(2, 6, sound);
    script
}
