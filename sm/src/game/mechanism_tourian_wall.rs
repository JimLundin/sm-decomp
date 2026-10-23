//! Open or close the two invisible ten-row barriers around the Tourian Metroid.
#![forbid(unsafe_code)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Setup {
    Open,
    Close,
}
pub trait Context {
    type Block: Copy;
    fn block(&self, slot: usize) -> Self::Block;
    fn level(&self, block: Self::Block) -> u16;
    fn set_level(&mut self, block: Self::Block, value: u16);
    fn below(&self, block: Self::Block) -> Self::Block;
}
pub fn setup(c: &mut impl Context, slot: usize, kind: Setup) -> u8 {
    let mut block = c.block(slot);
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
