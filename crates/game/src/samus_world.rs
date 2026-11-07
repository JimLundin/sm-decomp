//! The room as the samus-motion model meets it: the engine's level, its block kinds and slope
//! shapes, and the solid enemies, answered in the model's terms (`clean_types::World`).
//!
//! Each block kind is named once, here, as `OtherBlockKinds` says it meets her motion. The
//! reactions a reached cell then starts (a scroll trigger, an item's block) aren't the
//! world's: the model reports the cells it reached and the caller applies them.

use clean_types::{
    AirKind, AnsweredKind, Axis, Block, Cell, Heading, Hitbox, Position, Quarters, Radius, Slope,
    SlopeFaces, SolidKind, World,
};
use data::level::slope::{Shape, Slopes, Square, Surface};
use data::level::{BlockKind, Bts, Offset, PlmChoice};

use crate::collision::solid_enemy::{SolidEnemies, obstacles};
use crate::collision::{Room, Touch, Touches};
use crate::state::room::BlockIndex;

/// An item's drawn block: a special block whose reaction lets her through.
const ITEM_BLOCK: PlmChoice = PlmChoice::Common(0x45);

/// The special air that changes the scrolling when she reaches it.
const SCROLL_TRIGGER: Bts = Bts(0x46);

/// Pixel rows in a block, less one: a ceiling's surface counts up from the block's bottom.
const LAST_ROW: u8 = 15;

/// The room and the frame's solid enemies.
#[derive(Clone, Debug)]
pub struct SamusWorld<'a> {
    room: Room<'a>,
    enemies: Vec<Hitbox>,
}

impl<'a> SamusWorld<'a> {
    /// The room's blocks and the interactive enemies that are solid to her or frozen.
    #[must_use]
    pub fn new(room: Room<'a>, enemies: SolidEnemies<'_>) -> Self {
        let enemies = obstacles(enemies.interactive, enemies.slots)
            .filter(|(_, enemy)| enemy.solid || enemy.frozen)
            .map(|(_, enemy)| Hitbox {
                x: enemy.position.x.pixels,
                y: enemy.position.y.pixels,
                radius: Radius {
                    x: enemy.radius_x,
                    y: enemy.radius_y,
                },
            })
            .collect();
        Self { room, enemies }
    }

    /// The level data index of `cell`: the row times the width, each by its low byte as the
    /// level's addressing takes them.
    fn index(&self, cell: Cell) -> BlockIndex {
        let [row, _] = cell.row.to_le_bytes();
        let [width, _] = self.room.width.0.to_le_bytes();
        BlockIndex(
            u16::from(row)
                .wrapping_mul(u16::from(width))
                .wrapping_add(cell.column),
        )
    }

    /// The level's kind at `cell`; past the level, air.
    fn kind(&self, cell: Cell) -> BlockKind {
        self.room
            .blocks
            .get(usize::from(self.index(cell).0))
            .map_or_else(BlockKind::default, |block| block.kind)
    }

    /// What reaching `cell` starts, if anything.
    #[must_use]
    pub fn reaction(&self, cell: Cell) -> Option<Touch> {
        let at = self.index(cell);
        let kind = self.kind(cell);
        if kind == BlockKind::SpecialAir(SCROLL_TRIGGER) {
            Some(Touch::Scroll(at))
        } else if kind == BlockKind::Special(ITEM_BLOCK) {
            Some(Touch::Item(at))
        } else {
            None
        }
    }

    /// The touches a frame's moves start: each reacting cell of `Outcome::reached`, in the
    /// order the moves reached them, whichever way the move went.
    #[must_use]
    pub fn touches(&self, reached: &[clean_types::Touch]) -> Touches {
        reached
            .iter()
            .filter_map(|touch| self.reaction(touch.cell))
            .collect()
    }

    /// The touch the cells she was inside at the frame's start start (`Outcome::inside`): only
    /// her centre's cell reacts, and only a scroll trigger (`$94:9956`). Her centre's cell is
    /// listed second, after her bottom edge's, when it is in another row; in the same row it
    /// isn't listed and nothing reacts.
    #[must_use]
    pub fn inside_touch(
        &self,
        inside: &[Cell],
        position: Position,
        radius: Radius,
    ) -> Option<Touch> {
        let y = position.y.pixel();
        let bottom = y.wrapping_add(radius.y).wrapping_sub(1);
        if (y >> 4) == (bottom >> 4) {
            return None;
        }
        let centre = *inside.get(1)?;
        self.reaction(centre)
            .filter(|touch| matches!(touch, Touch::Scroll(_)))
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
fn quarters(square: Square, slope: data::level::Slope) -> Quarters {
    let at =
        |bottom: bool, right: bool| square.solid_at(right ^ slope.x_flip, bottom ^ slope.y_flip);
    Quarters([
        [at(false, false), at(false, true)],
        [at(true, false), at(true, true)],
    ])
}

/// A sloped surface: a floor's surface row is its offset from the block's top; a ceiling
/// (flipped top to bottom) counts its offset up from the block's bottom row.
fn surface_of(surface: Surface, slope: data::level::Slope) -> Slope {
    let faces = if slope.y_flip {
        SlopeFaces::Ceiling
    } else {
        SlopeFaces::Floor
    };
    let mut rows = surface.tops;
    if slope.x_flip {
        rows.reverse();
    }
    if faces == SlopeFaces::Ceiling {
        for row in &mut rows {
            *row = LAST_ROW.saturating_sub(*row);
        }
    }
    Slope {
        faces,
        surface: rows,
        factor: surface.factor,
    }
}

impl World for SamusWorld<'_> {
    fn block(&self, cell: Cell) -> Block {
        match self.kind(cell) {
            BlockKind::Air(_) => Block::Air(AirKind::Plain),
            BlockKind::UnusedAir(_) => Block::Air(AirKind::Other),
            BlockKind::SpikeAir(_) => Block::Air(AirKind::Spiked),
            BlockKind::ShootableAir(_) => Block::Air(AirKind::ShotAir),
            BlockKind::BombableAir(_) => Block::Air(AirKind::BombAir),
            BlockKind::SpecialAir(_) => Block::Air(AirKind::Reacting),
            BlockKind::Solid(_) => Block::Solid(SolidKind::Plain),
            // The engine starts no door transition from her moves yet: every door is solid.
            BlockKind::Door(_) => Block::Solid(SolidKind::DoorToNowhere),
            BlockKind::Spike(_) => Block::Solid(SolidKind::Spikes),
            BlockKind::Shootable(_) => Block::Solid(SolidKind::ShotBlock),
            BlockKind::Grapple(_) => Block::Solid(SolidKind::GrappleBlock),
            BlockKind::Bombable(_) => Block::Answered(AnsweredKind::BombBlock),
            BlockKind::Special(_) => Block::Answered(AnsweredKind::Reacting),
            BlockKind::HorizontalExtension(Offset(by)) => Block::StandsFor(Axis::X, i16::from(by)),
            BlockKind::VerticalExtension(Offset(by)) => Block::StandsFor(Axis::Y, i16::from(by)),
            BlockKind::Slope(shape) => self
                .room
                .slopes
                .and_then(|slopes| slope(slopes, shape))
                .unwrap_or(Block::Air(AirKind::Plain)),
        }
    }

    /// Of the answered kinds only an item's block lets her through: a bomb block, a crumble
    /// block and a speed block are solid to the touch.
    fn lets_through(&self, cell: Cell, _: Heading) -> bool {
        matches!(self.kind(cell), BlockKind::Special(ITEM_BLOCK))
    }

    fn solid_enemies(&self) -> &[Hitbox] {
        &self.enemies
    }
}

pub mod frame;
pub mod pose;

#[cfg(test)]
mod tests;
