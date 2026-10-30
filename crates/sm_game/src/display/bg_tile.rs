//! Background tilemap entries: tile number, palette, priority and flips.
#![forbid(unsafe_code)]

pub const H_FLIP: u16 = 0x4000;
pub const V_FLIP: u16 = 0x8000;
pub const PRIORITY: u16 = 0x2000;
/// The three palette bits of an entry.
pub const PALETTE_MASK: u16 = 7 << 10;
/// The tile number and palette bits; priority and flips lie above them.
pub const TILE_AND_PALETTE: u16 = 0x3ff | PALETTE_MASK;

/// The palette bits for `palette` (0 to 7).
pub const fn palette(palette: u16) -> u16 {
    palette << 10
}

/// An entry for tile `number` in `palette`, without priority or flips.
pub const fn tile(number: u16, palette: u16) -> u16 {
    self::palette(palette) | number
}

/// `entry` with its palette bits replaced by `palette_bits`.
pub const fn recolour(entry: u16, palette_bits: u16) -> u16 {
    entry & !PALETTE_MASK | palette_bits
}

const _: () = {
    assert!(PALETTE_MASK == 0x1c00);
    assert!(!TILE_AND_PALETTE == 0xe000);
    assert!(palette(2) == 0x0800 && palette(3) == 0x0c00 && palette(5) == 0x1400);
    assert!(palette(6) == 0x1800 && palette(7) == 0x1c00);
};
