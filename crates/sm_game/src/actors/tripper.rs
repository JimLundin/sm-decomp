//! The tripper and the suspensor platform, the floating platforms that sink
//! under Samus and rise again (bank $A3:9B7B-$A3:9F2F). Was
//! `falling_platforms`.
use crate::actors::access::{AccessMode::Native, ActorKey as Key};
use crate::actors::api::{self, MovementResult, ShotHandling, SlopePolicy};
use crate::actors::combat::{self, current_slot};
use crate::actors::host::ActorLease;
use crate::actors::layouts::part_view;
use crate::enemy_script::EnemyScript;
use crate::session_slot;
use sm_data::enemy_speeds::SpeedTable;

part_view! {
    /// The platform's view of its slot.
    Platform: EnemyLayout40 {
        words: [
            target_y, right_subvelocity, right_velocity, left_subvelocity, left_velocity,
            y_speed_index, direction, speed,
        ],
        pad1: [u8; 26696],
        words: [
            y_movement_function, previous_position, x_movement_function, vertically_moving,
            vertically_still, max_y_speed_index, previous_y_movement_function, var_07, var_08,
            var_09, var_0a, var_0b, var_0c, var_0d, var_0e, var_0f, var_10, var_11, var_12, var_13,
            var_14, var_15, var_16, var_17, var_18, var_19, var_1a, var_1b, var_1c, var_1d, var_1e,
            suspensor_platform,
        ],
    }
}

/// The platforms' actor words, by `variables` lane; the Y speed table index
/// is the pre-instruction word.
mod lane {
    pub const TARGET_Y: usize = 0;
    pub const RIGHT_SUBVELOCITY: usize = 1;
    pub const LEFT_SUBVELOCITY: usize = 3;
}

/// Bank $A3 instruction lists: suspensor platform and tripper, vertically
/// moving or still, moving left; moving right is 16h further.
pub mod instruction_list {
    pub const SUSPENSOR_MOVING_LEFT: u16 = 0x9bbb;
    pub const SUSPENSOR_STILL_LEFT: u16 = 0x9be7;
    pub const TRIPPER_MOVING_LEFT: u16 = 0x9c13;
    pub const TRIPPER_STILL_LEFT: u16 = 0x9c3f;
    pub const RIGHT: u16 = 0x16;
}

/// Bank $A3 instructions: set moving left or right X movement.
pub mod instruction {}

/// The frozen tripper's spritemaps, facing left and right.
const FROZEN_LEFT: u16 = 0xa009;
const FROZEN_RIGHT: u16 = 0xa015;
const SIGN: u16 = 0x8000;

/// Set the instruction list for the platform's kind, motion and direction
/// ($A3:9E47-$A3:9E9E).
fn set_instruction_list(runtime: &mut impl ActorLease, moving: bool, right: bool) {
    let platform = current_slot(runtime);
    let suspensor = session_slot!(runtime, Platform[platform].suspensor_platform) & SIGN != 0;
    let instruction = [
        [
            instruction_list::TRIPPER_STILL_LEFT,
            instruction_list::TRIPPER_MOVING_LEFT,
        ],
        [
            instruction_list::SUSPENSOR_STILL_LEFT,
            instruction_list::SUSPENSOR_MOVING_LEFT,
        ],
    ][usize::from(suspensor)][usize::from(moving)]
        + u16::from(right) * instruction_list::RIGHT;
    let key = combat::current(runtime);
    runtime.edit_actor(key, Native, |actor| actor.restart_animation(instruction));
}
/// Set tripper / suspensor platform instruction list - vertically moving
/// ($A3:9EBB) or still ($A3:9EE1), once.
fn set_vertical_motion(runtime: &mut impl ActorLease, moving: bool) {
    let platform = current_slot(runtime);
    let flag = if moving {
        session_slot!(runtime, Platform[platform].vertically_moving)
    } else {
        session_slot!(runtime, Platform[platform].vertically_still)
    };
    if flag == 0 {
        if moving {
            session_slot!(runtime, Platform[platform].vertically_moving = 1);
        } else {
            session_slot!(runtime, Platform[platform].vertically_still = 1);
        }
        let right = session_slot!(runtime, Platform[platform].x_movement_function) != 0;
        set_instruction_list(runtime, moving, right);
    }
    if moving {
        session_slot!(runtime, Platform[platform].vertically_still = 0);
    } else {
        session_slot!(runtime, Platform[platform].vertically_moving = 0);
    }
}
/// Initialisation AI - suspensor platform ($A3:9C9F, `suspensor`) or tripper
/// ($A3:9CBA), and their common initialisation AI ($A3:9CCC).
pub fn initialize(suspensor: bool, runtime: &mut impl ActorLease) {
    let key = combat::current(runtime);
    let platform = current_slot(runtime);
    if suspensor {
        session_slot!(runtime, Platform[platform].suspensor_platform = 0xffff);
    }
    let direction = runtime.resolve_actor(key, Native).parameters[0];
    session_slot!(runtime, Platform[platform].x_movement_function = direction);
    // Initialization selects by kind; the fast variant retains its saved sign word.
    let instruction = if suspensor {
        instruction_list::SUSPENSOR_STILL_LEFT
    } else {
        instruction_list::TRIPPER_STILL_LEFT
    } + if direction != 0 {
        instruction_list::RIGHT
    } else {
        0
    };
    runtime.edit_actor(key, Native, |actor| actor.instruction = instruction);
    let index = 4 * isize::from(runtime.resolve_actor(key, Native).parameters[1].to_ne_bytes()[0]);
    // The right velocity, then the left, each whole word before its subvelocity.
    for (offset, variable) in [2, 1, 4, 3].into_iter().enumerate() {
        let value = runtime.lease_enemy_speed(SpeedTable::Linear, index + offset as isize);
        runtime.edit_actor(key, Native, |actor| actor.variables[variable] = value);
    }
    session_slot!(runtime, Platform[platform]; y_movement_function = 0, vertically_moving = 0,
        vertically_still = 0);
    runtime.edit_actor(key, Native, |actor| {
        actor.variables[lane::TARGET_Y] = actor.y.whole.wrapping_add(1);
        actor.pre_instruction = 0;
    });
    let maximum = runtime.resolve_actor(key, Native).parameters[1].to_ne_bytes()[1];
    session_slot!(
        runtime,
        Platform[platform].max_y_speed_index = u16::from(maximum)
    );
}
/// Increment the Y speed table index, up to the maximum.
fn accelerate(runtime: &mut impl ActorLease, key: Key) -> u16 {
    runtime.edit_actor(key, Native, |actor| {
        actor.pre_instruction = actor.pre_instruction.wrapping_add(1)
    });
    let platform = i32::from(key.byte_index());
    let maximum = session_slot!(runtime, Platform[platform].max_y_speed_index);
    runtime.edit_actor(key, Native, |actor| {
        if (actor.pre_instruction as i16).wrapping_sub(maximum as i16) >= 0 {
            actor.pre_instruction = maximum;
        }
        actor.pre_instruction
    })
}
/// Tripper / suspensor platform X movement function - moving left ($A3:9D5E)
/// or right ($A3:9D83): turn round at walls.
fn move_x(runtime: &mut impl ActorLease, right: bool) {
    let key = combat::current(runtime);
    let platform = current_slot(runtime);
    let x = runtime.resolve_actor(key, Native).x.whole;
    session_slot!(runtime, Platform[platform].previous_position = x);
    let current = combat::current(runtime);
    let amount = {
        let actor = runtime.resolve_actor(key, Native);
        let low = if right {
            lane::RIGHT_SUBVELOCITY
        } else {
            lane::LEFT_SUBVELOCITY
        };
        ((u32::from(actor.variables[low + 1]) << 16) | u32::from(actor.variables[low])) as i32
    };
    if api::move_horizontal(runtime, current, amount, SlopePolicy::Ignore)
        == MovementResult::Blocked
    {
        session_slot!(
            runtime,
            Platform[platform].x_movement_function = u16::from(!right)
        );
        set_instruction_list(runtime, false, !right);
    }
}
/// Tripper / suspensor platform Y movement function - rising ($A3:9DA8) back
/// to the target Y, or sinking ($A3:9DE4) under Samus, carrying her.
fn move_y(runtime: &mut impl ActorLease, sinking: bool) {
    let key = combat::current(runtime);
    let platform = current_slot(runtime);
    let mut stopped = !sinking && {
        let actor = runtime.resolve_actor(key, Native);
        (actor.y.whole as i16).wrapping_sub(actor.variables[lane::TARGET_Y] as i16) < 0
    };
    if !stopped {
        if !sinking {
            set_vertical_motion(runtime, true);
        }
        let step = accelerate(runtime, key);
        if sinking {
            let delta = runtime
                .resolve_actor(key, Native)
                .x
                .whole
                .wrapping_sub(session_slot!(runtime, Platform[platform].previous_position));
            let carry = &mut runtime.session_mut().player.contact.external_x.whole;
            *carry = carry.wrapping_add(delta);
            let y = runtime.resolve_actor(key, Native).y.whole;
            session_slot!(runtime, Platform[platform].previous_position = y);
        }
        let current = combat::current(runtime);
        let index = 4 * step as isize + if sinking { 0 } else { 2 };
        let high = runtime.lease_enemy_speed(SpeedTable::Quadratic, index + 1);
        let low = runtime.lease_enemy_speed(SpeedTable::Quadratic, index);
        let amount = ((u32::from(high) << 16) | u32::from(low)) as i32;
        stopped = api::move_vertical(runtime, current, amount) == MovementResult::Blocked;
    }
    if stopped {
        runtime.edit_actor(key, Native, |actor| actor.pre_instruction = 0);
        set_vertical_motion(runtime, false);
    }
    if sinking {
        let delta = runtime
            .resolve_actor(key, Native)
            .y
            .whole
            .wrapping_sub(session_slot!(runtime, Platform[platform].previous_position));
        let carry = &mut runtime.session_mut().player.contact.external_y.whole;
        *carry = carry.wrapping_add(delta);
    }
}
/// Main AI - tripper / suspensor platform ($A3:9D16): sink while Samus stands
/// on it, else rise, moving sideways; a change of direction restarts the
/// speed. Was `advance`.
pub fn main_ai(runtime: &mut impl ActorLease) {
    let key = combat::current(runtime);
    let platform = current_slot(runtime);
    session_slot!(runtime, Platform[platform].y_movement_function = 0);
    if api::touches_player_from_above(runtime, combat::current(runtime)) {
        session_slot!(runtime, Platform[platform].y_movement_function = 1);
    }
    // Full native dispatch indices keep both original two-entry bounds checks.
    let right = [false, true][usize::from(session_slot!(
        runtime,
        Platform[platform].x_movement_function
    ))];
    move_x(runtime, right);
    let current = current_slot(runtime);
    let sinking = [false, true][usize::from(session_slot!(
        runtime,
        Platform[current].y_movement_function
    ))];
    move_y(runtime, sinking);
    if session_slot!(runtime, Platform[platform].y_movement_function)
        != session_slot!(runtime, Platform[platform].previous_y_movement_function)
    {
        runtime.edit_actor(key, Native, |actor| actor.pre_instruction = 0);
    }
    let function = session_slot!(runtime, Platform[platform].y_movement_function);
    session_slot!(
        runtime,
        Platform[platform].previous_y_movement_function = function
    );
}
/// Enemy shot - tripper ($A3:9F08): frozen, it keeps its facing spritemap.
pub fn shot(runtime: &mut impl ActorLease) {
    combat::shot_reaction(runtime, ShotHandling::Finalize);
    let key = combat::current(runtime);
    if runtime.resolve_actor(key, Native).frozen != 0 {
        let platform = current_slot(runtime);
        let right = session_slot!(runtime, Platform[platform].x_movement_function) != 0;
        runtime.edit_actor(key, Native, |actor| {
            actor.spritemap = if right { FROZEN_RIGHT } else { FROZEN_LEFT }
        });
    }
}
/// Instruction - set moving left or right X movement ($A3:9C6B-$A3:9C8C).
/// Was `direction`.
pub fn set_x_movement(
    right: bool,
    script: EnemyScript,
    runtime: &mut impl ActorLease,
) -> EnemyScript {
    let platform = current_slot(runtime);
    session_slot!(
        runtime,
        Platform[platform].x_movement_function = u16::from(right)
    );
    script
}
