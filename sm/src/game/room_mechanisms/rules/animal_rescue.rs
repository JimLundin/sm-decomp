//! Animal rescue wall, escape passage and the rescued-event door gate.
#![forbid(unsafe_code)]
use super::super::{
    fields::{Fields, SoundChannel},
    state::Column,
};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Setup {
    RescueWall,
    MainStreetPassage,
}

pub fn setup(c: &mut impl Fields, slot: usize, kind: Setup) -> u8 {
    match kind {
        Setup::RescueWall => {
            let block = c.slot(Column::Block, slot);
            c.install(block, 0xc04f);
            let below = c.below(block);
            c.install(below, 0xd0ff);
            let below = c.below(below);
            c.install(below, 0xd0ff);
        }
        Setup::MainStreetPassage => {
            if !c.event(0xf) {
                c.release(slot)
            }
        }
    }
    0
}
pub fn mark_rescued(c: &mut impl Fields) {
    c.mark_event(0xf)
}
pub fn move_passage(c: &mut impl Fields, slot: usize) {
    let block = c.slot(Column::Block, slot);
    let next = block.wrapping_add(8);
    c.set_slot(Column::Block, slot, next)
}
pub fn poll_rescue_gate(c: &mut impl Fields, slot: usize) {
    if c.event(0xf) {
        c.set_slot(Column::Counter, slot, 0);
        let link = c.slot(Column::ReturnProgram, slot);
        c.set_slot(Column::Program, slot, link);
        c.set_slot(Column::InstructionDelay, slot, 1);
    } else {
        if c.slot(Column::Counter, slot) != 0 {
            c.sound(SoundChannel::Two, 6, 0x57)
        }
        c.set_slot(Column::Counter, slot, 0);
    }
}
