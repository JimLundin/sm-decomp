//! The room as the samus-motion model meets it: the engine's level, its block kinds and slope
//! shapes, and the solid enemies, answered in the model's terms (`clean_types::World`).
//!
//! Each block kind is named once, here, as `OtherBlockKinds` says it meets her motion. The
//! reactions a reached cell then starts (the blocks', an item's) aren't the world's: the model
//! reports the cells it reached and the caller applies them.

use clean_types::{Block, Cell, Hitbox, Pair, Radius, Slope, SlopeFaces, Way, World};
use core::num::Wrapping;
use data::level::slope::{Shape, Slopes, Square, Surface as Tops};
use data::level::{BlockKind, Offset, PlmChoice};

use crate::collision::solid_enemy::{Obstacle, SolidEnemies, obstacles};
use crate::collision::{Room, Touch, Touches};
use crate::state::room::BlockIndex;

/// An item's drawn block: a special block whose reaction lets her through.
const ITEM_BLOCK: PlmChoice = PlmChoice::Common(0x45);

/// A surface's depth for a column far past the block: the level's offsets past 16 all lie
/// there.
const FAR_PAST: u8 = 17;

/// The room and the frame's solid enemies.
pub struct SamusWorld<'a> {
    room: Room<'a>,
    enemies: Vec<Hitbox>,
}

impl<'a> SamusWorld<'a> {
    /// The room's blocks and the interactive enemies that are solid to her or frozen.
    #[must_use]
    pub fn new(room: Room<'a>, enemies: SolidEnemies<'_>) -> Self {
        Self::of(
            room,
            obstacles(enemies.interactive, enemies.slots).map(|(_, enemy)| enemy),
        )
    }

    /// The room's blocks and those of `enemies` that are solid to her or frozen.
    #[must_use]
    pub fn of(room: Room<'a>, enemies: impl IntoIterator<Item = Obstacle>) -> Self {
        let enemies = enemies
            .into_iter()
            .filter(|enemy| enemy.solid || enemy.frozen)
            .map(|enemy| Hitbox {
                centre: Pair {
                    x: Wrapping(enemy.position.x.pixels),
                    y: Wrapping(enemy.position.y.pixels),
                },
                radius: Radius {
                    x: Wrapping(enemy.radius_x),
                    y: Wrapping(enemy.radius_y),
                },
            })
            .collect();
        Self { room, enemies }
    }

    /// The level data index of `cell`: the row times the width, each by its low byte as the
    /// level's addressing takes them.
    fn index(&self, cell: Cell) -> BlockIndex {
        let [row, _] = cell.y.to_le_bytes();
        let [width, _] = self.room.width.0.to_le_bytes();
        BlockIndex(
            u16::from(row)
                .wrapping_mul(u16::from(width))
                .wrapping_add(cell.x),
        )
    }

    /// The cell at a level data index: the inverse of [`Self::index`], a zero width leaving the
    /// index as the column.
    fn cell(&self, index: u16) -> Cell {
        let [width, _] = self.room.width.0.to_le_bytes();
        let width = u16::from(width);
        Cell {
            x: index.checked_rem(width).unwrap_or(index),
            y: index.checked_div(width).unwrap_or(0),
        }
    }

    /// The cell an extension at `cell` names, `by` blocks along its row or column; an offset of
    /// 0 names nothing.
    fn extended(&self, cell: Cell, by: i8, rows: bool) -> Option<Cell> {
        let step = i16::from(by).cast_unsigned();
        let step = if rows {
            step.wrapping_mul(self.room.width.0)
        } else {
            step
        };
        (by != 0).then(|| self.cell(self.index(cell).0.wrapping_add(step)))
    }

    /// The level's kind at `cell`; past the level, air.
    fn kind(&self, cell: Cell) -> BlockKind {
        self.room
            .blocks
            .get(usize::from(self.index(cell).0))
            .map_or_else(BlockKind::default, |block| block.kind)
    }

    /// What reaching a cell starts, if anything: an item's block its item's touch, a reacting
    /// cell of another kind the blocks'.
    #[must_use]
    pub fn reaction(&self, touch: clean_types::Touch) -> Option<Touch> {
        match self.kind(touch.cell) {
            BlockKind::Special(ITEM_BLOCK) => Some(Touch::Item(self.index(touch.cell))),
            BlockKind::Special(_)
            | BlockKind::Shootable(_)
            | BlockKind::Bombable(_)
            | BlockKind::SpecialAir(_)
            | BlockKind::ShootableAir(_)
            | BlockKind::BombableAir(_) => Some(Touch::Blocks {
                cell: touch.cell,
                way: touch.way,
            }),
            BlockKind::Air(_)
            | BlockKind::Slope(_)
            | BlockKind::SpikeAir(_)
            | BlockKind::HorizontalExtension(_)
            | BlockKind::UnusedAir(_)
            | BlockKind::Solid(_)
            | BlockKind::Door(_)
            | BlockKind::Spike(_)
            | BlockKind::VerticalExtension(_)
            | BlockKind::Grapple(_) => None,
        }
    }

    /// The touches a frame's moves start: each reacting cell of `Outcome::reached`, in the
    /// order the moves reached them, whichever way the move went.
    #[must_use]
    pub fn touches(&self, reached: &[clean_types::Touch]) -> Touches {
        reached
            .iter()
            .filter_map(|&touch| self.reaction(touch))
            .collect()
    }
}

/// A slope's shape in the model's terms, by the level's flips; `None` where the tables name
/// no shape.
fn slope(slopes: &Slopes, slope: data::level::Slope) -> Option<Block> {
    Some(match slopes.shape(slope.shape)? {
        Shape::Square(square) => Block::Stepped(quarters(square, slope)),
        Shape::Surface(surface) => Block::Slope(surface_of(surface, slope)),
    })
}

/// A square slope's quarters, by row then column, each read through the flips.
fn quarters(square: Square, slope: data::level::Slope) -> [[bool; 2]; 2] {
    let at =
        |bottom: bool, right: bool| square.solid_at(right ^ slope.x_flip, bottom ^ slope.y_flip);
    [
        [at(false, false), at(false, true)],
        [at(true, false), at(true, true)],
    ]
}

/// A sloped surface. The level counts each column's surface from the block's open side, its
/// top for a floor and its bottom for a ceiling (flipped top to bottom), as the model's depth
/// does; 16 is a column with nothing solid, just past the block, and a greater offset lies far
/// past it.
fn surface_of(surface: Tops, slope: data::level::Slope) -> Slope {
    let faces = if slope.y_flip {
        SlopeFaces::Ceiling
    } else {
        SlopeFaces::Floor
    };
    let mut tops = surface.tops;
    if slope.x_flip {
        tops.reverse();
    }
    Slope {
        faces,
        depth: tops.map(|top| Wrapping(u16::from(top.min(FAR_PAST)))),
        factor: Wrapping(u32::from(surface.factor)),
    }
}

impl World for SamusWorld<'_> {
    fn block(&self, cell: Cell) -> Block {
        match self.kind(cell) {
            BlockKind::Air(_)
            | BlockKind::UnusedAir(_)
            | BlockKind::SpikeAir(_)
            | BlockKind::ShootableAir(_)
            | BlockKind::BombableAir(_)
            | BlockKind::SpecialAir(_) => Block::Air,
            // The engine starts no door transition from her moves yet: every door is solid.
            BlockKind::Solid(_)
            | BlockKind::Door(_)
            | BlockKind::Spike(_)
            | BlockKind::Shootable(_)
            | BlockKind::Grapple(_) => Block::Solid,
            BlockKind::Bombable(_) | BlockKind::Special(_) => Block::Answered,
            BlockKind::HorizontalExtension(Offset(by)) => {
                Block::StandsFor(self.extended(cell, by, false))
            }
            BlockKind::VerticalExtension(Offset(by)) => {
                Block::StandsFor(self.extended(cell, by, true))
            }
            BlockKind::Slope(shape) => self
                .room
                .slopes
                .and_then(|slopes| slope(slopes, shape))
                .unwrap_or(Block::Air),
        }
    }

    /// Of the answered kinds only an item's block lets her through: a bomb block, a crumble
    /// block and a speed block are solid to the touch.
    fn lets_through(&self, cell: Cell, _: Way) -> bool {
        matches!(self.kind(cell), BlockKind::Special(ITEM_BLOCK))
    }

    fn solid_enemies(&self) -> &[Hitbox] {
        &self.enemies
    }
}

pub mod frame;
pub mod state;
pub mod step;

#[cfg(test)]
mod tests;
