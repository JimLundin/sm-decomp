//! The hardware encodings of the content's pictures (docs/architecture/assets.md §1 point 7).
//!
//! A tileset's block looks go out as the tile table's words, its sheet as 4bpp planar tiles in
//! VRAM, and its palette rows as colour RAM's words.

use data::assets::art::{BlockLook, PaletteSet, TILE_SIDE, Tile, TileSheet};

use crate::state::video::TilemapEntry;

/// The bytes of a 4bpp tile.
const TILE_BYTES: usize = 32;
/// The bytes of the last two bit planes' rows into a tile: the first two planes' come first.
const HIGH_PLANES: usize = 16;

/// The tile table of `looks`: each look's four tilemap words in reading order, low byte first.
#[must_use]
pub fn tile_table(looks: &[BlockLook]) -> Vec<u8> {
    looks
        .iter()
        .flat_map(|look| look.cells())
        .flat_map(|cell| TilemapEntry::from(cell).word().to_le_bytes())
        .collect()
}

/// The 4bpp planar bytes of `sheet`, as VRAM holds its tiles.
#[must_use]
pub fn planar_4bpp(sheet: &TileSheet) -> Vec<u8> {
    sheet.0.iter().flat_map(planar_tile).collect()
}

/// Row `y`'s pixels' bit planes 0 and 1 are bytes `2y` and `2y + 1`, planes 2 and 3 the same
/// 16 bytes on; pixel `x` is bit `7 - x` of each.
fn planar_tile(tile: &Tile) -> [u8; TILE_BYTES] {
    let mut bytes = [0; TILE_BYTES];
    let (low, high) = bytes.split_at_mut(HIGH_PLANES);
    for ((row, low), high) in tile
        .0
        .iter()
        .zip(low.as_chunks_mut::<2>().0)
        .zip(high.as_chunks_mut::<2>().0)
    {
        let mut planes = [0u8; 4];
        for (x, &pixel) in row.iter().enumerate() {
            let shift = TILE_SIDE.wrapping_sub(1).wrapping_sub(x);
            for (plane, byte) in planes.iter_mut().enumerate() {
                *byte |= ((pixel >> plane) & 1) << shift;
            }
        }
        let [zero, one, two, three] = planes;
        *low = [zero, one];
        *high = [two, three];
    }
    bytes
}

/// The colour words of `palettes`, row by row, low byte first, as colour RAM holds them.
#[must_use]
pub fn colour_words(palettes: &PaletteSet) -> Vec<u8> {
    palettes
        .0
        .iter()
        .flat_map(|row| row.0)
        .flat_map(|colour| colour.0.to_le_bytes())
        .collect()
}
