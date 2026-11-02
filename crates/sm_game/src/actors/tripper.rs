//! The tripper and the suspensor platform, the floating platforms that sink
//! under Samus and rise again (bank $A3:9B7B-$A3:9F2F). Was
//! `falling_platforms`.
use crate::actors::host::ActorLease;
use crate::actors::layouts::part_view;
use crate::enemy_script::EnemyScript;

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

#[path = "tripper_owned.rs"]
pub mod owned;

pub fn initialize(suspensor: bool, runtime: &mut impl ActorLease) {
    owned::initialize(suspensor, runtime);
}
pub fn main_ai(runtime: &mut impl ActorLease) {
    owned::main_ai(runtime);
}
pub fn shot(runtime: &mut impl ActorLease) {
    owned::shot(runtime);
}
pub fn set_x_movement(
    right: bool,
    script: EnemyScript,
    runtime: &mut impl ActorLease,
) -> EnemyScript {
    owned::set_x_movement(right, script, runtime)
}
