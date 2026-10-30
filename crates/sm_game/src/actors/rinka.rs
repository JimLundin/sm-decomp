//! Rinka, the ring that drifts out of its spawn point at Samus, respawning off screen; in
//! Mother Brain's room they share spawn points and a counter (bank $A2:B602-$A2:BA5A).
#![forbid(unsafe_code)]
use crate::actors::access::{AccessMode::Native, ActorAccess as _, ActorKey as Key};
use crate::actors::api::{DeathHandling, DeathKind, ShotHandling, angle_from_delta};
use crate::actors::combat;
use crate::actors::host::{ActorLease, ActorRuntime as _, BossEffects as _, LeaseActors};
use crate::actors::{population, scheduling};
use crate::enemy_projectiles::shared_pre::{x_velocity_at_angle, y_velocity_at_angle};
use crate::enemy_script::EnemyScript;
use crate::geometry::Position;
use crate::player_projectiles::content::table;
use crate::ram::words;

/// Rinka functions, in its first actor word; its second and third are its X and Y
/// velocities, its fourth its spawn point availability index, and its pre-instruction
/// word a timer. A non-zero first parameter is a Mother Brain's room rinka.
const FIRE: u16 = 0xb7df;
const KILLED: u16 = 0xb844;
const WAITING_TO_FIRE: u16 = 0xb852;
const MOVING: u16 = 0xb85b;
const INVISIBLE: u16 = 0x100;
const INTANGIBLE: u16 = 0x400;
const OFF_SCREEN: u16 = 0x800;
const DELETE: u16 = 0x200;
/// $7E:783A: Mother Brain's room rinkas finished; $7E:783C: the rinka counter.
const FINISHED: usize = 0x783a;
const COUNTER: usize = 0x783c;
/// $A2:B75B: Mother Brain's room spawn X, Y and availability index, 11 rows; $7E:87FE:
/// the spawn point availability words; $7E:7020: each enemy's spawn position.
const SPAWNS: u32 = 0xa2_b75b;
const SPAWN_ROWS: usize = 33;
const AVAILABILITY: usize = 0x87fe;
const SPAWN_POSITION: usize = 0x7020;

fn spawn_word(runtime: &impl ActorLease, index: usize) -> u16 {
    table(runtime, SPAWNS, index as isize)
}
fn occupied(runtime: &impl ActorLease, id: u16) -> bool {
    runtime.lease_native_word(AVAILABILITY + usize::from(id)) & 1 != 0
}
fn mother_brains(runtime: &impl ActorLease, key: Key) -> bool {
    runtime.resolve_actor(key, Native).parameters[0] != 0
}
/// Check if position is on screen ($A2:B8FF), as the source does: the Y sign twice.
fn outside_spawn_view(runtime: &impl ActorLease, x: u16, y: u16) -> bool {
    let (left, top) = (
        runtime.word(words::layer1_x_pos),
        runtime.word(words::layer1_y_pos),
    );
    y & 0x8000 != 0
        || (y.wrapping_sub(top) as i16) < 0
        || y.wrapping_sub(top).wrapping_sub(224) & 0x8000 == 0
        || y & 0x8000 != 0
        || (x.wrapping_sub(left) as i16) < 0
        || x.wrapping_sub(left).wrapping_sub(256) & 0x8000 == 0
}
/// Get availability index of enemy spawn position ($A2:B79D).
fn saved_spawn(runtime: &impl ActorLease, index: u16) -> u16 {
    let saved = SPAWN_POSITION + usize::from(index);
    for row in (0..SPAWN_ROWS).step_by(3) {
        if spawn_word(runtime, row) == runtime.lease_native_word(saved)
            && spawn_word(runtime, row + 1) == runtime.lease_native_word(saved + 2)
        {
            return spawn_word(runtime, row + 2);
        }
    }
    spawn_word(runtime, 2)
}
/// Spawn Mother Brain's room rinka ($A2:B69B): its own point if on screen and free, else
/// the first free on screen, else the first free.
fn select_spawn(runtime: &mut impl ActorLease, index: u16) {
    let key = Key::from_legacy(index);
    let saved = SPAWN_POSITION + usize::from(index);
    let (x, y) = (
        runtime.lease_native_word(saved),
        runtime.lease_native_word(saved + 2),
    );
    if !outside_spawn_view(runtime, x, y) {
        let id = saved_spawn(runtime, index);
        if !occupied(runtime, id) {
            runtime.edit_actor(key, Native, |actor| actor.variables[3] = id);
            runtime.write_slice_native(AVAILABILITY + usize::from(id), u16::MAX);
            return;
        }
    }
    for visible in [true, false] {
        for row in (0..SPAWN_ROWS).step_by(3) {
            let (x, y) = (spawn_word(runtime, row), spawn_word(runtime, row + 1));
            if visible && outside_spawn_view(runtime, x, y)
                || occupied(runtime, spawn_word(runtime, row + 2))
            {
                continue;
            }
            runtime.write_slice_native(saved, x);
            runtime.edit_actor(key, Native, |actor| actor.x.whole = x);
            runtime.write_slice_native(saved + 2, y);
            runtime.edit_actor(key, Native, |actor| actor.y.whole = y);
            let id = spawn_word(runtime, row + 2);
            // The visible choice publishes the availability before the actor's index.
            if visible {
                runtime.write_slice_native(AVAILABILITY + usize::from(id), u16::MAX);
            }
            runtime.edit_actor(key, Native, |actor| actor.variables[3] = id);
            if !visible {
                runtime.write_slice_native(AVAILABILITY + usize::from(id), u16::MAX);
            }
            return;
        }
    }
}
/// Reset rinka ($A2:B654): wait to fire, or go if Mother Brain's room is finished.
fn reset(runtime: &mut impl ActorLease, key: Key) {
    runtime.edit_actor(key, Native, |actor| {
        actor.variables[0] = WAITING_TO_FIRE;
        actor.pre_instruction = 26;
        actor.variables[1] = 0;
        actor.variables[2] = 0;
    });
    let mother_brains = mother_brains(runtime, key);
    if mother_brains && runtime.lease_native_word(FINISHED) != 0 {
        runtime.edit_actor(key, Native, |actor| actor.properties |= DELETE);
    } else {
        // Instruction list - Mother Brain's room ($A2:BA0C) or not ($A2:B9E0).
        let list = if mother_brains { 0xba0c } else { 0xb9e0 };
        runtime.edit_actor(key, Native, |actor| actor.restart_animation(list));
    }
}
/// Respawn rinka ($A2:B63E).
fn respawn(runtime: &mut impl ActorLease, index: u16) {
    let key = Key::from_legacy(index);
    if mother_brains(runtime, key) {
        select_spawn(runtime, index);
    }
    let saved = SPAWN_POSITION + usize::from(index);
    let x = runtime.lease_native_word(saved);
    runtime.edit_actor(key, Native, |actor| actor.x.whole = x);
    let y = runtime.lease_native_word(saved + 2);
    runtime.edit_actor(key, Native, |actor| actor.y.whole = y);
    reset(runtime, key);
}
/// Decrement rinka counter ($A2:B880), for a visible Mother Brain's room rinka.
fn decrement_counter(runtime: &mut impl ActorLease, key: Key) {
    if mother_brains(runtime, key) && runtime.resolve_actor(key, Native).properties & INVISIBLE == 0
    {
        let count = runtime.lease_native_word(COUNTER).wrapping_sub(1) as i16;
        runtime.write_slice_native(COUNTER, count.max(0) as u16);
    }
}
/// Mark rinka spawn point available ($A2:B8BB).
fn release_spawn(runtime: &mut impl ActorLease, key: Key) {
    if mother_brains(runtime, key) {
        let id = runtime.resolve_actor(key, Native).variables[3];
        if id != 0 {
            runtime.write_slice_native(AVAILABILITY + usize::from(id), 0);
            runtime.edit_actor(key, Native, |actor| actor.variables[3] = 0);
        }
    }
}
/// Check if rinka is on screen ($A2:B8D3), with 10h px of margin.
fn off_screen(runtime: &impl ActorLease, key: Key) -> bool {
    let actor = runtime.resolve_actor(key, Native);
    let (x, y) = (actor.x.whole, actor.y.whole);
    let top = y
        .wrapping_add(16)
        .wrapping_sub(runtime.word(words::layer1_y_pos));
    let left = x
        .wrapping_add(16)
        .wrapping_sub(runtime.word(words::layer1_x_pos));
    let on_screen = y & 0x8000 == 0
        && top & 0x8000 == 0
        && top.wrapping_sub(256) & 0x8000 != 0
        && x & 0x8000 == 0
        && left & 0x8000 == 0
        && left.wrapping_sub(288) & 0x8000 != 0;
    !on_screen
}
/// Delete and respawn rinka ($A2:B865).
fn recycle(runtime: &mut impl ActorLease, index: u16) {
    let key = Key::from_legacy(index);
    let mut delete = false;
    if mother_brains(runtime, key) {
        release_spawn(runtime, key);
        delete = runtime.lease_native_word(FINISHED) != 0;
    }
    decrement_counter(runtime, key);
    if delete {
        population::delete_connected(runtime);
    } else {
        respawn(runtime, index);
    }
}
/// Move by an 8.8 velocity through the physical byte lanes, as the source does.
fn advance_native(position: &mut Position, velocity: u16) {
    let mut fraction = position.fraction.to_ne_bytes();
    let carry = u16::from(fraction[1]) + u16::from(velocity.to_ne_bytes()[0]);
    fraction[1] = carry as u8;
    position.fraction = u16::from_ne_bytes(fraction);
    position.whole = position
        .whole
        .wrapping_add_signed(i16::from((velocity >> 8) as i8))
        .wrapping_add(carry >> 8);
}
fn die(runtime: &mut impl ActorLease) {
    combat::die(runtime, DeathKind::Rinka { animation: 0 });
}
/// Rinka shared contact reaction ($A2:B960): a dead Mother Brain's room rinka bursts and
/// waits to respawn; any other dies.
fn react_to_death(runtime: &mut impl ActorLease, index: u16) {
    let key = Key::from_legacy(index);
    if runtime.resolve_actor(key, Native).health != 0 {
        return;
    }
    decrement_counter(runtime, key);
    release_spawn(runtime, key);
    if !mother_brains(runtime, key) {
        return die(runtime);
    }
    runtime.edit_actor(key, Native, |actor| {
        actor.properties |= INTANGIBLE | INVISIBLE
    });
    let (x, y) = (
        runtime.resolve_actor(key, Native).x.whole,
        runtime.resolve_actor(key, Native).y.whole,
    );
    runtime.foreign(|game| {
        game.set_projectile_spawn_point(x, y);
        game.spawn_room_projectile(0xe509, 3);
    });
    runtime.edit_actor(key, Native, |actor| {
        actor.variables[0] = KILLED;
        actor.pre_instruction = 1;
    });
}
/// Initialisation AI - rinka ($A2:B602).
pub fn initialize<H: LeaseActors + ?Sized>(game: &mut H) {
    let mut lease = game.lease();
    let runtime = &mut lease;
    let key = combat::current(runtime);
    let properties = if mother_brains(runtime, key) {
        select_spawn(runtime, key.byte_index());
        0x2c00
    } else {
        0x6400
    };
    runtime.edit_actor(key, Native, |actor| {
        actor.properties = (actor.properties & !0x6c00) | properties;
        actor.palette = 0x400;
    });
    reset(runtime, combat::current(runtime));
}
/// Main AI - rinka ($A2:B7C4); false for an unknown function, as the source fails.
pub fn main_ai<H: LeaseActors + ?Sized>(game: &mut H) -> bool {
    let mut lease = game.lease();
    let runtime = &mut lease;
    let key = combat::current(runtime);
    if mother_brains(runtime, key) && runtime.lease_native_word(FINISHED) != 0 {
        decrement_counter(runtime, key);
        release_spawn(runtime, key);
        die(runtime);
        return true;
    }
    let index = key.byte_index();
    match runtime.resolve_actor(key, Native).variables[0] {
        function @ (WAITING_TO_FIRE | MOVING) => {
            if function == MOVING {
                runtime.edit_actor(key, Native, |actor| {
                    advance_native(&mut actor.x, actor.variables[1]);
                    advance_native(&mut actor.y, actor.variables[2]);
                });
            }
            if off_screen(runtime, key) {
                recycle(runtime, index);
            }
        }
        function @ (FIRE | KILLED) => {
            let expired = runtime.edit_actor(key, Native, |actor| {
                actor.pre_instruction = actor.pre_instruction.wrapping_sub(1);
                actor.pre_instruction & 0x8000 != 0
            });
            if !expired {
            } else if function == KILLED {
                runtime.edit_actor(key, Native, |actor| actor.health = 10);
                respawn(runtime, index);
            } else {
                runtime.edit_actor(key, Native, |actor| {
                    actor.variables[0] = MOVING;
                    actor.properties = if actor.parameters[0] != 0 {
                        actor.properties & !INTANGIBLE
                    } else {
                        (actor.properties & 0xf3ff) | OFF_SCREEN
                    };
                });
                let x = runtime
                    .word(words::samus_x_pos)
                    .wrapping_sub(runtime.resolve_actor(key, Native).x.whole);
                let y = runtime
                    .word(words::samus_y_pos)
                    .wrapping_sub(runtime.resolve_actor(key, Native).y.whole);
                let angle = (-(i32::from(angle_from_delta(x, y)) + 0x80)) as u8 as u16;
                runtime.edit_actor(key, Native, |actor| {
                    actor.variables[1] = x_velocity_at_angle(0x120, angle);
                    actor.variables[2] = y_velocity_at_angle(0x120, angle);
                });
            }
        }
        _ => return false,
    }
    true
}
/// Frozen AI - rinka ($A2:B929): thaw off screen; go once Mother Brain's room is finished.
pub fn frozen<H: LeaseActors + ?Sized>(game: &mut H) {
    let mut lease = game.lease();
    let runtime = &mut lease;
    let key = combat::current(runtime);
    if off_screen(runtime, key) {
        runtime.edit_actor(key, Native, |actor| actor.frozen = 0);
    }
    scheduling::update_frozen_enemy(runtime);
    if runtime.lease_native_word(FINISHED) != 0 {
        decrement_counter(runtime, key);
        release_spawn(runtime, key);
        die(runtime);
    }
}
/// Power bomb reaction - rinka ($A2:B953), only when visible.
pub fn power_bomb<H: LeaseActors + ?Sized>(game: &mut H) {
    let mut lease = game.lease();
    let runtime = &mut lease;
    let key = combat::current(runtime);
    if runtime.resolve_actor(key, Native).properties & INVISIBLE == 0 {
        combat::power_bomb_reaction(runtime, DeathHandling::LeavePending);
        react_to_death(runtime, key.byte_index());
    }
}
/// Enemy touch ($A2:B947) or shot ($A2:B94D) - rinka.
pub fn react<H: LeaseActors + ?Sized>(shot: bool, game: &mut H) {
    let mut lease = game.lease();
    let runtime = &mut lease;
    if shot {
        combat::shot_reaction(runtime, ShotHandling::ReactionOnly);
    } else {
        combat::contact_reaction(runtime, DeathHandling::LeavePending);
    }
    let index = combat::current(runtime).byte_index();
    react_to_death(runtime, index);
}
/// Instructions - set enemy as intangible and invisible ($A2:B9B3), also active off
/// screen ($A2:B9BD), or fire rinka ($A2:B9C7), for enemy `k`.
pub fn instruction<H: LeaseActors + ?Sized>(
    ea: u32,
    k: u16,
    j: EnemyScript,
    game: &mut H,
) -> EnemyScript {
    let mut lease = game.lease();
    let runtime = &mut lease;
    let key = Key::from_legacy(k);
    if ea == 0xa2_b9c7 {
        runtime.edit_actor(key, Native, |actor| {
            actor.properties &= !(INTANGIBLE | INVISIBLE);
            actor.variables[0] = FIRE;
        });
        let count = runtime.lease_native_word(COUNTER).wrapping_add(1);
        runtime.write_slice_native(COUNTER, count);
    } else {
        let off_screen = if ea == 0xa2_b9bd { OFF_SCREEN } else { 0 };
        runtime.edit_actor(key, Native, |actor| {
            actor.properties |= INTANGIBLE | INVISIBLE | off_screen
        });
    }
    j
}
