//! Animal rescue wall, escape passage and the rescued-event door gate.
#![forbid(unsafe_code)]
use super::super::access::*;
use super::super::host::*;
use super::terrain_setup as terrain;
use crate::progress as flags;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Setup {
    RescueWall,
    MainStreetPassage,
}

pub fn setup(c: &mut impl Host, slot: usize, kind: Setup) -> u8 {
    match kind {
        Setup::RescueWall => {
            let block = mechanism!(c, slot, block);
            terrain::install(c, block, 0xc04f);
            let below = (block).wrapping_add(c.session().room.width.wrapping_mul(2));
            terrain::install(c, below, 0xd0ff);
            let below = (below).wrapping_add(c.session().room.width.wrapping_mul(2));
            terrain::install(c, below, 0xd0ff);
        }
        Setup::MainStreetPassage => {
            if !flags::event_happened(c, 0xf) {
                mechanism!(c, slot, header = 0)
            }
        }
    }
    0
}
pub fn mark_rescued(c: &mut impl Host) {
    flags::set_event(c, 0xf)
}
pub fn move_passage(c: &mut impl Host, slot: usize) {
    let block = mechanism!(c, slot, block);
    let next = block.wrapping_add(8);
    mechanism!(c, slot, block = next)
}
pub fn poll_rescue_gate(c: &mut impl Host, slot: usize) {
    if flags::event_happened(c, 0xf) {
        mechanism!(c, slot, counter = 0);
        let link = mechanism!(c, slot, return_program);
        mechanism!(c, slot, program = link);
        mechanism!(c, slot, instruction_delay = 1);
    } else {
        if mechanism!(c, slot, counter) != 0 {
            c.sound(SoundChannel::Two, 6, 0x57)
        }
        mechanism!(c, slot, counter = 0);
    }
}
