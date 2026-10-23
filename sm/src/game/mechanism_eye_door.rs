//! Eye-door placement and transition after its persistent open flag is set.
#![forbid(unsafe_code)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Part {
    Eye,
    Body,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Side {
    Right,
    Left,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tile {
    Eye,
    Body,
    Blue(Side),
    Extension(u8),
}
pub trait Context {
    type Block: Copy;
    type Row: Copy;
    type Door;
    type Program;
    fn door(&self, slot: usize) -> Self::Door;
    fn opened(&mut self, door: Self::Door) -> bool;
    fn block(&self, slot: usize) -> Self::Block;
    fn set_block(&mut self, slot: usize, block: Self::Block);
    fn row(&self) -> Self::Row;
    fn above(&self, block: Self::Block, row: Self::Row) -> Self::Block;
    fn below(&self, block: Self::Block, row: Self::Row) -> Self::Block;
    fn install(&mut self, block: Self::Block, tile: Tile);
    fn stop_watching(&mut self, slot: usize);
    fn link(&self, slot: usize) -> Self::Program;
    fn set_program(&mut self, slot: usize, program: Self::Program);
    fn set_timer(&mut self, slot: usize, timer: u16);
}
pub fn begin(c: &mut impl Context, slot: usize, part: Part) -> u8 {
    if !c.opened(c.door(slot)) {
        c.install(
            c.block(slot),
            if part == Part::Eye {
                Tile::Eye
            } else {
                Tile::Body
            },
        );
        if part == Part::Eye {
            let row = c.row();
            let block = c.block(slot);
            c.install(c.below(block, row), Tile::Extension(1));
        }
    }
    0
}
pub fn wake(c: &mut impl Context, slot: usize) {
    if c.opened(c.door(slot)) {
        c.stop_watching(slot);
        c.set_program(slot, c.link(slot));
        c.set_timer(slot, 1);
    }
}
pub fn convert(c: &mut impl Context, slot: usize, side: Side) {
    let old = c.block(slot);
    let row = c.row();
    c.set_block(slot, c.above(old, row));
    let mut block = c.block(slot);
    c.install(block, Tile::Blue(side));
    for distance in 1..=3 {
        block = c.below(block, c.row());
        c.install(block, Tile::Extension(distance));
    }
}
