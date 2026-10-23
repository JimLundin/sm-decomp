//! Delay sound effects after room entry, then retire the gate's mechanism.
#![forbid(unsafe_code)]
use super::super::access::*;
use super::super::host::*;

pub fn begin(c: &mut impl Host, slot: usize) -> u8 {
    let delay = if c.session().room.area == 6 { 32 } else { 240 };
    mechanism!(c, slot, counter = delay);
    mechanism!(c, slot, pre_action = 0xb7dd);
    0
}
pub fn poll(c: &mut impl Host, slot: usize) {
    let old = mechanism!(c, slot, counter);
    let remaining = mechanism!(c, slot, counter).wrapping_sub(1);
    mechanism!(c, slot, counter = remaining);
    if old == 1 {
        c.session_mut().feedback.sound_disabled = 0;
        mechanism!(c, slot, header = 0);
    }
}
