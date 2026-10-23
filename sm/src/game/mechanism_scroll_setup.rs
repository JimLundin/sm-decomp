//! Extension placement and scroll-mechanism activation.
#![forbid(unsafe_code)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    Right,
    Left,
    Down,
    Up,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Solidity {
    Air,
    Solid,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tile {
    Extension(Direction),
    Scroll(Solidity),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Setup {
    Place(Tile),
    Touch,
}
pub trait Context {
    type Block: Copy + Eq;
    fn block(&self, slot: usize) -> Self::Block;
    fn clear_block(&mut self, slot: usize);
    fn release(&mut self, slot: usize);
    fn install(&mut self, block: Self::Block, tile: Tile);
    fn active(&self, slot: usize) -> bool;
    fn set_active(&mut self, slot: usize, active: bool);
    fn advance(&mut self, slot: usize, bytes: u16);
    fn set_timer(&mut self, slot: usize);
}
pub fn setup(c: &mut impl Context, slot: usize, kind: Setup) -> u8 {
    match kind {
        Setup::Place(tile) => {
            c.install(c.block(slot), tile);
            match tile {
                Tile::Extension(_) => c.release(slot),
                Tile::Scroll(_) => {
                    c.set_active(slot, false);
                    c.advance(slot, 4);
                }
            }
        }
        Setup::Touch => {
            let block = c.block(slot);
            c.clear_block(slot);
            let Some(found) = (0..40).rev().find(|&found| c.block(found) == block) else {
                // Preserve the missing-scroll-target nonreturn after clearing the trigger block.
                loop {}
            };
            if !c.active(found) {
                c.set_active(found, true);
                c.advance(found, 1);
                c.advance(found, 1);
                c.set_timer(found);
            }
        }
    }
    0
}
