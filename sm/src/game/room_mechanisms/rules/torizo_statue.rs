//! Bomb Torizo statue release and the actor waiting for its removal.
#![forbid(unsafe_code)]
use super::super::access::*;
use super::super::host::*;
use crate::game::progress_flags as flags;

pub fn setup(c: &mut impl Host, slot: usize) -> u8 {
    if flags::boss_defeated(c, 4) {
        mechanism!(c, slot, header = 0);
    }
    0
}
pub fn poll(c: &mut impl Host, slot: usize) {
    if c.session().inventory.equipment.owned & 0x1000 != 0 {
        mechanism!(c, slot, instruction_delay = 1);
        let instruction = mechanism!(c, slot, program).wrapping_add(2);
        mechanism!(c, slot, program = instruction);
        mechanism!(c, slot, pre_action = 0xd356);
    }
}

pub fn present(c: &impl Host) -> bool {
    (0..40)
        .rev()
        .any(|slot| mechanism!(c, slot, header) == 0xd6ea)
}
