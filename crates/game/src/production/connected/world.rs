//! Actual queried cells qualify the bounded ordinary terrain family.
use crate::production::world::MovementWorld;
use types::{Block, Cell, Hitbox, Way, World};

pub(super) struct Guard<'a> {
    pub world: &'a MovementWorld,
    pub rejected: std::cell::Cell<bool>,
}
impl Guard<'_> {
    fn qualify(&self, at: Cell) {
        if !self.world.contains(at) || !matches!(self.world.block(at), Block::Air | Block::Solid) {
            self.rejected.set(true);
        }
    }
}
impl World for Guard<'_> {
    fn block(&self, at: Cell) -> Block {
        self.qualify(at);
        self.world.block(at)
    }
    fn lets_through(&self, at: Cell, way: Way) -> bool {
        self.qualify(at);
        self.world.lets_through(at, way)
    }
    fn solid_enemies(&self) -> &[Hitbox] {
        self.world.solid_enemies()
    }
}
