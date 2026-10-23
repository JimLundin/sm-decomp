//! Open or close the two invisible ten-row barriers around the Tourian Metroid.
#![forbid(unsafe_code)]
use super::super::{fields::Fields, state::Column};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Setup {
    Open,
    Close,
}

pub fn setup(c: &mut impl Fields, slot: usize, kind: Setup) -> u8 {
    let mut block = c.slot(Column::Block, slot);
    for _ in 0..10 {
        let level = c.level(block);
        c.set_level(
            block,
            level & 0xfff | if kind == Setup::Close { 0x8000 } else { 0 },
        );
        // The original observes room width even after the final terrain write.
        block = c.below(block);
    }
    0
}
