//! The roach, the Brinstar bug that scuttles at Samus in one of seven ways
//! (bank $A3:A051-$A3:A611).
use crate::actors::host::ActorLease;
use crate::actors::layouts::part_view;

part_view! {
    /// The roach's view of its slot and extended RAM. Each velocity is a
    /// subvelocity word then a velocity word.
    Roach: EnemyLayout72 {
        words: [reverse_timer, function, custom_angle, var_d, var_e, movement_timer, speed_and_angle, range_and_activation],
        pad1: [u8; 26696],
        words: [
            forward_x, forward_x_high, forward_y, forward_y_high, left_x, left_x_high, left_y,
            left_y_high, right_x, right_x_high, right_y, right_y_high, custom_x, custom_x_high,
            custom_y, custom_y_high, var_10, var_11, var_12, var_13, var_14, var_15, var_16, var_17,
            var_18, var_19, var_1a, var_1b, var_1c, var_1d, var_1e, var_1f,
        ],
        pad2: [u8; 1984],
        words: [
            forward_list_index, left_list_index, right_list_index, custom_list_index,
            new_instruction_list, instruction_list,
        ],
        _: [u16; 26],
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

#[path = "roach_owned.rs"]
pub mod owned;

pub fn initialize(runtime: &mut impl ActorLease) {
    owned::initialize(runtime);
}
pub fn main_ai(runtime: &mut impl ActorLease) {
    crate::probe::hit("roach.main_ai");
    owned::main_ai(runtime);
}
