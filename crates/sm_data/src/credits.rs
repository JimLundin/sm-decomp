//! Post-credits tables in bank $8B: the Super Metroid icon and Samus beam
//! VRAM transfers, the icon's grey-out palettes and the item percentage.
pub use crate::table::LengthError;
use crate::table::Rows;
pub use crate::table::import_words;

/// One VRAM transfer queue entry's ROM fields.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct VramTransfer {
    pub size: u16,
    pub source: u16,
    /// Source bank, in the low byte.
    pub bank: u16,
    pub destination: u16,
}

/// $8B:E45A: the Super Metroid icon's transfers, one per frame.
pub const ICON_TRANSFER_ADDRESS: u32 = 0x8b_e45a;
pub const ICON_TRANSFERS: usize = 6;
pub const ICON_TRANSFER_BYTES: usize = 8 * ICON_TRANSFERS;
pub type IconTransfers = Rows<VramTransfer, ICON_TRANSFERS>;

pub fn import_icon_transfers(bytes: &[u8]) -> Result<IconTransfers, LengthError> {
    Rows::import(bytes, |[size, source, bank, destination]| VramTransfer {
        size,
        source,
        bank,
        destination,
    })
}

/// $8B:E5E7: per grey-out step, the bank $8C ends of the icon's two
/// sixteen-colour palettes.
pub const ICON_PALETTE_ADDRESS: u32 = 0x8b_e5e7;
pub const ICON_PALETTE_STEPS: usize = 16;
pub const ICON_PALETTE_BYTES: usize = 4 * ICON_PALETTE_STEPS;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct IconPalettes {
    /// Last colour of palette 1's source.
    pub first: u16,
    /// Last colour of palette 15's source.
    pub second: u16,
}

pub type IconPaletteSteps = Rows<IconPalettes, ICON_PALETTE_STEPS>;

pub fn import_icon_palettes(bytes: &[u8]) -> Result<IconPaletteSteps, LengthError> {
    Rows::import(bytes, |[first, second]| IconPalettes { first, second })
}

/// $8B:E70D: the item percentage tables, up to $8B:E769: the RAM addresses
/// of the five capacities and their divisors, the eleven item bits, the five
/// beam bits, and each digit's two tiles.
pub const CAPACITY_ADDRESS: u32 = 0x8b_e70d;
pub const CAPACITY_DIVISOR_ADDRESS: u32 = 0x8b_e717;
pub const ITEM_BIT_ADDRESS: u32 = 0x8b_e721;
pub const BEAM_BIT_ADDRESS: u32 = 0x8b_e737;
pub const DIGIT_TILE_ADDRESS: u32 = 0x8b_e741;
pub const CAPACITIES: usize = 5;
pub const ITEMS: usize = 11;
pub const BEAMS: usize = 5;
pub const DIGITS: usize = 10;

pub type Capacities = Rows<u16, CAPACITIES>;
pub type ItemBits = Rows<u16, ITEMS>;
pub type BeamBits = Rows<u16, BEAMS>;
/// Top and bottom tiles per digit.
pub type DigitTiles = Rows<[u16; 2], DIGITS>;

/// $8B:F6B8 and $8B:F6D8: the Samus beam's source and destination per frame.
pub const BEAM_SOURCE_ADDRESS: u32 = 0x8b_f6b8;
pub const BEAM_DESTINATION_ADDRESS: u32 = 0x8b_f6d8;
pub const BEAM_FRAMES: usize = 16;
pub type BeamTransfers = Rows<u16, BEAM_FRAMES>;

pub fn import_digit_tiles(bytes: &[u8]) -> Result<DigitTiles, LengthError> {
    Rows::import(bytes, |tiles| tiles)
}
