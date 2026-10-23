//! Botwoon's delayed wall opening and the shared crumbling-row movement.
#![forbid(unsafe_code)]
use super::super::{fields::Fields, state::Column};

pub fn setup(c: &mut impl Fields, slot: usize) -> u8 {
    c.set_slot(Column::InstructionDelay, slot, 64);
    0
}
pub fn open_passage(c: &mut impl Fields) {
    c.native_scroll_pair(0, 0x101);
}
pub fn move_down(c: &mut impl Fields, slot: usize) {
    // The source captures the current block before observing the live row width.
    let block = c.slot(Column::Block, slot);
    let below = c.below(block);
    c.set_slot(Column::Block, slot, below);
}
