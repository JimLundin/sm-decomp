//! Alcoon, Norfair's lava seahorse, which leaps out of the lava near Samus,
//! walks and spits fireballs, then dives back in (bank $A8:DBC7-$A8:DFA2).
//! Was `lava_seahorse`.
use crate::actors::access::{AccessMode::Native, ActorAccess, ActorKey as Key};
use crate::actors::api::{self, MovementResult::Blocked, SlopePolicy};
use crate::actors::host::{ActorLease, ActorRuntime as _, LeaseActors, abs16};
use crate::actors::layouts::part_view;
use crate::enemy_projectiles::host::Effects as _;
use crate::enemy_script::EnemyScript;
use crate::ram::words as W;
use crate::session_slot;

part_view! {
    /// Alcoon's view of its slot and extended RAM.
    Alcoon: EnemyLayout40 {
        words: [function, y_velocity, y_subvelocity, x_velocity, unread_0fb0, spawn_y, parameter_1, parameter_2],
        pad1: [u8; 26696],
        words: [
            y_acceleration, y_subacceleration, spawn_x, landing_y, step_counter,
        ],
        _: [u16; 27],
    }
}

/// Alcoon's actor words, by `variables` lane; its spawn Y is the
/// pre-instruction word.
mod lane {
    pub const FUNCTION: usize = 0;
    pub const Y_VELOCITY: usize = 1;
    pub const Y_SUBVELOCITY: usize = 2;
    pub const X_VELOCITY: usize = 3;
    pub const UNREAD: usize = 4;
}

/// Bank $A8 Alcoon functions, stored in `function`.
pub mod function {
    pub const WAIT_FOR_SAMUS: u16 = 0xdd71;
    pub const EMERGING_RISING: u16 = 0xddc6;
    pub const EMERGING_FALLING: u16 = 0xde05;
    pub const EMERGED: u16 = 0xde4b;
    /// An RTL, while spitting fireballs.
    pub const RTL: u16 = 0xdecc;
    pub const HIDING_RISING: u16 = 0xdecd;
    pub const HIDING_FALLING: u16 = 0xdeec;
}

/// Bank $A8 Alcoon instruction lists, facing left; facing right is 70h further.
pub mod instruction_list {
    pub const LEFT_WALKING: u16 = 0xdbe7;
    pub const LEFT_WALKING_STEP: u16 = 0xdbe9;
    pub const LEFT_SPAWN_FIREBALLS: u16 = 0xdc03;
    pub const LEFT_LOOKING_UP: u16 = 0xdc4b;
    pub const LEFT_LOOKING_FORWARD: u16 = 0xdc51;
    pub const RIGHT_WALKING: u16 = 0xdc57;
    pub const RIGHT_WALKING_STEP: u16 = 0xdc59;
    pub const RIGHT_SPAWN_FIREBALLS: u16 = 0xdc73;
    pub const RIGHT_LOOKING_UP: u16 = 0xdcbb;
    pub const RIGHT_LOOKING_FORWARD: u16 = 0xdcc1;
}

/// Bank $A8 instructions Alcoon's scripts call.
pub mod instruction {
    pub const SPAWN_FIREBALL_0: u32 = 0xa8_df1c;
    pub const SPAWN_FIREBALL_2: u32 = 0xa8_df33;
    pub const SPAWN_FIREBALL_4: u32 = 0xa8_df39;
    pub const START_WALKING: u32 = 0xa8_df3f;
    pub const STEP_AND_MOVE_HORIZONTALLY: u32 = 0xa8_df63;
    pub const MOVE_HORIZONTALLY: u32 = 0xa8_df71;
}

/// Alcoon constants ($A8:DCC7): the X thresholds to emerge and to hide, and
/// the Y range to emerge.
const EMERGE_X: i16 = 0x50;
const HIDE_X: i16 = 0x70;
const EMERGE_Y: u16 = 32;
/// Set up jump movement ($A8:DD37): Y velocity -C.0h, acceleration 0.8000h;
/// hiding jumps start at -4.0h.
const JUMP_VELOCITY: i16 = -12;
const HIDE_VELOCITY: i16 = -4;
const JUMP_ACCELERATION: u16 = 0x8000;
/// Alcoon walks 2 px a frame and is pushed down 2 px while emerged.
const WALK_SPEED: i16 = 2;
const PUSH_DOWN: i32 = 2 << 16;
/// Properties bit 13: process instructions.
const PROCESS_INSTRUCTIONS: u16 = 0x2000;
/// Alcoon's cry (5Eh) and fireball (3Fh), library 2; the fireball ($86:9E90).
const CRY_SOUND: u16 = 0x5e;
const FIREBALL_SOUND: u16 = 0x3f;
const FIREBALL: u16 = 0x9e90;
const SIGN: u16 = 0x8000;
/// $7E:0F7E and $7E:0FAA: the Y position and Y velocity words, read as
/// whole-and-fraction pairs from the slot record.
const Y_POSITION: usize = 0xf7e;
const Y_VELOCITY: usize = 0xfaa;

fn selected(runtime: &impl ActorLease) -> Key {
    Key::from_legacy(runtime.word(W::CUR_ENEMY_INDEX))
}
fn face(runtime: &mut impl ActorLease, key: Key, left: u16, right: u16) {
    runtime.edit_actor(key, Native, |actor| {
        actor.instruction = if actor.variables[lane::X_VELOCITY] & SIGN != 0 {
            left
        } else {
            right
        };
        actor.instruction_delay = 1;
    });
}
/// Set up jump movement ($A8:DD37).
fn set_up_jump(runtime: &mut impl ActorLease, key: Key) {
    runtime.edit_actor(key, Native, |actor| {
        actor.variables[lane::Y_VELOCITY] = JUMP_VELOCITY as u16;
        actor.variables[lane::Y_SUBVELOCITY] = 0;
        actor.variables[lane::X_VELOCITY] = 0;
        actor.variables[lane::UNREAD] = 0;
    });
    let alcoon = i32::from(key.byte_index());
    session_slot!(runtime, Alcoon[alcoon]; y_acceleration = 0, y_subacceleration = JUMP_ACCELERATION);
}
fn pair(runtime: &impl ActorLease, at: usize) -> u32 {
    (u32::from(runtime.lease_native_word(at)) << 16) | u32::from(runtime.lease_native_word(at + 2))
}
fn add_pair(runtime: &mut impl ActorLease, at: usize, amount: u32) -> u16 {
    let low = runtime.lease_native_word(at + 2);
    let high = runtime.lease_native_word(at);
    let next = ((u32::from(high) << 16) | u32::from(low)).wrapping_add(amount);
    runtime.write_slice_native(at + 2, next as u16);
    runtime.write_slice_native(at, (next >> 16) as u16);
    (next >> 16) as u16
}
fn velocity(runtime: &impl ActorLease, key: Key) -> u32 {
    pair(runtime, Y_VELOCITY + usize::from(key.byte_index()))
}
/// Alcoon Y acceleration ($A8:DD55); the new Y velocity.
fn accelerate(runtime: &mut impl ActorLease, key: Key) -> u16 {
    let alcoon = i32::from(key.byte_index());
    let amount = session_slot!(runtime, Alcoon[alcoon](y_acceleration, y_subacceleration));
    add_pair(runtime, Y_VELOCITY + usize::from(key.byte_index()), amount)
}
fn drift(runtime: &mut impl ActorLease, key: Key) {
    let amount = velocity(runtime, key);
    add_pair(runtime, Y_POSITION + usize::from(key.byte_index()), amount);
}
fn samus_offset(runtime: &impl ActorLease, key: Key) -> u16 {
    runtime
        .player_x()
        .wrapping_sub(runtime.resolve_actor(key, Native).x.whole)
}
fn direction(left: bool) -> u16 {
    if left {
        WALK_SPEED.wrapping_neg() as u16
    } else {
        WALK_SPEED as u16
    }
}
/// Initialisation AI - Alcoon ($A8:DCCD): simulate a jump to find the
/// landing Y position.
pub fn initialize(runtime: &mut impl ActorLease) {
    let key = selected(runtime);
    let alcoon = i32::from(key.byte_index());
    session_slot!(runtime, Alcoon[alcoon].step_counter = 0);
    runtime.edit_actor(key, Native, |actor| actor.pre_instruction = actor.y.whole);
    let x = runtime.resolve_actor(key, Native).x.whole;
    session_slot!(runtime, Alcoon[alcoon].spawn_x = x);
    runtime.edit_actor(key, Native, |actor| {
        actor.properties |= PROCESS_INSTRUCTIONS;
        actor.instruction = instruction_list::LEFT_WALKING;
        actor.variables[lane::FUNCTION] = function::WAIT_FOR_SAMUS;
    });
    let live = selected(runtime);
    set_up_jump(runtime, live);
    loop {
        drift(runtime, key);
        let live = selected(runtime);
        if accelerate(runtime, live) & SIGN == 0 {
            break;
        }
    }
    loop {
        let live = selected(runtime);
        let amount = velocity(runtime, key) as i32;
        if api::move_vertical(runtime, live, amount) == Blocked {
            break;
        }
        let live = selected(runtime);
        accelerate(runtime, live);
    }
    let y = runtime.resolve_actor(key, Native).y.whole;
    session_slot!(runtime, Alcoon[alcoon].landing_y = y);
    runtime.edit_actor(key, Native, |actor| actor.y.whole = actor.pre_instruction);
}
/// Main AI - Alcoon ($A8:DD6B): run its function. Was `advance`.
pub fn main_ai(runtime: &mut impl ActorLease) {
    let key = selected(runtime);
    let alcoon = i32::from(key.byte_index());
    let function = runtime.resolve_actor(key, Native).variables[lane::FUNCTION];
    match function {
        // Alcoon function - wait for Samus to get near ($A8:DD71).
        function::WAIT_FOR_SAMUS => {
            let dy =
                session_slot!(runtime, Alcoon[alcoon].landing_y).wrapping_sub(runtime.player_y());
            if abs16(dy).wrapping_sub(EMERGE_Y) & SIGN == 0 {
                return;
            }
            let dx = samus_offset(runtime, key);
            if (abs16(dx) as i16).wrapping_sub(EMERGE_X) >= 0 {
                return;
            }
            set_up_jump(runtime, key);
            runtime.edit_actor(key, Native, |actor| {
                actor.variables[lane::X_VELOCITY] = direction(dx & SIGN != 0)
            });
            face(
                runtime,
                key,
                instruction_list::LEFT_LOOKING_UP,
                instruction_list::RIGHT_LOOKING_UP,
            );
            runtime.edit_actor(key, Native, |actor| {
                actor.variables[lane::FUNCTION] = function::EMERGING_RISING
            });
            runtime.queue_sound(2, 6, CRY_SOUND);
        }
        // Alcoon function - emerging - rising ($A8:DDC6), hiding - rising
        // ($A8:DECD) and hiding - falling ($A8:DEEC).
        function::EMERGING_RISING | function::HIDING_RISING | function::HIDING_FALLING => {
            drift(runtime, key);
            let actor = runtime.resolve_actor(key, Native);
            if function == function::HIDING_FALLING
                && (actor.y.whole as i16).wrapping_sub(actor.pre_instruction as i16) >= 0
            {
                runtime.edit_actor(key, Native, |actor| actor.y.whole = actor.pre_instruction);
                let x = session_slot!(runtime, Alcoon[alcoon].spawn_x);
                runtime.edit_actor(key, Native, |actor| {
                    actor.x.whole = x;
                    actor.variables[lane::FUNCTION] = function::WAIT_FOR_SAMUS;
                });
            } else if accelerate(runtime, key) & SIGN == 0 && function != function::HIDING_FALLING {
                runtime.edit_actor(key, Native, |actor| {
                    actor.variables[lane::FUNCTION] = if function == function::EMERGING_RISING {
                        function::EMERGING_FALLING
                    } else {
                        function::HIDING_FALLING
                    }
                });
                if function == function::EMERGING_RISING {
                    face(
                        runtime,
                        key,
                        instruction_list::LEFT_LOOKING_FORWARD,
                        instruction_list::RIGHT_LOOKING_FORWARD,
                    );
                }
            }
        }
        // Alcoon function - emerging - falling ($A8:DE05): land and walk.
        function::EMERGING_FALLING => {
            let amount = velocity(runtime, key) as i32;
            if api::move_vertical(runtime, key, amount) == Blocked {
                let left = samus_offset(runtime, key) & SIGN != 0;
                runtime.edit_actor(key, Native, |actor| {
                    actor.variables[lane::X_VELOCITY] = direction(left)
                });
                face(
                    runtime,
                    key,
                    instruction_list::LEFT_WALKING,
                    instruction_list::RIGHT_WALKING,
                );
                runtime.edit_actor(key, Native, |actor| {
                    actor.variables[lane::FUNCTION] = function::EMERGED
                });
                session_slot!(runtime, Alcoon[alcoon].step_counter = 1);
            } else {
                accelerate(runtime, key);
            }
        }
        // Alcoon function - emerged ($A8:DE4B): hide when far from home,
        // else spit fireballs at Samus once she is ahead.
        function::EMERGED => {
            api::move_vertical(runtime, key, PUSH_DOWN);
            let dx = session_slot!(runtime, Alcoon[alcoon].spawn_x)
                .wrapping_sub(runtime.resolve_actor(key, Native).x.whole);
            if (abs16(dx) as i16).wrapping_sub(HIDE_X) >= 0 {
                runtime.edit_actor(key, Native, |actor| {
                    actor.variables[lane::FUNCTION] = function::HIDING_RISING;
                    actor.variables[lane::Y_VELOCITY] = HIDE_VELOCITY as u16;
                    actor.variables[lane::Y_SUBVELOCITY] = 0;
                });
                face(
                    runtime,
                    key,
                    instruction_list::LEFT_LOOKING_FORWARD,
                    instruction_list::RIGHT_LOOKING_FORWARD,
                );
            } else if session_slot!(runtime, Alcoon[alcoon].step_counter) == 0 {
                let left = samus_offset(runtime, key) & SIGN != 0;
                if left
                    == (runtime.resolve_actor(key, Native).variables[lane::X_VELOCITY] & SIGN != 0)
                {
                    face(
                        runtime,
                        key,
                        instruction_list::LEFT_SPAWN_FIREBALLS,
                        instruction_list::RIGHT_SPAWN_FIREBALLS,
                    );
                    runtime.edit_actor(key, Native, |actor| {
                        actor.variables[lane::FUNCTION] = function::RTL
                    });
                }
            }
        }
        function::RTL => {}
        _ => runtime.unreachable(),
    }
}
/// Run one of Alcoon's script instructions (one of [`instruction`]).
pub fn instruction<H: LeaseActors + ?Sized>(
    address: u32,
    script: EnemyScript,
    game: &mut H,
) -> EnemyScript {
    use instruction::*;
    let runtime = &mut game.lease();
    let key = selected(runtime);
    let alcoon = i32::from(key.byte_index());
    match address {
        // Spawn alcoon fireball with Y velocity index [A] ($A8:DF20).
        SPAWN_FIREBALL_0 | SPAWN_FIREBALL_2 | SPAWN_FIREBALL_4 => {
            let index = match address {
                SPAWN_FIREBALL_0 => 0,
                SPAWN_FIREBALL_2 => 2,
                _ => 4,
            };
            runtime.foreign(|game| {
                game.spawn_projectile(index, key.byte_index(), FIREBALL);
            });
            runtime.queue_sound(2, 6, FIREBALL_SOUND);
        }
        START_WALKING => {
            runtime.edit_actor(key, Native, |actor| {
                actor.variables[lane::FUNCTION] = function::EMERGED
            });
            let count = runtime.word(W::RANDOM_NUMBER) & 3;
            session_slot!(
                runtime,
                Alcoon[alcoon].step_counter = if count == 0 { 2 } else { count }
            );
            return EnemyScript::jump(
                if runtime.resolve_actor(key, Native).variables[lane::X_VELOCITY] & SIGN != 0 {
                    instruction_list::LEFT_WALKING
                } else {
                    instruction_list::RIGHT_WALKING
                },
            );
        }
        STEP_AND_MOVE_HORIZONTALLY | MOVE_HORIZONTALLY => {
            if address == STEP_AND_MOVE_HORIZONTALLY {
                let count = session_slot!(runtime, Alcoon[alcoon].step_counter);
                if count != 0 {
                    session_slot!(runtime, Alcoon[alcoon].step_counter = count.wrapping_sub(1));
                }
            }
            let key = selected(runtime);
            let amount = (runtime.resolve_actor(key, Native).variables[lane::X_VELOCITY] as i16
                as i32)
                << 16;
            if api::move_horizontal(runtime, key, amount, SlopePolicy::Ignore) == Blocked {
                let speed = runtime.resolve_actor(key, Native).variables[lane::X_VELOCITY] as i16;
                let next = if speed < 0 {
                    instruction_list::RIGHT_WALKING_STEP
                } else {
                    instruction_list::LEFT_WALKING_STEP
                };
                runtime.edit_actor(key, Native, |actor| {
                    actor.variables[lane::X_VELOCITY] = speed.wrapping_neg() as u16
                });
                return EnemyScript::jump(next);
            }
            let live = selected(runtime);
            api::snap_to_slopes(runtime, live);
        }
        _ => {
            runtime.unreachable();
            return EnemyScript::Stop;
        }
    }
    script
}
