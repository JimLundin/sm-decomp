//! Rio (bank $A2:BA7B-$A2:BE1B), with the swoop movement Geruta and Holtz share.
#![forbid(unsafe_code)]
use crate::actors::access::{AccessMode::Native, ActorAccess, ActorKey as Key};
use crate::actors::api::{self, Axis, MovementResult, SlopePolicy};
use crate::actors::combat::current;
use crate::actors::host::{ActorLease, BossEffects, LeaseActors, Random};
use crate::player_projectiles::content::offset;
use crate::projectile_geometry::SINE_COSINE;
use crate::session_access::SessionAccess as _;
use core::mem::take;

// Lanes (the last instruction list is the pre-instruction word), then functions.
const HOMING_X_VELOCITY: usize = 0;
const FUNCTION: usize = 1;
const Y_VELOCITY: usize = 2;
const X_VELOCITY: usize = 3;
const ANIMATION_FINISHED: usize = 4;
const WAIT_FOR_SAMUS_TO_GET_NEAR: u16 = 0xbbed;
const SWOOP_COOLDOWN: u16 = 0xbc32;
const SWOOP_DESCENDING: u16 = 0xbc48;
const SWOOP_ASCENDING: u16 = 0xbcb7;
const HOMING: u16 = 0xbcff;

pub fn lanes<R>(h: &mut impl ActorLease, key: Key, edit: impl FnOnce(&mut [u16; 5]) -> R) -> R {
    h.edit_actor(key, Native, |actor| edit(&mut actor.variables))
}

/// Set rio instruction list ($A2:BD54): restart the animation only when the list changes.
fn set_instruction_list(h: &mut impl ActorLease, list: u16) {
    h.edit_actor(current(h), Native, |actor| {
        if actor.pre_instruction != list {
            actor.pre_instruction = list;
            actor.restart_animation(list);
        }
    });
}

/// Lane `y` from entry `index` of table 0, lane `y + 1` from table 1 and pointed at Samus.
pub fn aim_velocity(h: &mut impl ActorLease, key: Key, y: usize, tables: [u32; 2], index: isize) {
    for (variable, address, entry) in [(y, tables[0], index), (y + 1, tables[1], 0)] {
        let image = h.leased_image().expect("ROM must be loaded");
        let value = image.table_word(offset(address), entry);
        lanes(h, key, |v| v[variable] = value);
    }
    let dx = h
        .session()
        .player
        .x
        .wrapping_sub(h.resolve_actor(key, Native).x.whole);
    if (dx as i16) < 0 {
        lanes(h, key, |v| v[y + 1] = v[y + 1].wrapping_neg());
    }
}

/// Move along `axis` by the velocity in lane `variable` / 100h; whether it was blocked.
pub fn move_axis(h: &mut impl ActorLease, key: Key, axis: Axis, variable: usize) -> bool {
    let amount = i32::from(h.resolve_actor(key, Native).variables[variable] as i16) << 8;
    let result = match axis {
        Axis::X => api::move_horizontal(h, key, amount, SlopePolicy::Ignore),
        Axis::Y => api::move_vertical(h, key, amount),
    };
    result == MovementResult::Blocked
}

/// Move horizontally by lane `variable`, negating it when blocked; whether it was blocked.
pub fn bounce_horizontal(h: &mut impl ActorLease, key: Key, variable: usize) -> bool {
    let blocked = move_axis(h, key, Axis::X, variable);
    if blocked {
        lanes(h, key, |v| v[variable] = v[variable].wrapping_neg());
    }
    blocked
}

/// Initialisation AI - enemy $D27F (rio) ($A2:BBCD).
pub fn initialize(runtime: &mut impl ActorLease) {
    runtime.edit_actor(current(runtime), Native, |actor| {
        actor.variables[ANIMATION_FINISHED] = 0;
        actor.pre_instruction = 0;
        actor.instruction = 0xbb4b;
        actor.variables[FUNCTION] = WAIT_FOR_SAMUS_TO_GET_NEAR;
    });
}

/// Instruction - set animation finished flag ($A2:BBC3).
pub fn set_animation_finished<H: LeaseActors + ?Sized>(game: &mut H) {
    let mut runtime = game.lease();
    let key = current(&runtime);
    lanes(&mut runtime, key, |v| v[ANIMATION_FINISHED] = 1);
}

/// Main AI - enemy $D27F (rio) ($A2:BBE3) after a random draw; swoop speeds are at $A2:BBBB.
pub fn main_ai<H: LeaseActors + BossEffects + Random + ?Sized>(game: &mut H) {
    game.next_random();
    let mut runtime = game.lease();
    let key = current(&runtime);
    let finished = |h: &mut H::Lease<'_>| lanes(h, key, |v| take(&mut v[ANIMATION_FINISHED]) != 0);
    let mut cry = false;
    match runtime.resolve_actor(key, Native).variables[FUNCTION] {
        WAIT_FOR_SAMUS_TO_GET_NEAR => {
            let h = &mut runtime;
            if ((api::player_delta(h, key, Axis::X) as i16).wrapping_abs() as u16) < 0xa0 {
                aim_velocity(h, key, Y_VELOCITY, [0xa2_bbbb, 0xa2_bbbf], 0);
                set_instruction_list(h, 0xbb7f);
                lanes(h, key, |v| v[FUNCTION] = SWOOP_DESCENDING);
                cry = !api::current_outside_screen(h);
            }
        }
        SWOOP_COOLDOWN => {
            let h = &mut runtime;
            if finished(h) {
                set_instruction_list(h, 0xbb53);
                lanes(h, key, |v| v[FUNCTION] = WAIT_FOR_SAMUS_TO_GET_NEAR);
            }
        }
        SWOOP_DESCENDING => {
            let h = &mut runtime;
            if bounce_horizontal(h, key, X_VELOCITY) || move_axis(h, key, Axis::Y, Y_VELOCITY) {
                lanes(h, key, |v| v[Y_VELOCITY] = v[Y_VELOCITY].wrapping_neg());
                lanes(h, key, |v| v[FUNCTION] = SWOOP_ASCENDING);
            } else if !lanes(h, key, |v| {
                v[Y_VELOCITY] = v[Y_VELOCITY].wrapping_sub(24);
                let homing = (v[Y_VELOCITY] as i16) < 0;
                if homing {
                    v[HOMING_X_VELOCITY] = v[X_VELOCITY];
                    (v[X_VELOCITY], v[Y_VELOCITY], v[FUNCTION]) = (0, 0, HOMING);
                }
                homing
            }) && finished(h)
            {
                set_instruction_list(h, 0xbb97);
            }
        }
        SWOOP_ASCENDING => {
            let h = &mut runtime;
            bounce_horizontal(h, key, X_VELOCITY);
            if move_axis(h, key, Axis::Y, Y_VELOCITY) {
                set_instruction_list(h, 0xbba3);
                lanes(h, key, |v| v[FUNCTION] = SWOOP_COOLDOWN);
            } else {
                lanes(h, key, |v| v[Y_VELOCITY] = v[Y_VELOCITY].wrapping_sub(24));
            }
        }
        HOMING => {
            let h = &mut runtime;
            let y = h.resolve_actor(key, Native).y.whole;
            if (y.wrapping_sub(h.session().player.y) as i16) >= 0 {
                lanes(h, key, |v| {
                    (v[X_VELOCITY], v[Y_VELOCITY]) = (v[HOMING_X_VELOCITY], u16::MAX)
                });
                lanes(h, key, |v| v[FUNCTION] = SWOOP_ASCENDING);
            } else {
                let angle = usize::from(api::angle_to_player(h, key));
                let (x, y) = (SINE_COSINE[angle + 64] as u16, SINE_COSINE[angle] as u16);
                lanes(h, key, |v| (v[X_VELOCITY], v[Y_VELOCITY]) = (x, y));
                move_axis(h, key, Axis::X, X_VELOCITY);
                move_axis(h, key, Axis::Y, Y_VELOCITY);
            }
        }
        _ => {
            drop(runtime);
            return game.unreachable();
        }
    }
    drop(runtime);
    if cry {
        game.queue_sound(1, 0x65, 6);
    }
}
