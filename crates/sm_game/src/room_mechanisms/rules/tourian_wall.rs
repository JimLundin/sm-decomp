//! Open or close the two invisible ten-row barriers around the Tourian Metroid.
use super::super::access::*;
use super::super::host::*;
use super::terrain_setup as terrain;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Setup {
    Open,
    Close,
}

pub fn setup(c: &mut impl Host, slot: usize, kind: Setup) -> u8 {
    let mut block = mechanism!(c, slot, block);
    for _ in 0..10 {
        let level = terrain::level(c, block);
        terrain::set_level(
            c,
            block,
            level & 0xfff | if kind == Setup::Close { 0x8000 } else { 0 },
        );
        // The original observes room width even after the final terrain write.
        block = (block).wrapping_add(c.session().room.width.wrapping_mul(2));
    }
    0
}
