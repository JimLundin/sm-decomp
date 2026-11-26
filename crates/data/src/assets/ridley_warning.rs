//! Ceres self-destruct warning artwork, in the room's tile and colour coordinates.

use crate::assets::art::{Cell, Colour, TileRef, TileSheet};

/// Part of a tile sheet installed by an art batch.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SheetPatch {
    /// The first tile of this batch in its destination sheet.
    pub first: TileRef,
    /// The consecutive tiles installed there.
    pub tiles: TileSheet,
}

/// The foreground sheet owner whose tiles are supplied by a batch.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ForegroundArt {
    /// Warning lettering in the room's foreground sheet.
    Lettering,
    /// The Ceres door pictures in the same sheet.
    Doors,
}

/// A batch of foreground tiles installed during the warning.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ForegroundPatch {
    /// The content supplied by the batch.
    pub art: ForegroundArt,
    /// Its location and tiles in the foreground sheet.
    pub patch: SheetPatch,
}

/// A cell position on the room's foreground map.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Place {
    /// The column, in tiles.
    pub column: u8,
    /// The row, in tiles.
    pub row: u8,
}

/// A horizontal strip of foreground cells in reading order.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Line {
    /// The position of the strip's first cell.
    pub first: Place,
    /// The placed tiles and their drawing attributes.
    pub cells: Vec<Cell>,
}

/// One positioned slot of the English text.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Letter {
    /// Where the slot lies on the map.
    pub place: Place,
    /// The cell to write, or no write for a space.
    pub glyph: Option<Cell>,
    /// Ordinary turns to wait after this slot.
    pub rest: u8,
}

/// The pictures used by the Ceres warning and the first escape-clock draws.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Art {
    /// Timer and lettering batches in the sprite sheet.
    pub sprites: [SheetPatch; 7],
    /// Lettering and door batches in the foreground sheet.
    pub foreground: [ForegroundPatch; 8],
    /// The fixed heading above the typed text.
    pub heading: Line,
    /// The subtitle's four fixed foreground strips.
    pub subtitle: [Line; 4],
    /// The English typing slots, including spaces.
    pub english: Vec<Letter>,
    /// The three heading colours in each glow step.
    pub glow: [[Colour; 3]; 16],
}
