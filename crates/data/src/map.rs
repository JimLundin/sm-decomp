//! The area maps: each area's map tilemap, the pictures past its last column and the cells its map
//! station reveals, and the bitmap layout the explored cells share with the station's.
//!
//! An area's map is 64 by 32 cells, two pages of 32 by 32 side by side. A bitmap of it keeps one
//! bit per cell, four bytes per row and 80h bytes per page, the most significant bit leftmost; the
//! map tilemap keeps one tilemap word per cell, 20h words per row and 400h words per page.

use canonical::Canonical;

use crate::rom::{AddressError, Lorom, RomAddress};
use crate::room::Area;

/// The map tilemaps' long pointers, one per area.
const MAP_TILEMAP_POINTERS: RomAddress = RomAddress::new(0x82, 0x964a);
/// The map station bitmaps' pointers, one per area.
const MAP_STATION_POINTERS: RomAddress = RomAddress::new(0x82, 0x9717);
/// The areas with a map: the debug area's pointers name no map.
pub const MAPPED_AREAS: [Area; 7] = [
    Area::Crateria,
    Area::Brinstar,
    Area::Norfair,
    Area::WreckedShip,
    Area::Maridia,
    Area::Tourian,
    Area::Ceres,
];
/// A map's columns: two pages of [`PAGE_COLUMNS`] side by side.
pub const MAP_COLUMNS: u8 = 64;
/// A page's columns.
pub const PAGE_COLUMNS: u8 = 32;
/// A map's rows, the top one padding that no room is on.
pub const MAP_ROWS: u8 = 32;
/// A map bitmap's bytes: a page's rows top to bottom, then the other page's, four bytes a row,
/// the leftmost cell the most significant bit.
pub const MAP_BIT_BYTES: usize = 0x100;
/// A map tilemap's words: a page's rows top to bottom, then the other page's, a word a cell.
pub const MAP_TILEMAP_WORDS: usize = 0x800;
/// The words read past an area's tilemap: the next tilemap's left page, whose first columns the
/// mini-map shows past the map's last column.
pub const BEYOND_WORDS: usize = 0x400;
/// A bitmap's bytes per page.
const PAGE_BYTES: usize = 0x80;
/// A bitmap's bytes per page row.
const ROW_BYTES: usize = 4;

/// A set of a map's cells: the cells explored, or the cells a map station reveals. Row `y`'s
/// word holds the cell in column `x` at bit `x`.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MapCells(pub [u64; 32]);

/// The row and the bit in its word of the cell that bit `bit` (0 the most significant) of bitmap
/// byte `byte` holds.
fn bitmap_cell(byte: usize, bit: u8) -> (usize, u64) {
    let page = byte / PAGE_BYTES;
    let row = byte % PAGE_BYTES / ROW_BYTES;
    let column = page
        .wrapping_mul(usize::from(PAGE_COLUMNS))
        .wrapping_add((byte % ROW_BYTES).wrapping_mul(8))
        .wrapping_add(usize::from(bit));
    let cell = u32::try_from(column)
        .ok()
        .and_then(|column| 1_u64.checked_shl(column))
        .unwrap_or(0);
    (row, cell)
}

/// A bitmap byte's bits, the most significant first.
const BITS: [u8; 8] = [0, 1, 2, 3, 4, 5, 6, 7];

impl MapCells {
    /// The set a bitmap in the original's layout holds.
    #[must_use]
    pub fn from_bitmap(bitmap: &[u8; MAP_BIT_BYTES]) -> Self {
        let mut cells = Self::default();
        for (index, &byte) in bitmap.iter().enumerate() {
            for &bit in BITS
                .iter()
                .filter(|&&bit| byte.wrapping_shl(u32::from(bit)) & 0x80 != 0)
            {
                let (row, cell) = bitmap_cell(index, bit);
                if let Some(word) = cells.0.get_mut(row) {
                    *word |= cell;
                }
            }
        }
        cells
    }

    /// The set as a bitmap in the original's layout.
    #[must_use]
    pub fn to_bitmap(&self) -> [u8; MAP_BIT_BYTES] {
        core::array::from_fn(|index| {
            BITS.iter().fold(0, |byte, &bit| {
                let (row, cell) = bitmap_cell(index, bit);
                let set = self.0.get(row).is_some_and(|word| word & cell != 0);
                (byte << 1) | u8::from(set)
            })
        })
    }
}

/// One area's map.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AreaMap {
    /// The area.
    pub area: Area,
    /// The map tilemap's words, a page's rows then the other's, as the PPU reads them.
    pub tiles: Vec<u16>,
    /// The [`BEYOND_WORDS`] words after them, laid out as a page.
    pub beyond: Vec<u16>,
    /// The cells the area's map station reveals.
    pub station: MapCells,
}

/// Every area's map, from the ROM.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct AreaMaps(pub Vec<AreaMap>);

impl AreaMaps {
    /// The maps of [`MAPPED_AREAS`] from the ROM.
    ///
    /// # Errors
    ///
    /// [`AddressError`] when a pointer table, a tilemap or a bitmap is outside the image.
    pub fn load(rom: Lorom<'_>) -> Result<Self, AddressError> {
        let mut maps = Vec::with_capacity(MAPPED_AREAS.len());
        for (index, area) in (0_usize..).zip(MAPPED_AREAS) {
            let long = read_array::<3>(rom, MAP_TILEMAP_POINTERS, index.wrapping_mul(3))?;
            let tilemap = RomAddress::from_long(long);
            let read = MAP_TILEMAP_WORDS.wrapping_add(BEYOND_WORDS);
            let mut tiles: Vec<u16> = rom
                .read(tilemap, read.wrapping_mul(2))?
                .as_chunks::<2>()
                .0
                .iter()
                .map(|&word| u16::from_le_bytes(word))
                .collect();
            let beyond = tiles.split_off(MAP_TILEMAP_WORDS);
            let near = read_array::<2>(rom, MAP_STATION_POINTERS, index.wrapping_mul(2))?;
            let bitmap = RomAddress::new(MAP_STATION_POINTERS.bank, u16::from_le_bytes(near));
            let bytes: &[u8; MAP_BIT_BYTES] = rom
                .read(bitmap, MAP_BIT_BYTES)?
                .first_chunk()
                .ok_or(AddressError::PastEnd(bitmap))?;
            maps.push(AreaMap {
                area,
                tiles,
                beyond,
                station: MapCells::from_bitmap(bytes),
            });
        }
        Ok(Self(maps))
    }
}

/// The `N` bytes `offset` bytes into the table at `table`.
fn read_array<const N: usize>(
    rom: Lorom<'_>,
    table: RomAddress,
    offset: usize,
) -> Result<[u8; N], AddressError> {
    let at = table.advance(offset).ok_or(AddressError::PastEnd(table))?;
    rom.read(at, N)?
        .first_chunk()
        .copied()
        .ok_or(AddressError::PastEnd(at))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rom::Rom;

    /// The ROM, from `SM_ROM` or the checkout's `rom/sm.smc`; `None` where it isn't present.
    fn rom() -> Option<Rom> {
        let path = std::env::var_os("SM_ROM").map_or_else(
            || std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../rom/sm.smc"),
            std::path::PathBuf::from,
        );
        let image = std::fs::read(path).ok()?;
        Rom::new(image).ok()
    }

    #[test]
    fn a_cell_set_round_trips_the_bitmap() {
        let bitmap: [u8; MAP_BIT_BYTES] =
            core::array::from_fn(|index| u8::try_from(index).unwrap().wrapping_mul(37));
        let cells = MapCells::from_bitmap(&bitmap);
        assert_eq!(cells.to_bitmap(), bitmap);
        // Byte 81h's most significant bit is the right page's top row, column 8; byte 7Fh's
        // least significant the left page's last row, column 31.
        let mut one = [0; MAP_BIT_BYTES];
        one[0x81] = 0x80;
        one[0x7F] = 0x01;
        let mut expected = MapCells::default();
        expected.0[0] = 1 << 40;
        expected.0[31] = 1 << 31;
        assert_eq!(MapCells::from_bitmap(&one), expected);
    }

    #[test]
    fn every_mapped_area_loads_from_the_rom() {
        let Some(cartridge) = rom() else {
            return;
        };
        let maps = AreaMaps::load(cartridge.lorom()).unwrap();
        assert_eq!(maps.0.len(), MAPPED_AREAS.len());
        for map in &maps.0 {
            assert_eq!(map.tiles.len(), MAP_TILEMAP_WORDS);
            assert_eq!(map.beyond.len(), BEYOND_WORDS);
            // Every map's top row is padding: no room is on it, so no station reveals it.
            assert_eq!(map.station.0[0], 0);
        }
        // Brinstar's map station reveals cells; the map tiles are in bank $B5, and Brinstar's
        // beyond pictures are Crateria's map, the block after its own.
        let of = |area| maps.0.iter().find(|map| map.area == area).unwrap();
        let brinstar = of(Area::Brinstar);
        assert_ne!(brinstar.station, MapCells::default());
        assert!(brinstar.tiles.iter().any(|&word| word & 0x03ff != 0x001f));
        assert_eq!(brinstar.beyond, of(Area::Crateria).tiles[..BEYOND_WORDS]);
    }
}

// ROM sources for the data above, kept out of the public docs (the original's bank and
// RAM addresses, as the decoders use them):
// Bank $90 log, update mini-map `$90:A91B`, and its RAM map note on `$7E:07F7`: an area's
// The map tilemaps' long pointers, one per area (`$90:AA7C`: `$82:964A + [area] * 3`).
// The map station bitmaps' pointers into bank $82, one per area (`$90:A9C8`:
// `$82:9717 + [area] * 2`). The area maps are consecutive 1000h-byte blocks of bank $B5
// (Brinstar `$8000`, Crateria `$9000`, ...), so the words past one are the next block's.
