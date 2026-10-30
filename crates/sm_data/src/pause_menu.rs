//! Pause menu map screen tables in bank $82.
pub use crate::table::LengthError;
use crate::table::Rows;
pub use crate::table::import_words;

/// $82:C749: two spritemap IDs; the area labels draw from the first.
pub const LABEL_SPRITEMAP_ADDRESS: u32 = 0x82_c749;
pub const LABEL_SPRITEMAPS: usize = 2;
/// $82:C74D: each area's bank $82 map icon list, ended by $FFFF.
pub const MAP_ICON_ADDRESS: u32 = 0x82_c74d;
pub const MAP_ICON_AREAS: usize = 6;
/// $82:9717: each area's map data pointer.
pub const AREA_MAP_ADDRESS: u32 = 0x82_9717;
pub const AREA_MAPS: usize = 8;
/// $82:C0FA: palette bits per pause menu sprite animation.
pub const ANIMATION_PALETTE_ADDRESS: u32 = 0x82_c0fa;
/// $82:C1E4: animation spritemap base IDs pointer per animation.
pub const ANIMATION_SPRITEMAP_ADDRESS: u32 = 0x82_c1e4;
pub const ANIMATIONS: usize = 9;
/// $82:C18E: each equipment category's bank $82 list of selector positions.
pub const SELECTOR_POSITION_ADDRESS: u32 = 0x82_c18e;
pub const SELECTOR_CATEGORIES: usize = 4;

/// $82:A987: the map screen's palette animation, one sixteen-colour palette
/// per animation frame.
pub const PALETTE_ANIMATION_ADDRESS: u32 = 0x82_a987;
pub const PALETTE_FRAMES: usize = 14;
pub const PALETTE_ANIMATION_BYTES: usize = 32 * PALETTE_FRAMES;

pub type Palette = [u16; 16];
pub type PaletteAnimation = Rows<Palette, PALETTE_FRAMES>;

pub fn import_palette_animation(bytes: &[u8]) -> Result<PaletteAnimation, LengthError> {
    Rows::import(bytes, |colors| colors)
}
