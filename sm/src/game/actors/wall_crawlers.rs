//! Shared wall/floor crawling, edge turns and falling for the A3 crawler family.
#![forbid(unsafe_code)]
use super::access::{AccessMode::Native, ActorAccess, ActorKey as Key};
use super::{api, combat};
use crate::game::session_runtime::Runtime;
use crate::game_state::Game;
use crate::ram_map::word_arrays;
use sm_game::enemy_script::EnemyScript;
use sm_game::geometry::Point;
use sm_game::player_projectiles::content::{self, table};

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Kind {
    Crab,
    Slug,
    Fireball,
    Common,
    Orange,
}
fn extra(key: Key) -> usize {
    0x7800 + usize::from(key.byte_index())
}
fn prepare_speed(runtime: &mut Runtime<'_>, key: Key, common: bool) {
    if common {
        runtime.edit_actor(key, Native, |actor| {
            actor.spritemap = 0x804d;
            actor.instruction_delay = 1;
            actor.pre_instruction = 0xe6c1;
        });
    }
    let parameter = runtime.resolve_actor(key, Native).parameters[0];
    if !common || parameter != 255 {
        let speed = table(runtime, 0xa3e5f0, parameter as isize);
        runtime.edit_actor(key, Native, |actor| {
            actor.variables[0] = speed;
            actor.variables[1] = speed;
        });
    }
    runtime.edit_actor(key, Native, |actor| match actor.properties & 3 {
        0 => actor.variables[0] = actor.variables[0].wrapping_neg(),
        2 => actor.variables[1] = actor.variables[1].wrapping_neg(),
        _ => (),
    });
}
pub(crate) fn prepare(raw: u16, game: &mut Game) {
    prepare_speed(&mut Runtime::new(game), Key::from_legacy(raw), true);
}
pub(crate) fn initialize(kind: Kind, game: &mut Game) {
    let mut runtime = Runtime::new(game);
    let key = combat::current(&runtime);
    let (address, parameter) = match kind {
        Kind::Crab => (0xa396db, Some(8)),
        Kind::Slug => (0xa3992b, Some(10)),
        Kind::Fireball => (0xa3b667, Some(6)),
        Kind::Common => (0xa3e2cc, None),
        Kind::Orange => (0xa3e03b, None),
    };
    if let Some(value) = parameter {
        runtime.edit_actor(key, Native, |actor| actor.parameters[1] = value);
    }
    let index = {
        let actor = runtime.resolve_actor(key, Native);
        (if kind == Kind::Fireball {
            actor.properties
        } else {
            actor.instruction
        }) & 3
    };
    let instruction = table(&runtime, address, index as isize);
    runtime.edit_actor(key, Native, |actor| actor.instruction = instruction);
    let key = if kind == Kind::Orange {
        runtime.edit_actor(key, Native, |actor| actor.pre_instruction = 0xe08a);
        key
    } else {
        combat::current(&runtime)
    };
    prepare_speed(&mut runtime, key, kind != Kind::Orange);
}
pub(crate) fn state_address(game: &mut Game) -> u32 {
    let runtime = Runtime::new(game);
    let key = combat::current(&runtime);
    u32::from(runtime.resolve_actor(key, Native).pre_instruction) | 0xa30000
}
pub(crate) fn set_state(raw: u16, script: EnemyScript, game: &mut Game) -> EnemyScript {
    let next = script.word(0);
    Runtime::new(game).edit_actor(Key::from_legacy(raw), Native, |actor| {
        actor.pre_instruction = next
    });
    script.offset(1)
}
fn fall(runtime: &mut Runtime<'_>, key: Key) {
    let phase = runtime.resolve_actor(key, Native).pre_instruction;
    runtime.write_slice_native(extra(key) + 6, phase);
    runtime.edit_actor(key, Native, |actor| actor.pre_instruction = 0xe785);
}
fn move_axis(runtime: &mut Runtime<'_>, key: Key, axis: usize, amount: i32, orange: bool) -> bool {
    let result = if axis == 0 {
        let slopes = if orange {
            api::SlopePolicy::Follow
        } else {
            api::SlopePolicy::Ignore
        };
        api::move_horizontal(runtime, key, amount, slopes)
    } else {
        api::move_vertical(runtime, key, amount)
    };
    result == api::MovementResult::Blocked
}
fn animate(runtime: &mut Runtime<'_>, key: Key, axis: usize, orange: bool) {
    let actor = runtime.resolve_actor(key, Native);
    let negative = actor.variables[1 - axis] & 0x8000 != 0;
    let instruction = if orange {
        match (axis, negative) {
            (0, false) => 0xe01f,
            (0, true) => 0xe003,
            (_, false) => 0xdfe7,
            (_, true) => 0xdfcb,
        }
    } else {
        let address = match (axis, negative) {
            (0, false) => 0xa3e63c,
            (0, true) => 0xa3e630,
            (_, false) => 0xa3e654,
            (_, true) => 0xa3e648,
        };
        table(runtime, address, (actor.parameters[1] >> 1) as isize)
    };
    runtime.edit_actor(key, Native, |actor| {
        actor.instruction = instruction;
        actor.instruction_delay = 1;
    });
}
fn slope_velocity(runtime: &mut Runtime<'_>) -> i32 {
    let key = combat::current(runtime);
    let point = {
        let actor = runtime.resolve_actor(key, Native);
        let y = if actor.variables[1] & 0x8000 != 0 {
            actor.y.whole.wrapping_sub(actor.radius.y)
        } else {
            actor.y.whole.wrapping_add(actor.radius.y).wrapping_sub(1)
        };
        Point {
            x: actor.x.whole,
            y,
        }
    };
    api::select_block_at(runtime, point);
    let index = usize::from(runtime.session().room.collision_cursor);
    if runtime.word(word_arrays::level_data + 2 * index) & 0xf000 == 0x1000
        && runtime.byte(0x16402 + usize::from(runtime.session().room.collision_cursor)) & 31 >= 5
    {
        let image = content::image(runtime);
        let behavior = runtime.byte(0x16402 + usize::from(runtime.session().room.collision_cursor));
        let scale = image.table_word(
            content::offset(0xa3e931),
            ((4 * (u16::from(behavior) & 31)) >> 1) as isize,
        );
        let speed = runtime.resolve_actor(key, Native).variables[0];
        let magnitude = if speed & 0x8000 != 0 {
            speed.wrapping_neg()
        } else {
            speed
        };
        let product = runtime.foreign(|game| game.multiply_with_scratch(magnitude, scale)) as i32;
        if speed & 0x8000 != 0 {
            -product
        } else {
            product
        }
    } else {
        i32::from(runtime.resolve_actor(key, Native).variables[0] as i16) << 8
    }
}
pub(crate) fn crawl(raw: u16, axis: api::Axis, orange: bool, game: &mut Game) {
    let axis = usize::from(axis == api::Axis::Y);
    let mut runtime = Runtime::new(game);
    let key = Key::from_legacy(raw);
    let at = extra(key);
    if runtime.session().feedback.earthquake_timer == 30
        && runtime.session().feedback.earthquake_kind == 20
    {
        fall(&mut runtime, key);
    }
    let speed = i32::from(runtime.resolve_actor(key, Native).variables[axis] as i16) << 8;
    let leading = speed + if speed < 0 { -0x10000 } else { 0x10000 };
    if move_axis(&mut runtime, key, axis, leading, orange) {
        runtime.write_slice_native(at + 8, 0);
        if axis == 0 {
            api::snap_to_slopes(&mut runtime, key);
        }
        let amount = if axis == 1 && !orange {
            slope_velocity(&mut runtime)
        } else {
            i32::from(runtime.resolve_actor(key, Native).variables[1 - axis] as i16) << 8
        };
        if move_axis(&mut runtime, key, 1 - axis, amount, orange) {
            runtime.edit_actor(key, Native, |actor| {
                actor.variables[axis] = actor.variables[axis].wrapping_neg()
            });
            animate(&mut runtime, key, axis, orange);
        } else {
            if axis == 1 {
                api::snap_to_slopes(&mut runtime, key);
            }
            if orange {
                let actor = runtime.resolve_actor(key, Native);
                let (position, player) = if axis == 0 {
                    (actor.y.whole, runtime.session().player.y)
                } else {
                    (actor.x.whole, runtime.session().player.x)
                };
                let positive = player.wrapping_sub(position) & 0x8000 == 0;
                runtime.edit_actor(key, Native, |actor| {
                    let speed = &mut actor.variables[1 - axis];
                    if positive == (*speed & 0x8000 != 0) {
                        *speed = speed.wrapping_neg();
                    }
                });
            }
        }
    } else {
        let edges = runtime.native_word(at + 8).wrapping_add(1);
        runtime.write_slice_native(at + 8, edges);
        if edges.wrapping_sub(4) & 0x8000 != 0 {
            runtime.edit_actor(key, Native, |actor| {
                actor.variables[1 - axis] = actor.variables[1 - axis].wrapping_neg()
            });
            animate(&mut runtime, key, axis, orange);
        } else {
            fall(&mut runtime, key);
        }
    }
}
pub(crate) fn falling(raw: u16, game: &mut Game) {
    let mut runtime = Runtime::new(game);
    let key = Key::from_legacy(raw);
    let at = extra(key);
    let velocity =
        (u32::from(runtime.native_word(at + 4)) << 16) | u32::from(runtime.native_word(at + 2));
    if api::move_vertical(&mut runtime, key, velocity as i32) == api::MovementResult::Blocked {
        runtime.edit_actor(key, Native, |actor| {
            if actor.parameters[0] == 255 {
                actor.variables[0] = 128;
                actor.variables[1] = 128;
            }
        });
        for offset in [2, 4, 8] {
            runtime.write_slice_native(at + offset, 0);
        }
        let phase = runtime.native_word(at + 6);
        runtime.edit_actor(key, Native, |actor| actor.pre_instruction = phase);
    } else {
        if runtime.native_word(at + 4).wrapping_sub(4) & 0x8000 != 0 {
            let low = runtime.native_word(at + 2);
            let high = runtime.native_word(at + 4);
            let next = ((u32::from(high) << 16) | u32::from(low)).wrapping_add(0x8000);
            runtime.write_slice_native(at + 2, next as u16);
            runtime.write_slice_native(at + 4, (next >> 16) as u16);
        }
        if runtime.native_word(at + 2) == 0 && runtime.native_word(at + 4) == 0 {
            runtime.edit_actor(key, Native, |actor| actor.pre_instruction = 0xe6c8);
        }
    }
}
