//! The per-Tick intents (MODEL §2): what the model shows, in its own terms. `exact` encodes
//! them as the original's bytes; the wide and HD renderers read them as they are.

use clean_types::Pair;
use data::spritemap::{SpriteAttributes, SpriteSize, Spritemap};

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
