//! Rinka spawning, flight and recycling through the shared actor codec.
//! Boss counters and spawn occupancy remain RAM-authoritative during migration.
#![forbid(unsafe_code)]
use crate::game::actors::api::{DeathHandling, DeathKind, ShotHandling, angle_from_delta};
use crate::game::enemy_projectiles::{Math_MultByCos, Math_MultBySin, SpawnEprojWithRoomGfx};
use crate::game::session_runtime::Runtime;
use crate::game_state::Game;
use sm_game::geometry::Position;
use {
    crate::game::actors::{self},
    sm_game::actors::access::{AccessMode::Native, ActorAccess, ActorKey as Key},
};

const INVISIBLE: u16 = 0x100;
const TANGIBLE: u16 = 0x400;
const BOSS_FINISHED: usize = 0x783a;
const ACTIVE_COUNT: usize = 0x783c;

fn spawn_word(runtime: &Runtime<'_>, index: usize) -> u16 {
    runtime
        .loaded_image()
        .expect("ROM must be loaded")
        .table_word(crate::rtl::lorom_offset(0xa2b75b) as usize, index as isize)
}

fn occupied(runtime: &Runtime<'_>, id: u16) -> bool {
    runtime.native_word(0x87fe + usize::from(id)) & 1 != 0
}

fn outside_spawn_view(runtime: &Runtime<'_>, x: u16, y: u16) -> bool {
    // The original checks the y sign twice, including before the horizontal test.
    y & 0x8000 != 0
        || (y.wrapping_sub(runtime.word(0x915)) as i16) < 0
        || y.wrapping_sub(runtime.word(0x915)).wrapping_sub(224) & 0x8000 == 0
        || y & 0x8000 != 0
        || (x.wrapping_sub(runtime.word(0x911)) as i16) < 0
        || x.wrapping_sub(runtime.word(0x911)).wrapping_sub(256) & 0x8000 == 0
}

fn saved_spawn(runtime: &Runtime<'_>, index: u16) -> u16 {
    let saved = 0x7000 + usize::from(index);
    for row in (0..33).step_by(3) {
        if spawn_word(runtime, row) == runtime.native_word(saved + 0x20)
            && spawn_word(runtime, row + 1) == runtime.native_word(saved + 0x22)
        {
            return spawn_word(runtime, row + 2);
        }
    }
    spawn_word(runtime, 2)
}

fn select_spawn(runtime: &mut Runtime<'_>, index: u16) {
    let key = Key::from_legacy(index);
    let saved = 0x7000 + usize::from(index);
    if !outside_spawn_view(
        runtime,
        runtime.native_word(saved + 0x20),
        runtime.native_word(saved + 0x22),
    ) {
        let id = saved_spawn(runtime, index);
        if !occupied(runtime, id) {
            runtime.edit_actor(key, Native, |actor| actor.variables[3] = id);
            runtime.write_slice_native(0x87fe + usize::from(id), u16::MAX);
            return;
        }
    }
    for row in (0..33).step_by(3) {
        let x = spawn_word(runtime, row);
        let y = spawn_word(runtime, row + 1);
        if !outside_spawn_view(runtime, x, y) && !occupied(runtime, spawn_word(runtime, row + 2)) {
            runtime.write_slice_native(saved + 0x20, x);
            runtime.edit_actor(key, Native, |actor| actor.x.whole = x);
            runtime.write_slice_native(saved + 0x22, y);
            runtime.edit_actor(key, Native, |actor| actor.y.whole = y);
            let id = spawn_word(runtime, row + 2);
            // The visible choice publishes occupancy before the actor's id.
            runtime.write_slice_native(0x87fe + usize::from(id), u16::MAX);
            runtime.edit_actor(key, Native, |actor| actor.variables[3] = id);
            return;
        }
    }
    for row in (0..33).step_by(3) {
        if !occupied(runtime, spawn_word(runtime, row + 2)) {
            let x = spawn_word(runtime, row);
            runtime.write_slice_native(saved + 0x20, x);
            runtime.edit_actor(key, Native, |actor| actor.x.whole = x);
            let y = spawn_word(runtime, row + 1);
            runtime.write_slice_native(saved + 0x22, y);
            runtime.edit_actor(key, Native, |actor| actor.y.whole = y);
            let id = spawn_word(runtime, row + 2);
            runtime.edit_actor(key, Native, |actor| actor.variables[3] = id);
            runtime.write_slice_native(0x87fe + usize::from(id), u16::MAX);
            return;
        }
    }
}

fn start_animation(runtime: &mut Runtime<'_>, key: Key) {
    runtime.edit_actor(key, Native, |actor| {
        actor.variables[0] = 0xb852;
        actor.pre_instruction = 26;
        actor.variables[1] = 0;
        actor.variables[2] = 0;
    });
    let boss = runtime.resolve_actor(key, Native).parameters[0] != 0;
    if boss && runtime.native_word(BOSS_FINISHED) != 0 {
        runtime.edit_actor(key, Native, |actor| actor.properties |= 0x200);
    } else {
        runtime.edit_actor(key, Native, |actor| {
            actor.instruction = if boss { 0xba0c } else { 0xb9e0 };
            actor.instruction_delay = 1;
            actor.timer = 0;
        });
    }
}

fn respawn(runtime: &mut Runtime<'_>, index: u16) {
    let key = Key::from_legacy(index);
    if runtime.resolve_actor(key, Native).parameters[0] != 0 {
        select_spawn(runtime, index);
    }
    let saved = 0x7000 + usize::from(index);
    let x = runtime.native_word(saved + 0x20);
    runtime.edit_actor(key, Native, |actor| actor.x.whole = x);
    let y = runtime.native_word(saved + 0x22);
    runtime.edit_actor(key, Native, |actor| actor.y.whole = y);
    start_animation(runtime, key);
}

fn decrement_active(runtime: &mut Runtime<'_>, key: Key) {
    if runtime.resolve_actor(key, Native).parameters[0] != 0
        && runtime.resolve_actor(key, Native).properties & INVISIBLE == 0
    {
        let count = runtime.native_word(ACTIVE_COUNT).wrapping_sub(1) as i16;
        runtime.write_slice_native(ACTIVE_COUNT, count.max(0) as u16);
    }
}

fn release_spawn(runtime: &mut Runtime<'_>, key: Key) {
    if runtime.resolve_actor(key, Native).parameters[0] != 0 {
        let id = runtime.resolve_actor(key, Native).variables[3];
        if id != 0 {
            runtime.write_slice_native(0x87fe + usize::from(id), 0);
            runtime.edit_actor(key, Native, |actor| actor.variables[3] = 0);
        }
    }
}

fn outside_flight_view(runtime: &Runtime<'_>, key: Key) -> bool {
    let y = runtime.resolve_actor(key, Native).y.whole;
    if y & 0x8000 == 0 {
        let relative_y = y.wrapping_add(16).wrapping_sub(runtime.word(0x915));
        if relative_y & 0x8000 == 0 && relative_y.wrapping_sub(256) & 0x8000 != 0 {
            let x = runtime.resolve_actor(key, Native).x.whole;
            if x & 0x8000 == 0 {
                let relative_x = x.wrapping_add(16).wrapping_sub(runtime.word(0x911));
                if relative_x & 0x8000 == 0 && relative_x.wrapping_sub(288) & 0x8000 != 0 {
                    return false;
                }
            }
        }
    }
    true
}

fn recycle(runtime: &mut Runtime<'_>, index: u16) {
    let key = Key::from_legacy(index);
    let mut delete = false;
    if runtime.resolve_actor(key, Native).parameters[0] != 0 {
        release_spawn(runtime, key);
        delete = runtime.native_word(BOSS_FINISHED) != 0;
    }
    decrement_active(runtime, key);
    if delete {
        actors::population::delete_connected(runtime);
    } else {
        respawn(runtime, index);
    }
}

fn advance_native(position: &mut Position, velocity: u16) {
    // Legacy movement uses physical byte lanes even on big-endian hosts.
    let mut fraction = position.fraction.to_ne_bytes();
    let carry = u16::from(fraction[1]) + u16::from(velocity.to_ne_bytes()[0]);
    fraction[1] = carry as u8;
    position.fraction = u16::from_ne_bytes(fraction);
    position.whole = position
        .whole
        .wrapping_add_signed(i16::from((velocity >> 8) as i8))
        .wrapping_add(carry >> 8);
}

fn react_to_death(runtime: &mut Runtime<'_>, index: u16) {
    let key = Key::from_legacy(index);
    if runtime.resolve_actor(key, Native).health != 0 {
        return;
    }
    decrement_active(runtime, key);
    release_spawn(runtime, key);
    if runtime.resolve_actor(key, Native).parameters[0] != 0 {
        runtime.edit_actor(key, Native, |actor| {
            actor.properties |= TANGIBLE | INVISIBLE
        });
        let point = {
            let actor = runtime.resolve_actor(key, Native);
            crate::types::Point16U {
                x: actor.x.whole,
                y: actor.y.whole,
            }
        };
        runtime.foreign(|game| {
            game.eproj_spawn_pt = point;
            SpawnEprojWithRoomGfx(0xe509, 3, game);
        });
        runtime.edit_actor(key, Native, |actor| {
            actor.variables[0] = 0xb844;
            actor.pre_instruction = 1;
        });
    } else {
        actors::combat::die(runtime, DeathKind::Rinka { animation: 0 });
    }
}

pub(crate) fn initialize(game: &mut Game) {
    let mut runtime = Runtime::new(game);
    let key = Key::from_legacy(runtime.word(0xe54));
    let properties = if runtime.resolve_actor(key, Native).parameters[0] != 0 {
        let index = runtime.word(0xe54);
        select_spawn(&mut runtime, index);
        0x2c00
    } else {
        0x6400
    };
    runtime.edit_actor(key, Native, |actor| {
        actor.properties = (actor.properties & !0x6c00) | properties;
        actor.palette = 1024;
    });
    let key = Key::from_legacy(runtime.word(0xe54));
    start_animation(&mut runtime, key);
}

pub(crate) fn advance(game: &mut Game) -> bool {
    let mut runtime = Runtime::new(game);
    let key = Key::from_legacy(runtime.word(0xe54));
    if runtime.resolve_actor(key, Native).parameters[0] != 0
        && runtime.native_word(BOSS_FINISHED) != 0
    {
        let key = Key::from_legacy(runtime.word(0xe54));
        decrement_active(&mut runtime, key);
        let key = Key::from_legacy(runtime.word(0xe54));
        release_spawn(&mut runtime, key);
        actors::combat::die(&mut runtime, DeathKind::Rinka { animation: 0 });
        return true;
    }
    let phase = runtime.resolve_actor(key, Native).variables[0];
    let index = runtime.word(0xe54);
    let key = Key::from_legacy(index);
    match phase {
        0xb852 | 0xb85b => {
            if phase == 0xb85b {
                let moving = Key::from_legacy(runtime.word(0xe54));
                runtime.edit_actor(moving, Native, |actor| {
                    advance_native(&mut actor.x, actor.variables[1]);
                    advance_native(&mut actor.y, actor.variables[2]);
                });
            }
            if outside_flight_view(&runtime, key) {
                recycle(&mut runtime, index);
            }
        }
        0xb7df | 0xb844 => {
            let expired = runtime.edit_actor(key, Native, |actor| {
                actor.pre_instruction = actor.pre_instruction.wrapping_sub(1);
                actor.pre_instruction & 0x8000 != 0
            });
            if expired {
                if phase == 0xb844 {
                    runtime.edit_actor(key, Native, |actor| actor.health = 10);
                    respawn(&mut runtime, index);
                } else {
                    runtime.edit_actor(key, Native, |actor| {
                        actor.variables[0] = 0xb85b;
                        actor.properties = if actor.parameters[0] != 0 {
                            actor.properties & !TANGIBLE
                        } else {
                            (actor.properties & 0xf3ff) | 0x800
                        };
                    });
                    let x = runtime
                        .word(0xaf6)
                        .wrapping_sub(runtime.resolve_actor(key, Native).x.whole);
                    let y = runtime
                        .word(0xafa)
                        .wrapping_sub(runtime.resolve_actor(key, Native).y.whole);
                    let angle = (-(i32::from(angle_from_delta(x, y)) + 0x80)) as u8 as u16;
                    runtime.edit_actor(key, Native, |actor| {
                        actor.variables[1] = Math_MultBySin(0x120, angle);
                        actor.variables[2] = Math_MultByCos(0x120, angle);
                    });
                }
            }
        }
        _ => return false,
    }
    true
}

pub(crate) fn frozen(index: u16, game: &mut Game) {
    let mut runtime = Runtime::new(game);
    let key = Key::from_legacy(index);
    if outside_flight_view(&runtime, key) {
        runtime.edit_actor(key, Native, |actor| actor.frozen = 0);
    }
    actors::scheduling::update_frozen_enemy(&mut runtime);
    if runtime.native_word(BOSS_FINISHED) != 0 {
        decrement_active(&mut runtime, key);
        release_spawn(&mut runtime, key);
        actors::combat::die(&mut runtime, DeathKind::Rinka { animation: 0 });
    }
}

pub(crate) fn power_bomb(index: u16, game: &mut Game) {
    let mut runtime = Runtime::new(game);
    if runtime
        .resolve_actor(Key::from_legacy(index), Native)
        .properties
        & INVISIBLE
        == 0
    {
        actors::combat::power_bomb_reaction(&mut runtime, DeathHandling::LeavePending);
        react_to_death(&mut runtime, index);
    }
}

pub(crate) fn contact(game: &mut Game) {
    let mut runtime = Runtime::new(game);
    actors::combat::contact_reaction(&mut runtime, DeathHandling::LeavePending);
    let index = runtime.word(0xe54);
    react_to_death(&mut runtime, index);
}

pub(crate) fn shot(game: &mut Game) {
    let mut runtime = Runtime::new(game);
    actors::combat::shot_reaction(&mut runtime, ShotHandling::ReactionOnly);
    let index = runtime.word(0xe54);
    react_to_death(&mut runtime, index);
}

pub(crate) fn hide(index: u16, offscreen: bool, game: &mut Game) {
    Runtime::new(game).edit_actor(Key::from_legacy(index), Native, |actor| {
        actor.properties |= TANGIBLE | INVISIBLE | if offscreen { 0x800 } else { 0 };
    });
}

pub(crate) fn launch(index: u16, game: &mut Game) {
    let mut runtime = Runtime::new(game);
    runtime.edit_actor(Key::from_legacy(index), Native, |actor| {
        actor.properties &= !(TANGIBLE | INVISIBLE);
        actor.variables[0] = 0xb7df;
    });
    let count = runtime.native_word(ACTIVE_COUNT).wrapping_add(1);
    runtime.write_slice_native(ACTIVE_COUNT, count);
}
