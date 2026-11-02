//! Zoa, Maridia's refill creatures, which rise towards Samus from the sand,
//! shoot off sideways and return off screen (bank $A3:B3A1-$A3:B52F). Was
//! `refill_candy`.
use crate::actors::host::ActorLease;
use crate::enemy_script::EnemyScript;

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
pub mod instruction {}

/// $A3:B3C1: facing left, shooting; $A3:B40D: the instruction list for each
/// instruction list index; $A3:B415: the X speeds, by X speed table index.
const LEFT_SHOOTING: u16 = 0xb3c1;
/// $7E:7800: Zoa's X speed table index.
/// Instruction list indices: facing left or right, rising (shooting is one less).
const LEFT_RISING: u16 = 1;
const RIGHT_RISING: u16 = 3;
/// Samus is near within 80h px columns; Zoa rises at 0.8000h px a frame.
const NEAR_COLUMNS: u16 = 0x80;
const RISE: i32 = -0x8000;
/// Properties bit 8: invisible.
const INVISIBLE: u16 = 0x100;
const SIGN: u16 = 0x8000;

#[path = "zoa_owned.rs"]
pub mod owned;

pub fn initialize(runtime: &mut impl ActorLease) {
    owned::initialize(runtime);
}
pub fn main_ai(runtime: &mut impl ActorLease) {
    owned::main_ai(runtime);
}
pub fn set_x_speed_index(
    value: u16,
    script: EnemyScript,
    runtime: &mut impl ActorLease,
) -> EnemyScript {
    owned::set_x_speed_index(value, script, runtime)
}
