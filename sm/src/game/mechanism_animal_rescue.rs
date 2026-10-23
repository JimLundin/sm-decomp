//! Animal rescue wall, escape passage and the rescued-event door gate.
#![forbid(unsafe_code)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Setup {
    RescueWall,
    MainStreetPassage,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Terrain {
    RescueShot,
    UpExtension,
}
pub trait Context {
    type Block: Copy;
    fn block(&self, slot: usize) -> Self::Block;
    fn below(&self, block: Self::Block) -> Self::Block;
    fn four_right(&self, block: Self::Block) -> Self::Block;
    fn install(&mut self, block: Self::Block, terrain: Terrain);
    fn set_block(&mut self, slot: usize, block: Self::Block);
    fn rescued(&mut self) -> bool;
    fn mark_rescued(&mut self);
    fn release(&mut self, slot: usize);
    fn timer(&self, slot: usize) -> u16;
    fn set_timer(&mut self, slot: usize, value: u16);
    fn link(&self, slot: usize) -> u16;
    fn set_program(&mut self, slot: usize, value: u16);
    fn set_instruction_timer(&mut self, slot: usize, value: u16);
    fn dud(&mut self);
}
pub fn setup(c: &mut impl Context, slot: usize, kind: Setup) -> u8 {
    match kind {
        Setup::RescueWall => {
            let block = c.block(slot);
            c.install(block, Terrain::RescueShot);
            let below = c.below(block);
            c.install(below, Terrain::UpExtension);
            let below = c.below(below);
            c.install(below, Terrain::UpExtension);
        }
        Setup::MainStreetPassage => {
            if !c.rescued() {
                c.release(slot)
            }
        }
    }
    0
}
pub fn mark_rescued(c: &mut impl Context) {
    c.mark_rescued()
}
pub fn move_passage(c: &mut impl Context, slot: usize) {
    let block = c.block(slot);
    let next = c.four_right(block);
    c.set_block(slot, next)
}
pub fn poll_rescue_gate(c: &mut impl Context, slot: usize) {
    if c.rescued() {
        c.set_timer(slot, 0);
        let link = c.link(slot);
        c.set_program(slot, link);
        c.set_instruction_timer(slot, 1);
    } else {
        if c.timer(slot) != 0 {
            c.dud()
        }
        c.set_timer(slot, 0);
    }
}
