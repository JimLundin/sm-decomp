//! The slope shapes a slope block's BTS names, as Samus's block collision reads them.
//!
//! Shapes 0-4 are the square slopes, a block of four solid or empty quadrants; shapes 5-0x1F
//! the non-square ones,
//! a surface height for each pixel column and a
//! factor that scales a walk along them.

use crate::rom::{AddressError, Lorom, RomAddress};

/// The surface heights: 32 shapes of 16 bytes, one per pixel column.
const SURFACES: RomAddress = RomAddress::new(0x94, 0x8B2B);
/// The slope X speed modifiers: 32 entries of two words, the factor second.
const FACTORS: RomAddress = RomAddress::new(0x94, 0x8586);
/// The square slopes: five shapes of four quadrant bytes.
const SQUARES: RomAddress = RomAddress::new(0x94, 0x8E54);

/// Slope shapes a BTS can name: its low five bits.
pub const SHAPES: usize = 32;
/// The square shapes, which come first.
pub const SQUARE_SHAPES: usize = 5;
/// Pixel columns in a block.
const COLUMNS: usize = 16;
/// Bytes a factor entry takes: the unused speed modifier, then the factor.
const FACTOR_BYTES: usize = 4;
/// A quadrant byte with this bit set is solid (the collision tests it with `BMI`).
const SOLID: u8 = 0x80;
/// The surface height's bits (`AND #$001F`).
const HEIGHT: u8 = 0x1f;

/// A square slope's quadrants, in the table's order: top left, top right, bottom left,
/// bottom right.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Square {
    /// Which of the four quadrants are solid.
    pub solid: [bool; 4],
}

impl Square {
    /// Whether the quadrant at `right`, `bottom` of the unflipped shape is solid: the
    /// collision's table index is the shape's base plus the column bit plus twice the row bit.
    #[must_use]
    pub fn solid_at(self, right: bool, bottom: bool) -> bool {
        let index = usize::from(right) | (usize::from(bottom) << 1);
        self.solid.get(index).copied().unwrap_or(false)
    }
}

/// A non-square slope's surface: how far below the block's top it is in each pixel column,
/// with the factor a walk along it moves her by.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Surface {
    /// The height's offset from the block's top, a column each, five bits as the collision
    /// masks them.
    pub tops: [u8; COLUMNS],
    /// The walk's distance multiplier, times $100.
    pub factor: u16,
}

impl Surface {
    /// The surface's offset in `column`, its low four bits.
    #[must_use]
    pub fn top(self, column: u16) -> u8 {
        let index = usize::from(column & 0x000f);
        self.tops.get(index).copied().unwrap_or(0)
    }
}

/// Every slope shape.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Slopes {
    squares: Vec<Square>,
    surfaces: Vec<Surface>,
}

/// A slope shape as the collision dispatches on it (shapes below 5 are square).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Shape {
    /// A square slope.
    Square(Square),
    /// A non-square slope.
    Surface(Surface),
}

impl Slopes {
    /// The shapes in the ROM.
    ///
    /// # Errors
    ///
    /// [`AddressError`] when a table is outside the image.
    pub fn load(rom: Lorom<'_>) -> Result<Self, AddressError> {
        let squares = rom.read(SQUARES, SQUARE_SHAPES.wrapping_mul(4))?;
        let surfaces = rom.read(SURFACES, SHAPES.wrapping_mul(COLUMNS))?;
        let factors = rom.read(FACTORS, SHAPES.wrapping_mul(FACTOR_BYTES))?;
        Ok(Self::decode(squares, surfaces, factors))
    }

    /// The shapes in the three tables' bytes; a short table leaves its last shapes empty.
    #[must_use]
    pub fn decode(squares: &[u8], surfaces: &[u8], factors: &[u8]) -> Self {
        let squares = squares
            .as_chunks::<4>()
            .0
            .iter()
            .map(|quadrants| Square {
                solid: quadrants.map(|byte| byte & SOLID != 0),
            })
            .collect();
        let factors = factors.as_chunks::<FACTOR_BYTES>().0;
        let surfaces = surfaces
            .as_chunks::<COLUMNS>()
            .0
            .iter()
            .zip(factors)
            .map(|(tops, &[_, _, low, high])| Surface {
                tops: tops.map(|byte| byte & HEIGHT),
                factor: u16::from_le_bytes([low, high]),
            })
            .collect();
        Self { squares, surfaces }
    }

    /// The shape a slope BTS's low five bits name; `None` past the tables.
    #[must_use]
    pub fn shape(&self, shape: u8) -> Option<Shape> {
        let index = usize::from(shape);
        if index < SQUARE_SHAPES {
            self.squares.get(index).copied().map(Shape::Square)
        } else {
            self.surfaces.get(index).copied().map(Shape::Surface)
        }
    }
}

#[cfg(test)]
mod tests;

// ROM sources for the data above, kept out of the public docs (the original's bank and
// RAM addresses, as the decoders use them):
// Bank `$94` log (<https://patrickjohnston.org/bank/94>): shapes 0-4 are the square slopes,
// a block of four solid or empty quadrants (`$94:8E54`); shapes 5-1F the non-square ones,
// a surface height for each pixel column (`$94:8B2B`, "slope top Y offset by X pixel") and a
// factor that scales a walk along them (`$94:8586`, the second word of each entry).
// The slope X speed modifiers: 32 entries of two words, the factor second (`$94:8588`).
// A slope shape as the collision dispatches on it (`$94:8FBB`, `$94:8FDA`: shapes below 5
