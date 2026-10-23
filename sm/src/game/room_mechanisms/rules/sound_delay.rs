//! Delay sound effects after room entry, then retire the gate's mechanism.
#![forbid(unsafe_code)]
use super::super::{
    fields::{Fields, Global},
    state::Column,
};

pub fn begin(c: &mut impl Fields, slot: usize) -> u8 {
    let delay = if c.global(Global::Area) == 6 { 32 } else { 240 };
    c.set_slot(Column::Counter, slot, delay);
    c.set_slot(Column::PreAction, slot, 0xb7dd);
    0
}
pub fn poll(c: &mut impl Fields, slot: usize) {
    let old = c.slot(Column::Counter, slot);
    let remaining = c.slot(Column::Counter, slot).wrapping_sub(1);
    c.set_slot(Column::Counter, slot, remaining);
    if old == 1 {
        c.set_global(Global::SoundDisabled, 0);
        c.release(slot);
    }
}
