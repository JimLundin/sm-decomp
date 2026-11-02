//! Gripper, Ripper II and Ripper behavior over the current owned actors.
#![forbid(unsafe_code)]
use super::{INSTRUCTION_LISTS, Kind, SPEED_INDEX, X_SUBVELOCITY, X_VELOCITY};
use crate::actors::access::{AccessMode::Native, ActorKey as Key};
use crate::actors::api::{MovementResult::Blocked, SlopePolicy};
use crate::actors::combat::current;
use sm_data::enemy_speeds::SpeedTable;

/// Speeds retain signed/native table semantics; the shot effect executes the
/// real shared combat/death lifecycle before the live selector is reacquired.
pub trait Host: crate::actors::movement::MovementHost + crate::room_mechanisms::host::Host {
    fn ripper_speed(&self, table: SpeedTable, index: isize) -> u16;
    fn finalized_shot_reaction(&mut self);
}

/// The X velocity from the linear speed table (negated entry moving left), high word first.
fn set_x_velocity(runtime: &mut impl Host, key: Key, speed_index: u16, moving_right: bool) {
    let index = (speed_index >> 1) as isize + if moving_right { 0 } else { 2 };
    for (offset, variable) in [(0, X_VELOCITY), (1, X_SUBVELOCITY)] {
        let value = runtime.ripper_speed(SpeedTable::Linear, index + offset);
        runtime.edit_actor(key, Native, |actor| actor.variables[variable] = value);
    }
}

fn moving_left(runtime: &impl Host, key: Key) -> bool {
    (runtime.resolve_actor(key, Native).variables[X_VELOCITY] & 0x8000) != 0
}

/// Initialisation AI ($A2:E1D3, $A2:E318, $A2:E49F); the Gripper's instruction list word
/// holds its speed and direction.
pub fn initialize(kind: Kind, runtime: &mut impl Host) {
    let key = current(runtime);
    if kind == Kind::Gripper {
        let (speed, moving_right) = runtime.edit_actor(key, Native, |actor| {
            actor.variables[SPEED_INDEX] = u16::from(actor.instruction.to_ne_bytes()[0]) * 8;
            (
                actor.variables[SPEED_INDEX],
                (actor.instruction & 0xfeff) != 0,
            )
        });
        set_x_velocity(runtime, key, speed, moving_right);
        let moving_right = !moving_left(runtime, key);
        runtime.edit_actor(key, Native, |actor| {
            actor.instruction = INSTRUCTION_LISTS[kind as usize][usize::from(moving_right)];
            actor.variables[..X_SUBVELOCITY].copy_from_slice(&actor.parameters);
        });
        return;
    }
    let (speed, moving_right) = runtime.edit_actor(key, Native, |actor| {
        actor.instruction = INSTRUCTION_LISTS[kind as usize][usize::from(actor.parameters[1] != 0)];
        actor.variables[SPEED_INDEX] = actor.parameters[0].wrapping_mul(8);
        (actor.variables[SPEED_INDEX], actor.parameters[1] != 0)
    });
    set_x_velocity(runtime, key, speed, moving_right);
}

/// Main AI ($A2:E221, $A2:E353, $A2:E4DA): turn when blocked or, for the Gripper, past a bound.
pub fn main_ai(kind: Kind, runtime: &mut impl Host) {
    let key = current(runtime);
    let [_, _, low, high, _] = runtime.resolve_actor(key, Native).variables;
    let velocity = ((u32::from(high) << 16) | u32::from(low)) as i32;
    if crate::actors::movement::move_horizontal(runtime, key, velocity, SlopePolicy::Ignore)
        != Blocked
    {
        if kind != Kind::Gripper {
            return;
        }
        // The bounds use the live current enemy; the turn keeps the original one.
        let current = current(runtime);
        let left = moving_left(runtime, current);
        let actor = runtime.resolve_actor(current, Native);
        let bound = actor.variables[usize::from(!left)];
        if ((actor.x.whole as i16).wrapping_sub(bound as i16) < 0) != left {
            return;
        }
    }
    // Movement may change the record: keep its key and reread its fields.
    let speed = runtime.resolve_actor(key, Native).variables[SPEED_INDEX];
    let turn_right = moving_left(runtime, key);
    set_x_velocity(runtime, key, speed, turn_right);
    let list = INSTRUCTION_LISTS[kind as usize][usize::from(turn_right)];
    runtime.edit_actor(key, Native, |actor| actor.restart_animation(list));
}

/// The prepared shot callback owns both synchronous operations.
pub fn shot(runtime: &mut impl Host) {
    runtime.finalized_shot_reaction();
    finish_shot(runtime);
}
/// Compatibility invokes this after its original shot reaction and new lease.
/// The key is deliberately captured after that reaction, not at callback entry.
pub fn finish_shot(runtime: &mut impl Host) {
    let key = current(runtime);
    if runtime.resolve_actor(key, Native).frozen != 0 {
        let spritemap = [0xe44b, 0xe43f][usize::from(moving_left(runtime, key))];
        runtime.edit_actor(key, Native, |actor| actor.spritemap = spritemap);
    }
}
