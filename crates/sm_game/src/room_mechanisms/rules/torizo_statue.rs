//! Bomb Torizo statue release and the actor waiting for its removal.
use super::super::access::*;
use super::super::host::*;
use crate::progress as flags;

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

/// Hold Torizo actor `index` waiting while the statue stands; once it has
/// crumbled, queue music track 6 and release the actor to its next
/// instruction.
pub fn wait_for_removal(c: &mut impl Host, index: u16) {
    c.statue_wait_actor(index, true);
    if present(c) {
        return;
    }
    c.music(6);
    c.statue_wait_actor(index, false);
}
