//! Zoa, Maridia's refill creatures, which rise towards Samus from the sand,
//! shoot off sideways and return off screen (bank $A3:B3A1-$A3:B52F). Was
//! `refill_candy`.
#![forbid(unsafe_code)]
use crate::actors::access::{AccessMode::Native, ActorAccess};
use crate::actors::api::{self, Axis};
use crate::actors::combat;
use crate::actors::host::{ActorLease, LeaseActors};
use crate::enemy_projectiles::host::Effects as _;
use crate::enemy_script::EnemyScript;
use crate::player_projectiles::content::offset;

/// Zoa's actor words, by `variables` lane; its function is in its first
/// parameter word.
mod lane {
    pub const SPAWN_X: usize = 1;
    pub const SPAWN_Y: usize = 2;
    pub const INSTRUCTION_LIST_INDEX: usize = 3;
    pub const PREVIOUS_INSTRUCTION_LIST_INDEX: usize = 4;
}

/// Bank $A3 Zoa functions, stored in the first parameter word.
pub mod function {
    pub const WAIT_FOR_SAMUS: u16 = 0xb482;
    pub const RISING: u16 = 0xb4a8;
    pub const SHOOTING: u16 = 0xb4d6;
}

/// Bank $A3 instructions setting the X speed table index to 4, 8 and Ch.
pub mod instruction {
    pub const X_SPEED_INDEX_4: u32 = 0xa3_b429;
    pub const X_SPEED_INDEX_8: u32 = 0xa3_b434;
    pub const X_SPEED_INDEX_C: u32 = 0xa3_b43f;
}

/// $A3:B3C1: facing left, shooting; $A3:B40D: the instruction list for each
/// instruction list index; $A3:B415: the X speeds, by X speed table index.
const LEFT_SHOOTING: u16 = 0xb3c1;
const INSTRUCTION_LISTS: u32 = 0xa3_b40d;
const X_SPEEDS: u32 = 0xa3_b415;
/// $7E:7800: Zoa's X speed table index.
const X_SPEED_INDEX: usize = 0x7800;
/// Instruction list indices: facing left or right, rising (shooting is one less).
const LEFT_RISING: u16 = 1;
const RIGHT_RISING: u16 = 3;
/// Samus is near within 80h px columns; Zoa rises at 0.8000h px a frame.
const NEAR_COLUMNS: u16 = 0x80;
const RISE: i32 = -0x8000;
/// Properties bit 8: invisible.
const INVISIBLE: u16 = 0x100;
const SIGN: u16 = 0x8000;

fn x_speed_index(runtime: &impl ActorLease) -> usize {
    X_SPEED_INDEX + usize::from(combat::current(runtime).byte_index())
}
fn table(runtime: &impl ActorLease, address: u32, index: isize) -> u16 {
    let image = runtime.leased_image().expect("ROM must be loaded");
    image.table_word(offset(address), index)
}
/// Start the instruction list for Zoa's instruction list index, if it changed.
fn set_instruction_list(runtime: &mut impl ActorLease) {
    let key = combat::current(runtime);
    let index = runtime.resolve_actor(key, Native).variables[lane::INSTRUCTION_LIST_INDEX];
    if index != runtime.resolve_actor(key, Native).variables[lane::PREVIOUS_INSTRUCTION_LIST_INDEX]
    {
        runtime.edit_actor(key, Native, |actor| {
            actor.variables[lane::PREVIOUS_INSTRUCTION_LIST_INDEX] = index
        });
        let instruction = table(runtime, INSTRUCTION_LISTS, index as isize);
        runtime.edit_actor(key, Native, |actor| actor.restart_animation(instruction));
    }
}
/// Initialisation AI - Zoa ($A3:B44A).
pub fn initialize<H: LeaseActors + ?Sized>(game: &mut H) {
    let mut runtime = game.lease();
    let key = combat::current(&runtime);
    runtime.edit_actor(key, Native, |actor| {
        actor.parameters[0] = function::WAIT_FOR_SAMUS;
        actor.variables[lane::INSTRUCTION_LIST_INDEX] = 0;
        actor.variables[lane::PREVIOUS_INSTRUCTION_LIST_INDEX] = 0;
    });
    let at = x_speed_index(&runtime);
    runtime.write_slice_native(at, 0);
    runtime.edit_actor(key, Native, |actor| {
        actor.instruction = LEFT_SHOOTING;
        actor.properties |= INVISIBLE;
        actor.variables[lane::SPAWN_X] = actor.x.whole;
        actor.variables[lane::SPAWN_Y] = actor.y.whole;
    });
}
/// Main AI - Zoa ($A3:B47C): run its function. Was `advance`.
pub fn main_ai<H: LeaseActors + ?Sized>(game: &mut H) {
    let mut runtime = game.lease();
    let key = combat::current(&runtime);
    match runtime.resolve_actor(key, Native).parameters[0] {
        // Zoa function - wait for Samus to get near ($A3:B482).
        function::WAIT_FOR_SAMUS => {
            let current = combat::current(&runtime);
            if api::player_within(&mut runtime, current, Axis::X, NEAR_COLUMNS) {
                let current = combat::current(&runtime);
                let index = if api::player_delta(&runtime, current, Axis::X) & SIGN == 0 {
                    RIGHT_RISING
                } else {
                    LEFT_RISING
                };
                let key = combat::current(&runtime);
                runtime.edit_actor(key, Native, |actor| {
                    actor.variables[lane::INSTRUCTION_LIST_INDEX] = index
                });
                set_instruction_list(&mut runtime);
                runtime.edit_actor(key, Native, |actor| actor.parameters[0] = function::RISING);
            }
        }
        // Zoa function - rising ($A3:B4A8): rise to Samus's height, then shoot.
        function::RISING => {
            runtime.edit_actor(key, Native, |actor| actor.properties &= !INVISIBLE);
            if api::player_delta(&runtime, key, Axis::Y) & SIGN != 0 {
                runtime.edit_actor(key, Native, |actor| actor.y.advance(RISE));
            } else {
                runtime.edit_actor(key, Native, |actor| {
                    actor.variables[lane::INSTRUCTION_LIST_INDEX] =
                        actor.variables[lane::INSTRUCTION_LIST_INDEX].wrapping_sub(1)
                });
                set_instruction_list(&mut runtime);
                runtime.edit_actor(key, Native, |actor| {
                    actor.pre_instruction = 0;
                    actor.parameters[0] = function::SHOOTING;
                });
            }
        }
        // Zoa function - shooting ($A3:B4D6): shoot sideways until off
        // screen, then return to the spawn point and wait.
        function::SHOOTING => {
            let index = (runtime.lease_native_word(x_speed_index(&runtime)) >> 1) as isize;
            let high = table(&runtime, X_SPEEDS, index);
            let low = table(&runtime, X_SPEEDS, index + 1);
            let speed = ((u32::from(high) << 16) | u32::from(low)) as i32;
            // Preserve ordinary signed negation, including its checked-build overflow.
            let speed = if runtime.resolve_actor(key, Native).variables
                [lane::INSTRUCTION_LIST_INDEX]
                != 0
            {
                speed
            } else {
                -speed
            };
            runtime.edit_actor(key, Native, |actor| actor.x.advance(speed));
            let outside = api::current_outside_screen(&runtime);
            if outside {
                runtime.edit_actor(key, Native, |actor| {
                    actor.properties |= INVISIBLE;
                    actor.x.whole = actor.variables[lane::SPAWN_X];
                    actor.y.whole = actor.variables[lane::SPAWN_Y];
                    actor.variables[lane::INSTRUCTION_LIST_INDEX] = 0;
                });
            }
            set_instruction_list(&mut runtime);
            if outside {
                runtime.edit_actor(key, Native, |actor| {
                    actor.parameters[0] = function::WAIT_FOR_SAMUS
                });
            }
        }
        _ => runtime.unreachable(),
    }
}
/// Instruction - enemy X speed table index = `value` ($A3:B429, $A3:B434, $A3:B43F).
pub fn set_x_speed_index<H: LeaseActors + ?Sized>(
    value: u16,
    script: EnemyScript,
    game: &mut H,
) -> EnemyScript {
    let mut runtime = game.lease();
    let at = x_speed_index(&runtime);
    runtime.write_slice_native(at, value);
    script
}
