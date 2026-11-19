//! Read-only geometry admitted from imported terrain or an independent source.

use types::{Block, Cell, Hitbox, Way, World};

/// Explicit geometry beyond the admitted rectangular grid.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Boundary {
    /// The admitted outside region is empty.
    Air,
    /// The admitted outside region is impassable.
    Solid,
}

/// Why collision geometry cannot be admitted.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WorldFault {
    /// Width must be nonzero and divide the complete grid.
    Shape,
    /// Every cell needs an explicit reaction permission.
    Permissions,
    /// A stand-in chain loops.
    StandInLoop(Cell),
    /// A stand-in names a cell outside this admitted grid.
    StandInOutside(Cell),
}

/// A checked rectangular grid and the supplied eligible actor boxes in their order.
/// No query mutates gameplay, terrain or reaction state.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct MovementWorld {
    width: u16,
    cells: Vec<Block>,
    lets_through: Vec<bool>,
    enemies: Vec<Hitbox>,
    boundary: Boundary,
}

impl MovementWorld {
    /// Validate complete saved geometry before an actual query.
    /// # Errors
    /// Malformed grids, permissions and stand-in chains are refused.
    pub fn validate(&self) -> Result<(), WorldFault> {
        Self::new(
            self.width,
            self.cells.clone(),
            self.lets_through.clone(),
            self.enemies.clone(),
            self.boundary,
        )
        .map(|_| ())
    }
    /// Admit complete typed geometry and explicit outside-region behaviour.
    ///
    /// # Errors
    /// Grid shape, missing permissions and invalid stand-in chains are refused.
    /// `lets_through` admits one direction-independent permission per cell;
    /// directional or opportunity-dependent reactions require another world provider.
    pub fn new(
        width: u16,
        cells: Vec<Block>,
        lets_through: Vec<bool>,
        enemies: Vec<Hitbox>,
        boundary: Boundary,
    ) -> Result<Self, WorldFault> {
        if width == 0
            || cells.is_empty()
            || !cells.len().is_multiple_of(usize::from(width))
            || cells
                .len()
                .checked_div(usize::from(width))
                .is_none_or(|height| height > usize::from(u16::MAX))
        {
            return Err(WorldFault::Shape);
        }
        if cells.len() != lets_through.len() {
            return Err(WorldFault::Permissions);
        }
        let world = Self {
            width,
            cells,
            lets_through,
            enemies,
            boundary,
        };
        for (index, _) in world.cells.iter().enumerate() {
            let start = Cell {
                x: u16::try_from(
                    index
                        .checked_rem(usize::from(width))
                        .ok_or(WorldFault::Shape)?,
                )
                .map_err(|_| WorldFault::Shape)?,
                y: u16::try_from(
                    index
                        .checked_div(usize::from(width))
                        .ok_or(WorldFault::Shape)?,
                )
                .map_err(|_| WorldFault::Shape)?,
            };
            let mut at = start;
            for visited in 0..=world.cells.len() {
                if visited == world.cells.len() {
                    return Err(WorldFault::StandInLoop(start));
                }
                let index = world.index(at).ok_or(WorldFault::StandInOutside(at))?;
                match world
                    .cells
                    .get(index)
                    .copied()
                    .ok_or(WorldFault::StandInOutside(at))?
                {
                    Block::StandsFor(Some(next)) => at = next,
                    Block::Air
                    | Block::Solid
                    | Block::Answered
                    | Block::Slope(_)
                    | Block::Stepped(_)
                    | Block::StandsFor(None) => break,
                }
            }
        }
        Ok(world)
    }

    pub(crate) fn contains(&self, at: Cell) -> bool {
        self.index(at).is_some()
    }

    fn index(&self, at: Cell) -> Option<usize> {
        if at.x >= self.width {
            return None;
        }
        let index = usize::from(at.y)
            .checked_mul(usize::from(self.width))?
            .checked_add(usize::from(at.x))?;
        (index < self.cells.len()).then_some(index)
    }
}

impl World for MovementWorld {
    fn block(&self, at: Cell) -> Block {
        self.index(at)
            .and_then(|index| self.cells.get(index))
            .copied()
            .unwrap_or(match self.boundary {
                Boundary::Air => Block::Air,
                Boundary::Solid => Block::Solid,
            })
    }

    fn lets_through(&self, at: Cell, _: Way) -> bool {
        self.index(at)
            .and_then(|index| self.lets_through.get(index))
            .copied()
            .unwrap_or(false)
    }

    fn solid_enemies(&self) -> &[Hitbox] {
        &self.enemies
    }
}
