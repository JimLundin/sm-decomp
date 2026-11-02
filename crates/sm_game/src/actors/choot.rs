//! Choot, which waits for Samus, jumps and drifts down in a falling pattern
//! (bank $A2:D80C-$A2:E17A). Was `rising_falling`.
use crate::actors::host::ActorLease;
use crate::actors::layouts::part_view;

part_view! {
    Choot: EnemyLayout40 {
        words: [function, y_speed_index, pattern_index, pattern_loops, pattern, pattern_y_distance, parameter_1, parameter_2],
        pad1: [u8; 26696],
        words: [spawn_x, spawn_y, falling_x, falling_y, origin_x, origin_y, initial_y_speed_index, jump_delay],
        _: [u16; 24],
    }
}

const WAIT_FOR_SAMUS: u16 = 0xe035;
const PREPARE_TO_JUMP: u16 = 0xe04f;
const JUMPING: u16 = 0xe06a;
const FALLING: u16 = 0xe0cd;
const IDLE_LIST: u16 = 0xd82c;
const JUMPING_LIST: u16 = 0xd834;
const FALLING_LIST: u16 = 0xd840;
/// $A2:DF5E, $A2:DF6A: the falling pattern pointers and Y distances, by pattern.
const PATTERN_END: u16 = 0x8000;
const SPEED_INDEX_STEP: u16 = 0x200;

#[path = "choot_owned.rs"]
pub mod owned;

pub fn initialize(runtime: &mut impl ActorLease) {
    owned::initialize(runtime);
}
pub fn main_ai(runtime: &mut impl ActorLease) {
    owned::main_ai(runtime);
}
