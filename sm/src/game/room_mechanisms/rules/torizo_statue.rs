//! Bomb Torizo statue release and the actor waiting for its removal.
#![forbid(unsafe_code)]
use super::super::{
    fields::{Fields, Global},
    state::Column,
};

pub fn setup(c: &mut impl Fields, slot: usize) -> u8 {
    if c.boss(4) != 0 {
        c.release(slot);
    }
    0
}
pub fn poll(c: &mut impl Fields, slot: usize) {
    if c.global(Global::CollectedItems) & 0x1000 != 0 {
        c.set_slot(Column::InstructionDelay, slot, 1);
        let instruction = c.slot(Column::Program, slot).wrapping_add(2);
        c.set_slot(Column::Program, slot, instruction);
        c.set_slot(Column::PreAction, slot, 0xd356);
    }
}

pub fn present(c: &impl Fields) -> bool {
    (0..40)
        .rev()
        .any(|slot| c.slot(Column::Header, slot) == 0xd6ea)
}
