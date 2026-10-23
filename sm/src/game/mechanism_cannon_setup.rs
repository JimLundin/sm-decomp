//! Draygon cannon damage linkage and initial collision shape.
#![forbid(unsafe_code)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Setup {
    Right,
    Left,
    DiagonalRight,
    DiagonalLeft,
    Unshielded,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Side {
    Right,
    Left,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tile {
    Shield,
    Horizontal(Side),
    Below,
}
pub trait Context {
    type Block: Copy;
    type DamageLink;
    fn damage_link(&self, slot: usize) -> Self::DamageLink;
    fn save_damage_link(&mut self, slot: usize, link: Self::DamageLink);
    fn set_hits(&mut self, slot: usize, hits: u16);
    fn block(&self, slot: usize) -> Self::Block;
    fn adjacent(&self, block: Self::Block, side: Side) -> Self::Block;
    fn below(&self, block: Self::Block) -> Self::Block;
    fn install(&mut self, block: Self::Block, tile: Tile);
}
pub fn setup(c: &mut impl Context, slot: usize, kind: Setup) -> u8 {
    c.save_damage_link(slot, c.damage_link(slot));
    c.set_hits(slot, if kind == Setup::Unshielded { 3 } else { 0 });
    if kind != Setup::Unshielded {
        let block = c.block(slot);
        c.install(block, Tile::Shield);
        let side = match kind {
            Setup::DiagonalRight => Some(Side::Right),
            Setup::DiagonalLeft => Some(Side::Left),
            _ => None,
        };
        if let Some(side) = side {
            c.install(c.adjacent(block, side), Tile::Horizontal(side));
        }
        let bottom = c.below(block);
        c.install(bottom, Tile::Below);
        if let Some(side) = side {
            c.install(c.adjacent(bottom, side), Tile::Below);
        }
    }
    0
}
