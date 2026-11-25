//! Shared colours for palette producers, transitions and video.

use serde::{Deserialize, Serialize};

/// A colour of the screen: 15 bits, three channels of 5 bits each, red in bits 0 to 4, green in
/// 5 to 9 and blue in 10 to 14.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "u16", into = "u16")]
pub struct Colour(u16);

impl TryFrom<u16> for Colour {
    type Error = &'static str;

    fn try_from(bits: u16) -> Result<Self, Self::Error> {
        Self::new(bits).ok_or("colour must have three five-bit channels")
    }
}

impl From<Colour> for u16 {
    fn from(colour: Colour) -> Self {
        colour.bits()
    }
}

/// The highest colour: every channel at 31, white.
const LAST_COLOUR: u16 = 0x7FFF;
/// A channel's 5 bits.
const CHANNEL: u16 = 0x1F;
/// Where green starts in a colour's bits.
const GREEN_AT: u32 = 5;
/// Where blue starts in a colour's bits.
const BLUE_AT: u32 = 10;

impl Colour {
    /// Black: every channel 0.
    pub const BLACK: Self = Self(0);

    /// The colour of these 15 bits, if they are 15 bits (`0000` to `7FFF`).
    #[must_use]
    pub const fn new(bits: u16) -> Option<Self> {
        if bits <= LAST_COLOUR {
            Some(Self(bits))
        } else {
            None
        }
    }

    /// Its 15 bits.
    #[must_use]
    pub const fn bits(self) -> u16 {
        self.0
    }

    /// Its channels, red, green and blue, each 0 to 31.
    #[must_use]
    pub const fn channels(self) -> [u16; 3] {
        [
            self.0 & CHANNEL,
            (self.0 >> GREEN_AT) & CHANNEL,
            (self.0 >> BLUE_AT) & CHANNEL,
        ]
    }

    /// The colour of these channels, red, green and blue; each keeps its low 5 bits.
    #[must_use]
    pub const fn from_channels([red, green, blue]: [u16; 3]) -> Self {
        Self((red & CHANNEL) | ((green & CHANNEL) << GREEN_AT) | ((blue & CHANNEL) << BLUE_AT))
    }
}

/// The number of ordered colours in a palette.
pub const PALETTE_SLOTS: usize = 256;

/// One complete ordered bank of colours. Every `u8` names a valid slot.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "Vec<Colour>", into = "Vec<Colour>")]
pub struct Palette(pub [Colour; PALETTE_SLOTS]);

impl TryFrom<Vec<Colour>> for Palette {
    type Error = &'static str;

    fn try_from(colours: Vec<Colour>) -> Result<Self, Self::Error> {
        colours
            .try_into()
            .map(Self)
            .map_err(|_| "palette must have 256 colours")
    }
}

impl From<Palette> for Vec<Colour> {
    fn from(palette: Palette) -> Self {
        Self::from(palette.0)
    }
}

/// The number of colours in one row of a palette, the colours a sprite draws from.
pub const ROW_COLOURS: usize = 16;

/// The 16 ordered colours of one row of a palette: a sprite's colours.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RowColours(pub [Colour; ROW_COLOURS]);

/// The authoritative banks, shared by palette producers and consumers.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Palettes {
    /// Colours video presents and transitions update.
    pub current: Palette,
    /// Destination colours read by transitions.
    pub target: Palette,
}
