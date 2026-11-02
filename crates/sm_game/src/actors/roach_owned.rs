//! Roach activation, facing and motion on the current owned actor interval.
#![forbid(unsafe_code)]
use super::{
    FACINGS, FORWARD_FACING, HALF_TURN_FACINGS, LEFT_FACING, MOVEMENT_FRAMES, QUARTER_TURN,
    RANDOM_SEED, REVERSE_DISTANCE, REVERSE_FRAMES, RIGHT_FACING, Roach, SIDE_ANGLE, SIGN,
    THREE_QUARTERS, function, lane,
};
use crate::actors::access::AccessMode::Native;
use crate::actors::api::{self, Axis};
use crate::actors::api::{MovementResult, SlopePolicy};
use crate::actors::combat::{self, current_slot};
use crate::actors::session_slots::{SessionSlots, SlotAccess};
use crate::geometry::Vector32;
use crate::session_slot;

#[derive(Clone, Copy, Debug)]
pub enum RoachTable {
    InstructionLists,
    ActivateFunctions,
}

pub trait Host: SessionSlots + crate::actors::host::ActorMotion {
    fn roach_table(&self, table: RoachTable, index: isize) -> u16;
    fn roach_direction_vector(&mut self, angle: u16, amplitude: u16) -> Vector32;
    fn roach_cosine_full(&mut self, angle: u16, amplitude: u16) -> u32;
    fn roach_sine_full(&mut self, angle: u16, amplitude: u16) -> u32;
    fn roach_random(&mut self) -> u16;
    fn unknown_roach_function(&mut self);
}

fn velocity(runtime: &impl Host, at: usize) -> u32 {
    (u32::from(runtime.slot_word(at + 2)) << 16) | u32::from(runtime.slot_word(at))
}
fn store_velocity(runtime: &mut impl Host, at: usize, value: u32) {
    runtime.set_slot_word(at, value as u16);
    runtime.set_slot_word(at + 2, (value >> 16) as u16);
}
fn forward_x(roach: i32) -> usize {
    crate::ram_slot!(@at Roach, roach, forward_x)
}
fn left_x(roach: i32) -> usize {
    crate::ram_slot!(@at Roach, roach, left_x)
}
fn right_x(roach: i32) -> usize {
    crate::ram_slot!(@at Roach, roach, right_x)
}
fn custom_x(roach: i32) -> usize {
    crate::ram_slot!(@at Roach, roach, custom_x)
}
/// Set roach instruction list ($A3:A12F), if it changed.
fn set_instruction_list(runtime: &mut impl Host) {
    let roach = current_slot(runtime);
    let instruction = session_slot!(runtime, Roach[roach].new_instruction_list);
    if instruction != session_slot!(runtime, Roach[roach].instruction_list) {
        session_slot!(runtime, Roach[roach].instruction_list = instruction);
        let key = combat::current(runtime);
        runtime.edit_actor(key, Native, |actor| actor.restart_animation(instruction));
    }
}
fn face(runtime: &mut impl Host, roach: i32, index: u16) {
    let instruction = runtime.roach_table(RoachTable::InstructionLists, (index >> 1) as isize);
    session_slot!(runtime, Roach[roach].new_instruction_list = instruction);
    set_instruction_list(runtime);
}
/// Calculate the X and Y velocities at `velocities` for the roach's angle,
/// adjusted, or its custom angle ($A3:A1F3, $A3:A23E, $A3:A3CA).
fn calculate_velocities(runtime: &mut impl Host, x: impl Fn(i32) -> usize, adjustment: Option<u8>) {
    let key = combat::current(runtime);
    let roach = current_slot(runtime);
    for (axis_offset, cosine) in [(0, true), (4, false)] {
        let (angle, speed) = {
            let actor = runtime.resolve_actor(key, Native);
            let angle = match adjustment {
                Some(delta) => u16::from(actor.parameters[0].to_ne_bytes()[1].wrapping_add(delta)),
                None => actor.variables[lane::CUSTOM_ANGLE],
            };
            (angle, u16::from(actor.parameters[0].to_ne_bytes()[0]))
        };
        let value = if cosine {
            runtime.roach_cosine_full(angle, speed)
        } else {
            runtime.roach_sine_full(angle, speed)
        };
        store_velocity(runtime, x(roach) + axis_offset, value);
    }
}
/// Calculate the custom velocities and determine the custom instruction list
/// index ($A3:A3B5).
fn retarget(runtime: &mut impl Host, roach: i32) {
    calculate_velocities(runtime, custom_x, None);
    let key = combat::current(runtime);
    let current = current_slot(runtime);
    let angle = runtime.resolve_actor(key, Native).variables[lane::CUSTOM_ANGLE] as u8;
    session_slot!(
        runtime,
        Roach[current].custom_list_index = 2 * u16::from(angle.wrapping_sub(FORWARD_FACING) >> 5)
    );
    let index = session_slot!(runtime, Roach[roach].custom_list_index);
    face(runtime, roach, index);
}
/// Move the roach by the velocities at `x`.
fn translate(runtime: &mut impl Host, x: impl Fn(i32) -> usize) {
    let key = combat::current(runtime);
    let roach = current_slot(runtime);
    let dx = velocity(runtime, x(roach));
    runtime.edit_actor(key, Native, |actor| actor.x.advance(dx as i32));
    let dy = velocity(runtime, x(roach) + 4);
    runtime.edit_actor(key, Native, |actor| actor.y.advance(dy as i32));
}
/// Move roach forward ($A3:A578), along its angle with the forward speeds.
fn move_forward(runtime: &mut impl Host) {
    let key = combat::current(runtime);
    let roach = current_slot(runtime);
    let angle = u16::from(runtime.resolve_actor(key, Native).parameters[0].to_ne_bytes()[1]);
    let point = Vector32 {
        x: velocity(runtime, forward_x(roach)),
        y: velocity(runtime, forward_x(roach) + 4),
    };
    runtime.session_mut().compatibility_enemy_scratch = angle;
    api::apply_vector(runtime, angle, point);
}
/// Reverse roach custom angle ($A3:A52A).
fn reverse(runtime: &mut impl Host) {
    let roach = current_slot(runtime);
    let at = custom_x(roach);
    // Each native word is negated independently; this is not 32-bit negation.
    for offset in [2, 0, 6, 4] {
        runtime.set_slot_word(at + offset, runtime.slot_word(at + offset).wrapping_neg());
    }
    let index = session_slot!(runtime, Roach[roach].custom_list_index)
        .wrapping_add(HALF_TURN_FACINGS)
        & FACINGS;
    session_slot!(runtime, Roach[roach].custom_list_index = index);
    face(runtime, roach, index);
}
/// Initialisation AI - roach ($A3:A14D).
pub fn initialize(runtime: &mut impl Host) {
    let key = combat::current(runtime);
    let roach = current_slot(runtime);
    session_slot!(runtime, Roach[roach]; instruction_list = 0, new_instruction_list = 0);
    // Calculate moving forward speeds ($A3:A183).
    let live = current_slot(runtime);
    let parameter = runtime
        .resolve_actor(combat::current(runtime), Native)
        .parameters[0]
        .to_ne_bytes();
    let point = runtime.roach_direction_vector(u16::from(parameter[1]), u16::from(parameter[0]));
    store_velocity(runtime, forward_x(live), point.x);
    store_velocity(runtime, forward_x(live) + 4, point.y);
    calculate_velocities(runtime, left_x, Some(0u8.wrapping_sub(SIDE_ANGLE)));
    calculate_velocities(runtime, right_x, Some(SIDE_ANGLE));
    // Determine the facing forward, left and right instruction list indices
    // ($A3:A289, $A3:A29E, $A3:A2B7).
    for (left, adjustment) in [
        (None, FORWARD_FACING),
        (Some(true), LEFT_FACING),
        (Some(false), RIGHT_FACING),
    ] {
        let live = current_slot(runtime);
        let angle = runtime
            .resolve_actor(combat::current(runtime), Native)
            .parameters[0]
            .to_ne_bytes()[1];
        let index = 2 * u16::from(angle.wrapping_sub(adjustment) >> 5);
        match left {
            None => session_slot!(runtime, Roach[live].forward_list_index = index),
            Some(true) => session_slot!(runtime, Roach[live].left_list_index = index),
            Some(false) => session_slot!(runtime, Roach[live].right_list_index = index),
        }
    }
    let index = session_slot!(runtime, Roach[roach].forward_list_index);
    face(runtime, roach, index);
    runtime.edit_actor(key, Native, |actor| {
        actor.variables[lane::FUNCTION] = function::WAIT_FOR_SAMUS
    });
}
/// Main AI - roach ($A3:A2D0): run its function. Was `advance`.
pub fn main_ai(runtime: &mut impl Host) {
    let key = combat::current(runtime);
    let roach = current_slot(runtime);
    let function = runtime.resolve_actor(key, Native).variables[lane::FUNCTION];
    match function {
        // Roach function - wait for Samus to get near ($A3:A2D7).
        function::WAIT_FOR_SAMUS => {
            let mut near = true;
            for axis in [Axis::X, Axis::Y] {
                let current = combat::current(runtime);
                let distance =
                    u16::from(runtime.resolve_actor(key, Native).parameters[1].to_ne_bytes()[0]);
                if !api::player_within(runtime, current, axis, distance) {
                    near = false;
                    break;
                }
            }
            if near {
                let index =
                    runtime.resolve_actor(key, Native).parameters[1].to_ne_bytes()[1] as isize;
                let next = runtime.roach_table(RoachTable::ActivateFunctions, index);
                runtime.edit_actor(key, Native, |actor| actor.variables[lane::FUNCTION] = next);
            }
        }
        function::ACTIVATE_FORWARDS | function::ACTIVATE_ZIGZAG => {
            runtime.edit_actor(key, Native, |actor| {
                actor.variables[lane::FUNCTION] = if function == function::ACTIVATE_FORWARDS {
                    function::FORWARDS
                } else {
                    function::ZIGZAG
                }
            })
        }
        function::ACTIVATE_RANDOM | function::ACTIVATE_RANDOM_THEN_REVERSE => {
            if function == function::ACTIVATE_RANDOM_THEN_REVERSE {
                runtime.edit_actor(key, Native, |actor| {
                    actor.variables[lane::REVERSE_TIMER] = REVERSE_FRAMES
                });
            }
            runtime.session_mut().timing.random_number = RANDOM_SEED;
            runtime.refresh_inputs();
            let current = combat::current(runtime);
            runtime.edit_actor(current, Native, |actor| {
                actor.variables[lane::FUNCTION] = if function == function::ACTIVATE_RANDOM {
                    function::RANDOM
                } else {
                    function::RANDOM_THEN_REVERSE
                }
            });
        }
        function::ACTIVATE_FORWARDS_THEN_WAIT => runtime.edit_actor(key, Native, |actor| {
            actor.pre_instruction = MOVEMENT_FRAMES;
            actor.variables[lane::FUNCTION] = function::FORWARDS_THEN_WAIT;
        }),
        function::ACTIVATE_TOWARDS_SAMUS
        | function::ACTIVATE_AWAY_FROM_SAMUS
        | function::RANDOM_THEN_REVERSE
        | function::RANDOM => {
            let random = matches!(function, function::RANDOM_THEN_REVERSE | function::RANDOM);
            let angle = if random {
                runtime.roach_random() as u8
            } else {
                let current = combat::current(runtime);
                api::angle_to_player(runtime, current) as u8
            };
            let current = combat::current(runtime);
            let current_roach = current_slot(runtime);
            runtime.edit_actor(current, Native, |actor| {
                actor.variables[lane::CUSTOM_ANGLE] = if random {
                    actor.variables[lane::CUSTOM_ANGLE]
                        .wrapping_add(u16::from(angle.wrapping_sub(QUARTER_TURN)))
                } else {
                    u16::from(
                        (if function == function::ACTIVATE_TOWARDS_SAMUS {
                            QUARTER_TURN
                        } else {
                            THREE_QUARTERS
                        })
                        .wrapping_sub(angle),
                    )
                };
            });
            retarget(runtime, current_roach);
            runtime.edit_actor(current, Native, |actor| {
                if random {
                    actor.pre_instruction = MOVEMENT_FRAMES;
                }
                actor.variables[lane::FUNCTION] = match function {
                    function::ACTIVATE_TOWARDS_SAMUS => function::TOWARDS_SAMUS,
                    function::ACTIVATE_AWAY_FROM_SAMUS => function::AWAY_FROM_SAMUS,
                    function::RANDOM_THEN_REVERSE => function::CUSTOM_ANGLE_REVERSE,
                    _ => function::CUSTOM_ANGLE_WAIT,
                };
            });
        }
        function::FORWARDS => move_forward(runtime),
        // Roach function - activated - index 1 (move forwards in a zigzag) ($A3:A40E).
        function::ZIGZAG => {
            let second = runtime.resolve_actor(key, Native).frame & 0x10 != 0;
            let index = if second {
                session_slot!(runtime, Roach[roach].right_list_index)
            } else {
                session_slot!(runtime, Roach[roach].left_list_index)
            };
            face(runtime, roach, index);
            translate(runtime, if second { right_x } else { left_x });
        }
        function::TOWARDS_SAMUS | function::AWAY_FROM_SAMUS => translate(runtime, custom_x),
        function::FORWARDS_THEN_WAIT | function::CUSTOM_ANGLE_WAIT => {
            let expired = runtime.edit_actor(key, Native, |actor| {
                actor.pre_instruction = actor.pre_instruction.wrapping_sub(1);
                actor.pre_instruction & SIGN != 0
            });
            if expired {
                runtime.edit_actor(key, Native, |actor| {
                    actor.variables[lane::FUNCTION] = if function == function::FORWARDS_THEN_WAIT {
                        function::WAIT_FOR_SAMUS
                    } else {
                        function::RANDOM
                    }
                });
            } else if function == function::FORWARDS_THEN_WAIT {
                move_forward(runtime);
            } else {
                let current = combat::current(runtime);
                let current_roach = current_slot(runtime);
                let x = velocity(runtime, custom_x(current_roach)) as i32;
                let mut blocked = api::move_horizontal(runtime, current, x, SlopePolicy::Ignore)
                    == MovementResult::Blocked;
                if !blocked {
                    let y = velocity(runtime, custom_x(current_roach) + 4) as i32;
                    blocked = api::move_vertical(runtime, current, y) == MovementResult::Blocked;
                }
                if blocked {
                    runtime.edit_actor(current, Native, |actor| {
                        actor.variables[lane::FUNCTION] = function::WAIT_FOR_SAMUS
                    });
                }
            }
        }
        function::CUSTOM_ANGLE_REVERSE => {
            let expired = runtime.edit_actor(key, Native, |actor| {
                actor.variables[lane::REVERSE_TIMER] =
                    actor.variables[lane::REVERSE_TIMER].wrapping_sub(1);
                actor.variables[lane::REVERSE_TIMER] & SIGN != 0
            });
            if !expired {
                let expired = runtime.edit_actor(key, Native, |actor| {
                    actor.pre_instruction = actor.pre_instruction.wrapping_sub(1);
                    actor.pre_instruction & SIGN != 0
                });
                if expired {
                    runtime.edit_actor(key, Native, |actor| {
                        actor.variables[lane::FUNCTION] = function::RANDOM_THEN_REVERSE
                    });
                    return;
                }
                let far = [Axis::X, Axis::Y].into_iter().all(|axis| {
                    let actor = runtime.resolve_actor(key, Native);
                    let (position, player) = match axis {
                        Axis::X => (actor.x.whole, runtime.session().player.x),
                        Axis::Y => (actor.y.whole, runtime.session().player.y),
                    };
                    let distance = (position.wrapping_sub(player) as i16).wrapping_abs() as u16;
                    distance.wrapping_sub(REVERSE_DISTANCE) & SIGN == 0
                });
                if far {
                    reverse(runtime);
                }
            }
            translate(runtime, custom_x);
        }
        _ => runtime.unknown_roach_function(),
    }
}
