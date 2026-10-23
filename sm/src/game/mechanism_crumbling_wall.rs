//! Botwoon's delayed wall opening and the shared crumbling-row movement.
#![forbid(unsafe_code)]
pub trait Context {
    type Block;
    fn set_delay(&mut self, slot: usize, frames: u16);
    fn open_passage(&mut self);
    fn block(&self, slot: usize) -> Self::Block;
    fn below(&self, block: Self::Block) -> Self::Block;
    fn set_block(&mut self, slot: usize, block: Self::Block);
}
pub fn setup(c: &mut impl Context, slot: usize) -> u8 {
    c.set_delay(slot, 64);
    0
}
pub fn open_passage(c: &mut impl Context) {
    c.open_passage();
}
pub fn move_down(c: &mut impl Context, slot: usize) {
    // The source captures the current block before observing the live row width.
    let block = c.block(slot);
    let below = c.below(block);
    c.set_block(slot, below);
}
