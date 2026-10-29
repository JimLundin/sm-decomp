//! The Shitroid's palette tables in bank $AD: the health-based shell and
//! innards colours and the colours it fades to black through in the cutscene.
pub use crate::table::LengthError;
use crate::table::Rows;

/// $AD:E7E2: colours 1..4 and Ah..Eh of sprite palette 7 (the shell), unused
/// first then from health >= 8C0h down.
pub const SHELL_PALETTE_ADDRESS: u32 = 0xad_e7e2;
/// $AD:E882: colours 5..9 of sprite palette 7 (the innards), in the same order.
pub const INNARDS_PALETTE_ADDRESS: u32 = 0xad_e882;
pub const HEALTH_PALETTE_ROWS: usize = 8;
pub const HEALTH_PALETTE_BYTES: usize = HEALTH_PALETTE_ROWS * 2;

/// $AD:E8E2: colours 1..Eh of sprite palette 7 while fading to black, unused
/// first.
pub const FADE_PALETTE_ADDRESS: u32 = 0xad_e8e2;
pub const FADE_PALETTE_ROWS: usize = 7;
pub const FADE_PALETTE_BYTES: usize = FADE_PALETTE_ROWS * 2;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Palette {
    /// Bank $AD pointer to the colours.
    pub colors: u16,
}

pub type HealthPalettes = Rows<Palette, HEALTH_PALETTE_ROWS>;
pub type FadePalettes = Rows<Palette, FADE_PALETTE_ROWS>;

pub fn import_health_palettes(bytes: &[u8]) -> Result<HealthPalettes, LengthError> {
    Rows::import(bytes, |[colors]| Palette { colors })
}

pub fn import_fade_palettes(bytes: &[u8]) -> Result<FadePalettes, LengthError> {
    Rows::import(bytes, |[colors]| Palette { colors })
}
