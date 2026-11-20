//! Actual queried cells qualify the bounded ordinary terrain family.
use crate::production::world::MovementWorld;
use types::{Block, Cell, Hitbox, Way, World};

pub(super) struct Guard<'a> {
    pub world: &'a MovementWorld,
    pub rejected: std::cell::Cell<bool>,
}
impl Guard<'_> {
    fn qualify(&self, at: Cell) {
        if !self.world.contains(at)
            || !matches!(
                self.world.block(at),
                Block::Air
                    | Block::Solid
                    | Block::Slope(types::Slope {
                        faces: types::SlopeFaces::Floor,
                        ..
                    })
            )
        {
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

#[cfg(test)]
mod tests {
    use super::*;

    fn qualified(block: Block, at: Cell) -> bool {
        let world = MovementWorld::new(
            1,
            vec![block],
            vec![false],
            vec![],
            crate::production::world::Boundary::Air,
        )
        .unwrap();
        let guard = Guard {
            world: &world,
            rejected: std::cell::Cell::new(false),
        };
        let _ = guard.block(at);
        !guard.rejected.get()
    }

    #[test]
    fn current_family_admits_floor_surfaces_and_refuses_other_terrain() {
        let at = Cell { x: 0, y: 0 };
        let floor = types::Slope {
            faces: types::SlopeFaces::Floor,
            depth: [std::num::Wrapping(8); 16],
            factor: types::SlopeFactor(192),
        };
        for block in [Block::Air, Block::Solid, Block::Slope(floor)] {
            assert!(qualified(block, at));
        }
        let ceiling = types::Slope {
            faces: types::SlopeFaces::Ceiling,
            ..floor
        };
        for block in [
            Block::Slope(ceiling),
            Block::Stepped([[false; 2]; 2]),
            Block::Answered,
        ] {
            assert!(!qualified(block, at));
        }
        assert!(!qualified(Block::Air, Cell { x: 1, y: 0 }));
    }
}
