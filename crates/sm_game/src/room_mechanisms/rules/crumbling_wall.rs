//! Botwoon's delayed wall opening and the shared crumbling-row movement.
use super::super::access::*;
use super::super::host::*;

pub fn setup(c: &mut impl Host, slot: usize) -> u8 {
    mechanism!(c, slot, instruction_delay = 64);
    0
}
pub fn open_passage(c: &mut impl Host) {
    c.scroll_pair(0, 0x101);
}
pub fn move_down(c: &mut impl Host, slot: usize) {
    // The source captures the current block before observing the live row width.
    let block = mechanism!(c, slot, block);
    let below = (block).wrapping_add(c.session().room.width.wrapping_mul(2));
    mechanism!(c, slot, block = below);
}
