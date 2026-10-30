//! The roach, the Brinstar bug that scuttles at Samus in one of seven ways
//! (bank $A3:A051-$A3:A611).
#![forbid(unsafe_code)]
use crate::actors::access::{AccessMode::Native, ActorAccess};
use crate::actors::api::{self, Axis, MovementResult, SlopePolicy};
use crate::actors::combat::{self, current_slot};
use crate::actors::host::{ActorLease, BossEffects as _, LeaseActors};
use crate::actors::layouts::part_view;
use crate::actors::session_slots::SlotAccess as _;
use crate::enemy_projectiles::host::Effects as _;
use crate::geometry::Vector32;
use crate::player_projectiles::content::table;
use crate::session_access::SessionAccess as _;
use crate::session_slot;

part_view! {
    /// The roach's view of its slot and extended RAM. Each velocity is a
    /// subvelocity word then a velocity word.
    Roach: EnemyLayout72 {
        words: [reverse_timer, function, custom_angle, var_D, var_E, movement_timer, speed_and_angle, range_and_activation],
        pad1: [u8; 26696],
        words: [
            forward_x, forward_x_high, forward_y, forward_y_high, left_x, left_x_high, left_y,
            left_y_high, right_x, right_x_high, right_y, right_y_high, custom_x, custom_x_high,
            custom_y, custom_y_high, var_10, var_11, var_12, var_13, var_14, var_15, var_16, var_17,
            var_18, var_19, var_1A, var_1B, var_1C, var_1D, var_1E, var_1F,
        ],
        pad2: [u8; 1984],
        words: [
            forward_list_index, left_list_index, right_list_index, custom_list_index,
            new_instruction_list, instruction_list, var_26, var_27, var_28, var_29, var_2A, var_2B,
            var_2C, var_2D, var_2E, var_2F, var_30, var_31, var_32, var_33, var_34, var_35, var_36,
            var_37, var_38, var_39, var_3A, var_3B, var_3C, var_3D, var_3E, var_3F,
        ],
    }
}

/// The roach's actor words, by `variables` lane; its movement timer is the
/// pre-instruction word.
mod lane {
    pub const REVERSE_TIMER: usize = 0;
    pub const FUNCTION: usize = 1;
    pub const CUSTOM_ANGLE: usize = 2;
}

/// Bank $A3 roach functions, stored in `function`.
pub mod function {
    pub const WAIT_FOR_SAMUS: u16 = 0xa2d7;
    pub const ACTIVATE_FORWARDS: u16 = 0xa301;
    pub const ACTIVATE_ZIGZAG: u16 = 0xa30b;
    pub const ACTIVATE_RANDOM: u16 = 0xa315;
    pub const ACTIVATE_RANDOM_THEN_REVERSE: u16 = 0xa325;
    pub const ACTIVATE_FORWARDS_THEN_WAIT: u16 = 0xa33b;
    pub const ACTIVATE_TOWARDS_SAMUS: u16 = 0xa34b;
    pub const ACTIVATE_AWAY_FROM_SAMUS: u16 = 0xa380;
    pub const FORWARDS: u16 = 0xa407;
    pub const ZIGZAG: u16 = 0xa40e;
    pub const TOWARDS_SAMUS: u16 = 0xa440;
    pub const AWAY_FROM_SAMUS: u16 = 0xa447;
    pub const FORWARDS_THEN_WAIT: u16 = 0xa44e;
    pub const CUSTOM_ANGLE_WAIT: u16 = 0xa462;
    pub const CUSTOM_ANGLE_REVERSE: u16 = 0xa476;
    pub const RANDOM_THEN_REVERSE: u16 = 0xa4b6;
    pub const RANDOM: u16 = 0xa4f0;
}

/// $A3:A111: the roach's instruction lists by facing; $A3:A121: its
/// activate functions, by its activation index.
const INSTRUCTION_LISTS: u32 = 0xa3_a111;
const ACTIVATE_FUNCTIONS: u32 = 0xa3_a121;
/// The left and right velocities are 20h either side of forwards; the
/// facing indices are taken 30h, 50h and 10h back.
const SIDE_ANGLE: u8 = 32;
const FORWARD_FACING: u8 = 48;
const LEFT_FACING: u8 = 80;
const RIGHT_FACING: u8 = 16;
/// Moving forwards then waiting, or randomly, lasts 20h frames; the reverse
/// timer starts at 200h, and the random draw is seeded with 11.
const MOVEMENT_FRAMES: u16 = 32;
const REVERSE_FRAMES: u16 = 512;
const RANDOM_SEED: u16 = 11;
/// The roach reverses once Samus is 6 blocks away on both axes.
const REVERSE_DISTANCE: u16 = 96;
/// Angles are a byte from the top; towards Samus is 40h less, away C0h less.
const QUARTER_TURN: u8 = 64;
const THREE_QUARTERS: u8 = 192;
/// Facings: 8 of 20h each; reversing turns 4.
const HALF_TURN_FACINGS: u16 = 4;
const FACINGS: u16 = 7;
const SIGN: u16 = 0x8000;

/// The 16.16 velocity whose subvelocity word is at `at`.
fn velocity(runtime: &impl ActorLease, at: usize) -> u32 {
    (u32::from(runtime.slot_word(at + 2)) << 16) | u32::from(runtime.slot_word(at))
}
fn store_velocity(runtime: &mut impl ActorLease, at: usize, value: u32) {
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
fn set_instruction_list(runtime: &mut impl ActorLease) {
    let roach = current_slot(runtime);
    let instruction = session_slot!(runtime, Roach[roach].new_instruction_list);
    if instruction != session_slot!(runtime, Roach[roach].instruction_list) {
        session_slot!(runtime, Roach[roach].instruction_list = instruction);
        let key = combat::current(runtime);
        runtime.edit_actor(key, Native, |actor| actor.restart_animation(instruction));
    }
}
fn face(runtime: &mut impl ActorLease, roach: i32, index: u16) {
    let instruction = table(runtime, INSTRUCTION_LISTS, (index >> 1) as isize);
    session_slot!(runtime, Roach[roach].new_instruction_list = instruction);
    set_instruction_list(runtime);
}
/// Calculate the X and Y velocities at `velocities` for the roach's angle,
/// adjusted, or its custom angle ($A3:A1F3, $A3:A23E, $A3:A3CA).
fn calculate_velocities(
    runtime: &mut impl ActorLease,
    x: impl Fn(i32) -> usize,
    adjustment: Option<u8>,
) {
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
        let value = runtime.foreign(|game| {
            if cosine {
                game.actor_cosine_full(angle, speed)
            } else {
                game.actor_sine_full(angle, speed)
            }
        });
        store_velocity(runtime, x(roach) + axis_offset, value);
    }
}
/// Calculate the custom velocities and determine the custom instruction list
/// index ($A3:A3B5).
fn retarget(runtime: &mut impl ActorLease, roach: i32) {
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
fn translate(runtime: &mut impl ActorLease, x: impl Fn(i32) -> usize) {
    let key = combat::current(runtime);
    let roach = current_slot(runtime);
    let dx = velocity(runtime, x(roach));
    runtime.edit_actor(key, Native, |actor| actor.x.advance(dx as i32));
    let dy = velocity(runtime, x(roach) + 4);
    runtime.edit_actor(key, Native, |actor| actor.y.advance(dy as i32));
}
/// Move roach forward ($A3:A578), along its angle with the forward speeds.
fn move_forward(runtime: &mut impl ActorLease) {
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
fn reverse(runtime: &mut impl ActorLease) {
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
pub fn initialize<H: LeaseActors + ?Sized>(game: &mut H) {
    let mut lease = game.lease();
    let runtime = &mut lease;
    let key = combat::current(runtime);
    let roach = current_slot(runtime);
    session_slot!(runtime, Roach[roach]; instruction_list = 0, new_instruction_list = 0);
    // Calculate moving forward speeds ($A3:A183).
    let live = current_slot(runtime);
    let parameter = runtime
        .resolve_actor(combat::current(runtime), Native)
        .parameters[0]
        .to_ne_bytes();
    let point = runtime.convert_angle_to_xy(u16::from(parameter[1]), u16::from(parameter[0]));
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
pub fn main_ai<H: LeaseActors + ?Sized>(game: &mut H) {
    let mut lease = game.lease();
    let runtime = &mut lease;
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
                let next = table(runtime, ACTIVATE_FUNCTIONS, index);
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
            runtime.write_slice_le(crate::ram::words::random_number, RANDOM_SEED);
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
                runtime.next_random() as u8
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
        _ => runtime.unreachable(),
    }
}
