//! The per-Tick intents (MODEL §2): what the model shows, in its own terms. `exact` encodes
//! them as the original's bytes; the wide and HD renderers read them as they are.

use clean_types::Pair;
use data::spritemap::{SpriteAttributes, SpriteSize, Spritemap};

use crate::state::room::BlockPoint;
use crate::state::video::TilemapEntry;

/// A point on the screen, in pixels from the picture's top-left corner, at full precision.
///
/// A piece left of or above the picture has a negative coordinate, and one past its right or
/// bottom edge a coordinate past the picture's size.
pub type ScreenPoint = Pair<i32>;

/// How a piece is placed among the frame's sprites.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Placing {
    /// One of a picture's pieces, small or large.
    Mapped(SpriteSize),
    /// A small sprite placed by itself.
    Lone,
}

/// One piece of a sprite: one of the sprite tiles, drawn with its look at a point.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SpritePiece {
    /// Its top-left corner.
    pub at: ScreenPoint,
    /// Its picture, palette, priority and flips.
    pub look: SpriteAttributes,
    /// How it is placed.
    pub placing: Placing,
}

/// What a domain draws into the frame's sprites: its pieces, the first drawn in front.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SpriteDraw {
    /// The pieces, in the order they are drawn.
    pub pieces: Vec<SpritePiece>,
}

impl SpriteDraw {
    /// Add `map`'s pieces, each placed from `origin` by its offset.
    pub fn place(&mut self, map: &Spritemap, origin: ScreenPoint) {
        self.pieces
            .extend(map.entries.iter().map(|entry| SpritePiece {
                at: Pair {
                    x: origin.x.wrapping_add(i32::from(entry.x)),
                    // Read signed, as pieces above the origin are drawn.
                    y: origin.y.wrapping_add(i32::from(entry.y.cast_signed())),
                },
                look: entry.attributes,
                placing: Placing::Mapped(entry.size),
            }));
    }

    /// Add one lone sprite.
    pub fn lone(&mut self, at: ScreenPoint, look: SpriteAttributes) {
        self.pieces.push(SpritePiece {
            at,
            look,
            placing: Placing::Lone,
        });
    }
}

/// The screen coordinate of a room pixel `at` seen from a camera at `camera`: their difference,
/// signed.
#[must_use]
pub fn on_screen(at: u16, camera: u16) -> i32 {
    i32::from(at.wrapping_sub(camera).cast_signed())
}

/// What one block of the room shows: its four cells, each a picture with its palette,
/// priority and flips.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BlockLook {
    /// The top-left cell.
    pub top_left: TilemapEntry,
    /// The top-right cell.
    pub top_right: TilemapEntry,
    /// The bottom-left cell.
    pub bottom_left: TilemapEntry,
    /// The bottom-right cell.
    pub bottom_right: TilemapEntry,
}

/// Whole rows of the overlay, the screen-fixed layer the HUD and the message box show on:
/// `cells` row by row, 32 to a row, from row `row` down.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OverlayRows {
    /// The first row written, from the overlay's top.
    pub row: u16,
    /// The cells, row-major.
    pub cells: Vec<TilemapEntry>,
}

/// What a domain writes to a background layer: where, and the looks it writes there.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TileWrite {
    /// Rows of the overlay.
    Overlay(OverlayRows),
    /// A row of the room's blocks on layer 1, rightward from the block at `at`.
    Blocks {
        /// The row's first block.
        at: BlockPoint,
        /// Each block's look, left to right.
        looks: Vec<BlockLook>,
    },
}

/// One of the room's two level layers, each shown on its own background.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LevelLayer {
    /// Layer 1, the playfield's blocks, on BG1.
    One,
    /// Layer 2, the room's back layer when it has level data, on BG2.
    Two,
}

/// A line of a layer's blocks.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BlockLine {
    /// A column, downward.
    Column,
    /// A row, rightward.
    Row,
}

impl BlockLine {
    /// The blocks a line coming into view holds: a column the screen's 16 down; a row 17
    /// across, as a screen scrolled part of a block across shows parts of 17.
    #[must_use]
    pub const fn blocks(self) -> u16 {
        match self {
            Self::Column => 16,
            Self::Row => 17,
        }
    }
}

/// A line of a layer's blocks coming into view as the camera moves into another block column
/// or row.
///
/// Its blocks are read from the layer's level at `level`, and land on the layer's background
/// at `background`: the background's scroll moves it by the scroll offset from the layer, so
/// the two can differ.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LayerExposure {
    /// The layer.
    pub layer: LevelLayer,
    /// A column or a row.
    pub line: BlockLine,
    /// The line's first block in the layer's level.
    pub level: BlockPoint,
    /// The block of the background the first block lands on.
    pub background: BlockPoint,
}
